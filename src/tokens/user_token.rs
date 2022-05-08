use twitch_types::{UserId, UserName, UserIdRef, UserNameRef};

use crate::client::Client;
use crate::tokens::{
    errors::{RefreshTokenError, UserTokenExchangeError, ValidationError},
    Scope, TwitchToken,
};
use crate::ClientSecret;

use super::errors::ImplicitUserTokenExchangeError;
use crate::types::{AccessToken, ClientId, RefreshToken};

/// An User Token from the [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-implicit-code-flow) or [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-authorization-code-flow)
///
/// Used for requests that need an authenticated user. See also [`AppAccessToken`](super::AppAccessToken)
///
/// See [`UserToken::builder`](UserTokenBuilder::new) for authenticating the user using the `OAuth authorization code flow`.
#[derive(Clone)]
pub struct UserToken {
    /// The access token used to authenticate requests with
    pub access_token: AccessToken,
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    /// Username of user associated with this token
    pub login: UserName,
    /// User ID of the user associated with this token
    pub user_id: UserId,
    /// The refresh token used to extend the life of this user token
    pub refresh_token: Option<RefreshToken>,
    /// Expiration from when the response was generated.
    expires_in: std::time::Duration,
    /// When this struct was created, not when token was created.
    struct_created: std::time::Instant,
    scopes: Vec<Scope>,
    /// Token will never expire
    ///
    /// This is only true for old client IDs, like <https://twitchapps.com/tmi> and others
    pub never_expiring: bool,
}

impl std::fmt::Debug for UserToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserToken")
            .field("access_token", &self.access_token)
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret)
            .field("login", &self.login)
            .field("user_id", &self.user_id)
            .field("refresh_token", &self.refresh_token)
            .field("expires_in", &self.expires_in())
            .field("scopes", &self.scopes)
            .finish()
    }
}

impl UserToken {
    /// Assemble token without checks.
    ///
    /// If `expires_in` is `None`, we'll assume `token.is_elapsed` is always false
    #[allow(clippy::too_many_arguments)]
    pub fn from_existing_unchecked(
        access_token: impl Into<AccessToken>,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_id: impl Into<ClientId>,
        client_secret: impl Into<Option<ClientSecret>>,
        login: UserName,
        user_id: UserId,
        scopes: Option<Vec<Scope>>,
        expires_in: Option<std::time::Duration>,
    ) -> UserToken {
        UserToken {
            access_token: access_token.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            login,
            user_id,
            refresh_token: refresh_token.into(),
            expires_in: expires_in.unwrap_or_else(|| {
                // TODO: Use Duration::MAX
                std::time::Duration::new(u64::MAX, 1_000_000_000 - 1)
            }),
            struct_created: std::time::Instant::now(),
            scopes: scopes.unwrap_or_default(),
            never_expiring: expires_in.is_none(),
        }
    }

