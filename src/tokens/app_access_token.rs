use super::errors::{TokenError, ValidationError};
use crate::{
    id::TwitchClient,
    tokens::{errors::RefreshTokenError, Scope, TwitchToken},
};
use oauth2::{AccessToken, AuthUrl, ClientId, ClientSecret, RefreshToken, TokenResponse};
use oauth2::{HttpRequest, HttpResponse};
use std::future::Future;

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
        f.debug_struct("UserToken")
            .field("access_token", &self.access_token)
            .field("refresh_token", &self.refresh_token)
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret)
            .field("expires_in", &self.expires_in())
            .field("scopes", &self.scopes)
            .finish()
    }
}

#[async_trait::async_trait(?Send)]
impl TwitchToken for AppAccessToken {
    fn token_type() -> super::BearerTokenType { super::BearerTokenType::AppAccessToken }

    fn client_id(&self) -> &ClientId { &self.client_id }

    fn token(&self) -> &AccessToken { &self.access_token }

    fn login(&self) -> Option<&str> { self.login.as_deref() }

    async fn refresh_token<RE, C, F>(
        &mut self,
        http_client: C,
    ) -> Result<(), RefreshTokenError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        let (access_token, expires_in, refresh_token) = if let Some(token) =
            self.refresh_token.take()
        {
            crate::refresh_token(http_client, token, &self.client_id, &self.client_secret).await?
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
    pub async fn from_existing<RE, C, F>(
        http_client: C,
        access_token: AccessToken,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_secret: ClientSecret,
    ) -> Result<AppAccessToken, ValidationError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
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
    pub async fn get_app_access_token<RE, C, F>(
        http_client: C,
        client_id: ClientId,
        client_secret: ClientSecret,
        scopes: Vec<Scope>,
    ) -> Result<AppAccessToken, TokenError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: Fn(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        let client = TwitchClient::new(
            client_id.clone(),
            Some(client_secret.clone()),
            AuthUrl::new(crate::AUTH_URL.to_owned())
                .expect("unexpected failure to parse auth url for app_access_token"),
            Some(oauth2::TokenUrl::new(crate::TOKEN_URL.to_string())?),
        );
        let client = client.set_auth_type(oauth2::AuthType::RequestBody);
        let mut client = client.exchange_client_credentials();
        for scope in scopes.clone() {
            client = client.add_scope(scope.as_oauth_scope());
        }
        let response = client
            .request_async(&http_client)
            .await
            .map_err(TokenError::Request)?;

        let app_access = AppAccessToken::from_existing_unchecked(
            response.access_token().clone(),
            response.refresh_token().cloned(),
            client_id,
            client_secret,
            None,
            Some(
                response
                    .scopes()
                    .cloned()
                    .map(|s| s.into_iter().map(|s| s.into()).collect())
                    .unwrap_or(scopes),
            ),
            response.expires_in(),
        );

        let _ = app_access.validate_token(http_client).await?; // Sanity check
        Ok(app_access)
    }
}
