use super::errors::{AppAccessTokenError, ValidationError};
use crate::{
    client::Client,
    id::TwitchTokenErrorResponse,
    tokens::{errors::RefreshTokenError, Scope, TwitchToken},
    types::{AccessToken, ClientId, ClientSecret, RefreshToken},
};

/// An App Access Token from the [OAuth client credentials flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-client-credentials-flow)
///
/// Used for server-to-server requests. Use [`UserToken`](super::UserToken) for requests that need to be in the context of an authenticated user.
///
/// In some contexts (i.e [EventSub](https://dev.twitch.tv/docs/eventsub)) an App Access Token can be used in the context of users that have authenticated
/// the specific Client ID
#[derive(Clone)]
pub struct AppAccessToken {
    /// The access token used to authenticate requests with
    pub access_token: AccessToken,
    /// The refresh token used to extend the life of this user token
    pub refresh_token: Option<RefreshToken>,
    /// Expiration from when the response was generated.
    expires_in: std::time::Duration,
    /// When this struct was created, not when token was created.
    struct_created: std::time::Instant,
    client_id: ClientId,
    client_secret: ClientSecret,
    // FIXME: This should be removed
    login: Option<String>,
    scopes: Vec<Scope>,
}

impl std::fmt::Debug for AppAccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppAccessToken")
            .field("access_token", &self.access_token)
            .field("refresh_token", &self.refresh_token)
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret)
            .field("expires_in", &self.expires_in())
            .field("scopes", &self.scopes)
            .finish()
    }
}

#[async_trait::async_trait]
impl TwitchToken for AppAccessToken {
    fn token_type() -> super::BearerTokenType { super::BearerTokenType::AppAccessToken }

    fn client_id(&self) -> &ClientId { &self.client_id }

    fn token(&self) -> &AccessToken { &self.access_token }

    fn login(&self) -> Option<&str> { self.login.as_deref() }

    fn user_id(&self) -> Option<&str> { None }

    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        let (access_token, expires_in, refresh_token) = if let Some(token) =
            self.refresh_token.take()
        {
            crate::refresh_token(http_client, &token, &self.client_id, &self.client_secret).await?
        } else {
            return Err(RefreshTokenError::NoRefreshToken);
        };
        self.access_token = access_token;
        self.expires_in = expires_in;
        self.refresh_token = refresh_token;
        Ok(())
    }

    fn expires_in(&self) -> std::time::Duration {
        self.expires_in
            .checked_sub(self.struct_created.elapsed())
            .unwrap_or_default()
    }

    fn scopes(&self) -> &[Scope] { self.scopes.as_slice() }
}

impl AppAccessToken {
    /// Assemble token without checks.
    ///
    /// If `expires_in` is `None`, we'll assume `token.is_elapsed() == true`
    pub fn from_existing_unchecked(
        access_token: AccessToken,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_id: impl Into<ClientId>,
        client_secret: impl Into<ClientSecret>,
        // FIXME: Remove?
        login: Option<String>,
        scopes: Option<Vec<Scope>>,
        expires_in: Option<std::time::Duration>,
    ) -> AppAccessToken {
        AppAccessToken {
            access_token,
            refresh_token: refresh_token.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            login,
            expires_in: expires_in.unwrap_or_default(),
            struct_created: std::time::Instant::now(),
            scopes: scopes.unwrap_or_default(),
        }
    }

    /// Assemble token and validate it. Retrieves [`client_id`](TwitchToken::client_id) and [`scopes`](TwitchToken::scopes).
    pub async fn from_existing<'a, RE, C>(
        http_client: &'a C,
        access_token: AccessToken,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_secret: ClientSecret,
    ) -> Result<AppAccessToken, ValidationError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        let token = access_token;
        let validated = crate::validate_token(http_client, &token).await?;
        Ok(Self::from_existing_unchecked(
            token,
            refresh_token.into(),
            validated.client_id,
            client_secret,
            None,
            validated.scopes,
            Some(validated.expires_in),
        ))
    }

    /// Generate app access token via [OAuth client credentials flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-client-credentials-flow)
    pub async fn get_app_access_token<'a, C>(
        http_client: &'a C,
        client_id: ClientId,
        client_secret: ClientSecret,
        scopes: Vec<Scope>,
    ) -> Result<AppAccessToken, AppAccessTokenError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        // FIXME: self.client.exchange_code(code) does not work as oauth2 currently only sends it in body as per spec, but twitch uses query params.
        use http::{HeaderMap, Method, StatusCode};
        use std::collections::HashMap;
        let scope: String = scopes
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let mut params = HashMap::new();
        params.insert("client_id", client_id.as_str());
        params.insert("client_secret", client_secret.secret());
        params.insert("grant_type", "client_credentials");
        params.insert("scope", &scope);

        let req = crate::construct_request(
            &crate::TOKEN_URL,
            &params,
            HeaderMap::new(),
            Method::POST,
            vec![],
        );

        let resp = http_client
            .req(req)
            .await
            .map_err(AppAccessTokenError::Request)?;
        match resp.status() {
            StatusCode::OK => (),
            c if c == StatusCode::BAD_REQUEST || c == StatusCode::FORBIDDEN => {
                return Err(AppAccessTokenError::TwitchError(serde_json::from_slice(
                    resp.body(),
                )?));
            }
            c => {
                return Err(AppAccessTokenError::TwitchError(TwitchTokenErrorResponse {
                    status: c,
                    // This is not returned as I'm unsure what could be contained
                    message: "<censored>".to_string(),
                }));
            }
        };
        let response: crate::id::TwitchTokenResponse = serde_json::from_slice(resp.body())?;
        let expires_in = response.expires_in();
        let app_access = AppAccessToken::from_existing_unchecked(
            response.access_token,
            response.refresh_token,
            client_id,
            client_secret,
            None,
            response.scopes,
            expires_in,
        );

        Ok(app_access)
    }
}