    /// Assemble token and validate it. Retrieves [`login`](TwitchToken::login), [`client_id`](TwitchToken::client_id) and [`scopes`](TwitchToken::scopes)
    ///
    /// If the token is already expired, this function will fail to produce a [`UserToken`] and return [`ValidationError::NotAuthorized`]
    pub async fn from_existing<'a, C>(
        http_client: &'a C,
        access_token: AccessToken,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_secret: impl Into<Option<ClientSecret>>,
    ) -> Result<UserToken, ValidationError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        let validated = crate::validate_token(http_client, &access_token).await?;
        Ok(Self::from_existing_unchecked(
            access_token,
            refresh_token.into(),
            validated.client_id,
            client_secret,
            validated.login.ok_or(ValidationError::NoLogin)?,
            validated.user_id.ok_or(ValidationError::NoLogin)?,
            validated.scopes,
            Some(validated.expires_in).filter(|d| {
                // FIXME: https://github.com/rust-lang/rust/pull/84084
                // FIXME: nanos are not returned
                // if duration is zero, the token will never expire. if the token was expired, twitch would return NotAuthorized
                // TODO: There could be a situation where this fails, if the token is just about to expire, say 500ms, does twitch round up to 1 or down to 0?
                !(d.as_secs() == 0 && d.as_nanos() == 0)
            }),
        ))
    }

    #[doc(hidden)]
    /// Returns true if this token is never expiring.
    ///
    /// Hidden because it's not expected to be used.
    pub fn never_expires(&self) -> bool { self.never_expiring }

    /// Create a [`UserTokenBuilder`] to get a token with the [OAuth Authorization Code](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
    pub fn builder(
        client_id: ClientId,
        client_secret: ClientSecret,
        // FIXME: Braid or string or this?
        redirect_url: url::Url,
    ) -> UserTokenBuilder {
        UserTokenBuilder::new(client_id, client_secret, redirect_url)
    }

    /// Generate a user token from [mock-api](https://github.com/twitchdev/twitch-cli/blob/main/docs/mock-api.md#auth-namespace)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[tokio::main]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error + 'static>>{
    /// let token = twitch_oauth2::UserToken::mock_token(
    ///     &reqwest::Client::builder()
    ///         .redirect(reqwest::redirect::Policy::none())
    ///         .build()?,
    ///     "mockclientid".into(),
    ///     "mockclientsecret".into(),
    ///     "user_id",
    ///     vec![],
    ///     ).await?;
    /// # Ok(())}
    /// # fn main() {run();}
    /// ```
    #[cfg_attr(nightly, doc(cfg(feature = "mock_api")))]
    #[cfg(feature = "mock_api")]
    pub async fn mock_token<'a, C>(
        http_client: &'a C,
        client_id: ClientId,
        client_secret: ClientSecret,
        user_id: impl AsRef<UserIdRef>,
        scopes: Vec<Scope>,
    ) -> Result<UserToken, UserTokenExchangeError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        use http::{HeaderMap, Method};
        use std::collections::HashMap;

        let user_id = user_id.as_ref();
        let scope_str = scopes.as_slice().join(" ");
        let mut params = HashMap::new();
        params.insert("client_id", client_id.as_str());
        params.insert("client_secret", client_secret.secret());
        params.insert("grant_type", "user_token");
        params.insert("scope", &scope_str);
        params.insert("user_id", user_id);

        let req = crate::construct_request(
            &crate::AUTH_URL,
            &params,
            HeaderMap::new(),
            Method::POST,
            vec![],
        );

        let resp = http_client
            .req(req)
            .await
            .map_err(UserTokenExchangeError::RequestError)?;
        let response: crate::id::TwitchTokenResponse = crate::parse_response(&resp)?;

        UserToken::from_existing(
            http_client,
            response.access_token,
            response.refresh_token,
            client_secret,
        )
        .await
        .map_err(Into::into)
    }

    /// Set the client secret
    pub fn set_secret(&mut self, secret: Option<ClientSecret>) { self.client_secret = secret }
}

#[async_trait::async_trait]
impl TwitchToken for UserToken {
    fn token_type() -> super::BearerTokenType { super::BearerTokenType::UserToken }

    fn client_id(&self) -> &ClientId { &self.client_id }

    fn token(&self) -> &AccessToken { &self.access_token }

    fn login(&self) -> Option<&UserNameRef> { Some(&self.login) }

