use crate::ClientSecret;
use crate::{
    id::TwitchTokenErrorResponse,
    tokens::{
        errors::{RefreshTokenError, UserTokenExchangeError, ValidationError},
        Scope, TwitchToken,
    },
};

use oauth2::{AccessToken, ClientId, RedirectUrl, RefreshToken};
use oauth2::{HttpRequest, HttpResponse};
use std::future::Future;

/// An User Token from the [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-implicit-code-flow) or [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-authorization-code-flow)
#[derive(Debug, Clone)]
pub struct UserToken {
    /// The access token used to authenticate requests with
    pub access_token: AccessToken,
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    /// Username of user associated with this token
    pub login: String,
    /// The refresh token used to extend the life of this user token
    pub refresh_token: Option<RefreshToken>,
    /// Expiration from when the response was generated.
    expires_in: Option<std::time::Duration>,
    /// When this struct was created, not when token was created.
    struct_created: std::time::Instant,
    scopes: Vec<Scope>,
}

impl UserToken {
    /// Assemble token without checks.
    pub fn from_existing_unchecked(
        access_token: impl Into<AccessToken>,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_id: impl Into<ClientId>,
        client_secret: impl Into<Option<ClientSecret>>,
        login: String,
        scopes: Option<Vec<Scope>>,
        expires_in: Option<std::time::Duration>,
    ) -> UserToken {
        UserToken {
            access_token: access_token.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            login,
            refresh_token: refresh_token.into(),
            expires_in,
            struct_created: std::time::Instant::now(),
            scopes: scopes.unwrap_or_else(Vec::new),
        }
    }

    /// Assemble token and validate it. Retrieves [`login`](TwitchToken::login), [`client_id`](TwitchToken::client_id) and [`scopes`](TwitchToken::scopes)
    pub async fn from_existing<RE, C, F>(
        http_client: C,
        access_token: AccessToken,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_secret: impl Into<Option<ClientSecret>>,
    ) -> Result<UserToken, ValidationError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        let validated = crate::validate_token(http_client, &access_token).await?;
        Ok(Self::from_existing_unchecked(
            access_token,
            refresh_token.into(),
            validated.client_id,
            client_secret,
            validated.login.ok_or(ValidationError::NoLogin)?,
            validated.scopes,
            validated.expires_in,
        ))
    }

    /// Create a [`UserTokenBuilder`] to get a token with the [OAuth Authorization Code](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
    pub fn builder(
        client_id: ClientId,
        client_secret: ClientSecret,
        redirect_url: RedirectUrl,
    ) -> Result<UserTokenBuilder, oauth2::url::ParseError> {
        UserTokenBuilder::new(client_id, client_secret, redirect_url)
    }
}

#[async_trait::async_trait(?Send)]
impl TwitchToken for UserToken {
    fn client_id(&self) -> &ClientId { &self.client_id }

    fn token(&self) -> &AccessToken { &self.access_token }

    fn login(&self) -> Option<&str> { Some(&self.login) }

    async fn refresh_token<RE, C, F>(
        &mut self,
        http_client: C,
    ) -> Result<(), RefreshTokenError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        if let Some(client_secret) = self.client_secret.clone() {
            let (access_token, expires, refresh_token) = if let Some(token) =
                self.refresh_token.take()
            {
                crate::refresh_token(http_client, token, &self.client_id, &client_secret).await?
            } else {
                return Err(RefreshTokenError::NoRefreshToken);
            };
            self.access_token = access_token;
            self.expires_in = expires;
            self.refresh_token = refresh_token;
            Ok(())
        } else {
            return Err(RefreshTokenError::NoClientSecretFound);
        }
    }

    fn expires_in(&self) -> Option<std::time::Duration> {
        self.expires_in.map(|e| e - self.struct_created.elapsed())
    }

    fn scopes(&self) -> Option<&[Scope]> { Some(self.scopes.as_slice()) }
}

/// Builder for [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
pub struct UserTokenBuilder {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) client: crate::TwitchClient,
    pub(crate) csrf: Option<oauth2::CsrfToken>,
    pub(crate) force_verify: bool,
    pub(crate) redirect_url: RedirectUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
}