    fn user_id(&self) -> Option<&UserIdRef> { Some(&self.user_id) }

    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client<'a>>::Error>>
    where
        Self: Sized,
        C: Client<'a>,
    {
        if let Some(client_secret) = self.client_secret.clone() {
            let (access_token, expires, refresh_token) = if let Some(token) =
                self.refresh_token.take()
            {
                crate::refresh_token(http_client, &token, &self.client_id, &client_secret).await?
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

    fn expires_in(&self) -> std::time::Duration {
        if !self.never_expiring {
            self.expires_in
                .checked_sub(self.struct_created.elapsed())
                .unwrap_or_default()
        } else {
            // We don't return an option here because it's not expected to use this if the token is known to be unexpiring.
            // TODO: Use Duration::MAX
            std::time::Duration::new(u64::MAX, 1_000_000_000 - 1)
        }
    }

    fn scopes(&self) -> &[Scope] { self.scopes.as_slice() }
}

/// Builder for [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
///
/// See [`ImplicitUserTokenBuilder`] for the [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-implicit-code-flow) (does not require Client Secret)
pub struct UserTokenBuilder {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) csrf: Option<crate::types::CsrfToken>,
    pub(crate) force_verify: bool,
    pub(crate) redirect_url: url::Url,
    client_id: ClientId,
    client_secret: ClientSecret,
}

impl UserTokenBuilder {
    /// Create a [`UserTokenBuilder`]
    ///
    /// # Notes
    ///
    /// The `url` crate converts empty paths into "/" (such as `https://example.com` into `https://example.com/`),
    /// which means that you'll need to add `https://example.com/` to your redirect URIs (note the "trailing" slash) if you want to use an empty path.
    ///
    /// To avoid this, use a path such as `https://example.com/twitch/register` or similar instead, where the `url` crate would not add a trailing `/`.
    pub fn new(
        client_id: ClientId,
        client_secret: ClientSecret,
        redirect_url: url::Url,
    ) -> UserTokenBuilder {
        UserTokenBuilder {
            scopes: vec![],
            csrf: None,
            force_verify: false,
            redirect_url,
            client_id,
            client_secret,
        }
    }

    /// Add scopes to the request
    pub fn set_scopes(mut self, scopes: Vec<Scope>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Add a single scope to request
    pub fn add_scope(&mut self, scope: Scope) { self.scopes.push(scope); }

    /// Enable or disable function to make the user able to switch accounts if needed.
    pub fn force_verify(mut self, b: bool) -> Self {
        self.force_verify = b;
        self
    }

    /// Generate the URL to request a code.
    ///
    /// Step 1. in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
    pub fn generate_url(&mut self) -> (url::Url, crate::types::CsrfToken) {
        let csrf = crate::types::CsrfToken::new_random();
        self.csrf = Some(csrf.clone());
        let mut url = crate::AUTH_URL.clone();

        let auth = vec![
            ("response_type", "code"),
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_url.as_str()),
            ("state", csrf.as_str()),
        ];

        url.query_pairs_mut().extend_pairs(auth);

        if !self.scopes.is_empty() {
            url.query_pairs_mut()
                .append_pair("scope", &self.scopes.as_slice().join(" "));
        }

        if self.force_verify {
            url.query_pairs_mut().append_pair("force_verify", "true");
        };

        (url, csrf)
    }

    /// Set the CSRF token.
    ///
    /// Hidden because you should preferably not use this.
    #[doc(hidden)]
    pub fn set_csrf(&mut self, csrf: crate::types::CsrfToken) { self.csrf = Some(csrf); }

    /// Generate the code with the help of the authorization code
    ///
    /// Step 3. and 4. in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow)
    ///
    /// On failure to authenticate due to wrong redirect url or other errors, twitch redirects the user to `<redirect_url or first defined url in dev console>?error=<error type>&error_description=<description of error>`
    pub async fn get_user_token<'a, C>(
        self,
        http_client: &'a C,
        state: &str,
        // TODO: Should be either str or AuthorizationCode
        code: &str,
    ) -> Result<UserToken, UserTokenExchangeError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        if let Some(csrf) = self.csrf {
            if csrf.secret() != state {
                return Err(UserTokenExchangeError::StateMismatch);
            }
        } else {
            return Err(UserTokenExchangeError::StateMismatch);
        }

        // FIXME: self.client.exchange_code(code) does not work as oauth2 currently only sends it in body as per spec, but twitch uses query params.
        use http::{HeaderMap, Method};
        use std::collections::HashMap;
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.secret());
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", self.redirect_url.as_str());

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
            .map_err(UserTokenExchangeError::RequestError)?;

        let response: crate::id::TwitchTokenResponse = crate::parse_response(&resp)?;
        UserToken::from_existing(
            http_client,
            response.access_token,
            response.refresh_token,
            self.client_secret,
        )
        .await
        .map_err(Into::into)
    }
}