impl UserTokenBuilder {
    /// Create a [`UserTokenBuilder`]
    pub fn new(
        client_id: ClientId,
        client_secret: ClientSecret,
        redirect_url: RedirectUrl,
    ) -> Result<UserTokenBuilder, oauth2::url::ParseError> {
        Ok(UserTokenBuilder {
            scopes: vec![],
            client: crate::TwitchClient::new(
                client_id.clone(),
                Some(client_secret.clone()),
                oauth2::AuthUrl::new(crate::AUTH_URL.to_string())?,
                Some(oauth2::TokenUrl::new(crate::TOKEN_URL.to_string())?),
            )
            .set_auth_type(oauth2::AuthType::BasicAuth)
            .set_redirect_uri(redirect_url.clone()),
            csrf: None,
            force_verify: false,
            redirect_url,
            client_id,
            client_secret,
        })
    }

    /// Add scopes to the request
    pub fn set_scopes(mut self, scopes: Vec<Scope>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Add a single scope to request
    pub fn add_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    /// Enable or disable function to make the user able to switch accounts if needed.
    pub fn force_verify(mut self, b: bool) -> Self {
        self.force_verify = b;
        self
    }

    /// Generate the URL to request a code.
    ///
    /// Step 1. in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
    pub fn generate_url(&mut self) -> (oauth2::url::Url, oauth2::CsrfToken) {
        let mut auth = self.client.authorize_url(oauth2::CsrfToken::new_random);

        for scope in self.scopes.iter() {
            auth = auth.add_scope(scope.as_oauth_scope())
        }

        auth = auth.add_extra_param(
            "force_verify",
            if self.force_verify { "true" } else { "false" },
        );

        let (url, csrf) = auth.url();
        self.csrf = Some(csrf.clone());
        (url, csrf)
    }

    /// Set the CSRF token.
    ///
    /// Hidden because you should preferably not use this.
    #[doc(hidden)]
    pub fn set_csrf(&mut self, csrf: oauth2::CsrfToken) { self.csrf = Some(csrf); }

    /// Generate the code with the help of the authorization code
    ///
    /// Step. 3 and 4 in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
    pub async fn get_user_token<RE, C, F>(
        self,
        http_client: C,
        state: &str,
        // TODO: Should be either str or AuthorizationCode
        code: &str,
    ) -> Result<UserToken, UserTokenExchangeError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: Copy + FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        if let Some(csrf) = self.csrf {
            if csrf.secret() != state {
                return Err(UserTokenExchangeError::StateMismatch);
            }
        } else {
            return Err(UserTokenExchangeError::StateMismatch);
        }

        // FIXME: self.client.exchange_code(code) does not work as oauth2 currently only sends it in body as per spec, but twitch uses query params.
        use oauth2::http::{HeaderMap, Method, StatusCode};
        use std::collections::HashMap;
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.secret().as_str());
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", self.redirect_url.as_str());
        let req = HttpRequest {
            url: oauth2::url::Url::parse_with_params(crate::TOKEN_URL, &params)
                .expect("unexpectedly failed to parse revoke url"),
            method: Method::POST,
            headers: HeaderMap::new(),
            body: vec![],
        };

        let resp = http_client(req)
            .await
            .map_err(UserTokenExchangeError::RequestError)?;
        match resp.status_code {
            StatusCode::BAD_REQUEST => {
                return Err(UserTokenExchangeError::TwitchError(
                    TwitchTokenErrorResponse {
                        status: StatusCode::BAD_REQUEST,
                        message: String::from_utf8_lossy(&resp.body).into_owned(),
                    },
                ))
            }
            StatusCode::OK => (),
            _ => todo!(),
        };
        let response: crate::id::TwitchTokenResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        > = serde_json::from_slice(resp.body.as_slice())?;
        UserToken::from_existing(
            http_client,
            response.access_token,
            response.refresh_token,
            None,
        )
        .await
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    #[test]
    fn generate_url() {
        dbg!(UserTokenBuilder::new(
            ClientId::new("clientid".to_string()),
            ClientSecret::new("secret".to_string()),
            oauth2::RedirectUrl::new("https://localhost".to_string()).unwrap(),
        )
        .unwrap()
        .force_verify(true)
        .generate_url()
        .0
        .to_string());
    }

    #[tokio::test]
    #[ignore]
    async fn get_token() {
        let mut t = UserTokenBuilder::new(
            ClientId::new("clientid".to_string()),
            ClientSecret::new("secret".to_string()),
            crate::RedirectUrl::new(r#"https://localhost"#.to_string()).unwrap(),
        )
        .unwrap()
        .force_verify(true);
        t.csrf = Some(oauth2::CsrfToken::new("random".to_string()));
        let token = t
            .get_user_token(crate::client::surf_http_client, "random", "authcode")
            .await
            .unwrap();
        println!("token: {:?} - {}", token, token.access_token.secret());
    }
}