/// Builder for [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-implicit-code-flow)
///
/// See [`UserTokenBuilder`] for the [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow) (requires Client Secret, generally more secure)
pub struct ImplicitUserTokenBuilder {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) csrf: Option<crate::types::CsrfToken>,
    pub(crate) redirect_url: url::Url,
    pub(crate) force_verify: bool,
    client_id: ClientId,
}

impl ImplicitUserTokenBuilder {
    /// Create a [`ImplicitUserTokenBuilder`]
    ///
    /// # Notes
    ///
    /// The `url` crate converts empty paths into "/" (such as `https://example.com` into `https://example.com/`),
    /// which means that you'll need to add `https://example.com/` to your redirect URIs (note the "trailing" slash) if you want to use an empty path.
    ///
    /// To avoid this, use a path such as `https://example.com/twitch/register` or similar instead, where the `url` crate would not add a trailing `/`.
    pub fn new(client_id: ClientId, redirect_url: url::Url) -> ImplicitUserTokenBuilder {
        ImplicitUserTokenBuilder {
            scopes: vec![],
            redirect_url,
            csrf: None,
            force_verify: false,
            client_id,
        }
    }

    /// Add scopes to the request
    pub fn set_scopes(mut self, scopes: Vec<Scope>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Add a single scope to request
    pub fn add_scope(&mut self, scope: Scope) { self.scopes.push(scope); }

    /// Enable or disable function to make the user able to switch accounts if needed.
    pub fn force_verify(mut self, b: bool) -> Self {
        self.force_verify = b;
        self
    }

    /// Generate the URL to request a token.
    ///
    /// Step 1. in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#auth-implicit-code-flow)
    pub fn generate_url(&mut self) -> (url::Url, crate::types::CsrfToken) {
        let csrf = crate::types::CsrfToken::new_random();
        self.csrf = Some(csrf.clone());
        let mut url = crate::AUTH_URL.clone();

        let auth = vec![
            ("response_type", "token"),
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_url.as_str()),
            ("state", csrf.as_str()),
        ];

        url.query_pairs_mut().extend_pairs(auth);

        if !self.scopes.is_empty() {
            url.query_pairs_mut()
                .append_pair("scope", &self.scopes.as_slice().join(" "));
        }

        if self.force_verify {
            url.query_pairs_mut().append_pair("force_verify", "true");
        };

        (url, csrf)
    }

    /// Generate the code with the help of the hash.
    ///
    /// You can skip this method and instead use the token in the hash directly with [`UserToken::from_existing()`], but it's provided here for convenience.
    ///
    /// Step 3. and 4. in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-implicit-code-flow)
    ///
    /// # Example
    ///
    /// When the user authenticates, they are sent to `<redirecturl>#access_token=<access_token>&scope=<scopes, space (%20) separated>&state=<csrf state>&token_type=bearer`
    ///
    /// On failure, they are sent to
    ///
    /// `<redirect_url or first defined url in dev console>?error=<error type>&error_description=<error description>&state=<csrf state>`
    /// Get the hash of the url with javascript.
    ///
    /// ```js
    /// document.location.hash.substr(1);
    /// ```
    ///
    /// and send it to your client in what ever way convenient.
    ///
    /// Provided below is an example of how to do it, no guarantees on the safety of this method.
    ///
    /// ```html
    /// <!DOCTYPE html>
    /// <html>
    /// <head>
    /// <title>Authorization</title>
    /// <meta name="ROBOTS" content="NOFOLLOW">
    /// <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    /// <script type="text/javascript">
    /// <!--
    /// function initiate() {
    ///     var hash = document.location.hash.substr(1);
    ///     document.getElementById("javascript").className = "";
    ///     if (hash != null) {
    ///             document.location.replace("/token?"+hash);
    ///     }
    ///     else {
    ///         document.getElementById("javascript").innerHTML = "Error: Access Token not found";
    ///     }
    /// }
    /// -->
    /// </script>
    /// <style type="text/css">
    ///     body { text-align: center; background-color: #FFF; max-width: 500px; margin: auto; }
    ///     noscript { color: red;  }
    ///     .hide { display: none; }
    /// </style>
    /// </head>
    /// <body onload="initiate()">
    /// <h1>Authorization</h1>
    /// <noscript>
    ///     <p>This page requires <strong>JavaScript</strong> to get your token.
    /// </noscript>
    /// <p id="javascript" class="hide">
    /// You should be redirected..
    /// </p>
    /// </body>
    /// </html>
    /// ```
    ///
    /// where `/token?` gives this function it's corresponding arguments in query params
    ///
    /// Make sure that `/token` removes the query from the history.
    ///
    /// ```html
    /// <!DOCTYPE html>
    /// <html>
    /// <head>
    /// <title>Authorization Successful</title>
    /// <meta name="ROBOTS" content="NOFOLLOW">
    /// <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    /// <script type="text/javascript">
    /// <!--
    /// function initiate() {
    ///     //
    ///     document.location.replace("/token_retrieved);
    /// }
    /// -->
    /// </script>
    /// <style type="text/css">
    ///     body { text-align: center; background-color: #FFF; max-width: 500px; margin: auto; }
    /// </style>
    /// </head>
    /// <body onload="initiate()">
    /// <h1>Authorization Successful</h1>
    /// </body>
    /// </html>
    /// ```
    ///
    ///
    pub async fn get_user_token<'a, C>(
        self,
        http_client: &'a C,
        state: Option<&str>,
        access_token: Option<&str>,
        error: Option<&str>,
        error_description: Option<&str>,
    ) -> Result<UserToken, ImplicitUserTokenExchangeError<<C as Client<'a>>::Error>>
    where
        C: Client<'a>,
    {
        if let Some(csrf) = self.csrf {
            if csrf.secret() != state.unwrap_or("") {
                return Err(ImplicitUserTokenExchangeError::StateMismatch);
            }
        } else {
            return Err(ImplicitUserTokenExchangeError::StateMismatch);
        }
        match (access_token, error, error_description) {
            (Some(access_token), None, None) => UserToken::from_existing(
                http_client,
                crate::types::AccessToken::new(access_token.to_string()),
                None,
                None,
            )
            .await
            .map_err(Into::into),
            (_, error, description) => {
                let (error, description) = (
                    error.map(|s| s.to_string()),
                    description.map(|s| s.to_string()),
                );
                Err(ImplicitUserTokenExchangeError::TwitchError { error, description })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    #[test]
    fn generate_url() {
        dbg!(UserTokenBuilder::new(
            ClientId::new("random_client"),
            ClientSecret::new("random_secret"),
            url::Url::parse("https://localhost").unwrap(),
        )
        .force_verify(true)
        .generate_url()
        .0
        .to_string());
    }

    #[tokio::test]
    #[ignore]
    async fn get_token() {
        let mut t = UserTokenBuilder::new(
            ClientId::new(
                std::env::var("TWITCH_CLIENT_ID").expect("no env:TWITCH_CLIENT_ID provided"),
            ),
            ClientSecret::new(
                std::env::var("TWITCH_CLIENT_SECRET")
                    .expect("no env:TWITCH_CLIENT_SECRET provided"),
            ),
            url::Url::parse(r#"https://localhost"#).unwrap(),
        )
        .force_verify(true);
        t.csrf = Some(crate::CsrfToken::new("random"));
        let token = t
            .get_user_token(&surf::Client::new(), "random", "authcode")
            .await
            .unwrap();
        println!("token: {:?} - {}", token, token.access_token.secret());
    }

    #[tokio::test]
    #[ignore]
    async fn get_implicit_token() {
        let mut t = ImplicitUserTokenBuilder::new(
            ClientId::new(
                std::env::var("TWITCH_CLIENT_ID").expect("no env:TWITCH_CLIENT_ID provided"),
            ),
            url::Url::parse(r#"http://localhost/twitch/register"#).unwrap(),
        )
        .force_verify(true);
        println!("{}", t.generate_url().0);
        t.csrf = Some(crate::CsrfToken::new("random"));
        let token = t
            .get_user_token(
                &surf::Client::new(),
                Some("random"),
                Some("authcode"),
                None,
                None,
            )
            .await
            .unwrap();
        println!("token: {:?} - {}", token, token.access_token.secret());
    }
}
