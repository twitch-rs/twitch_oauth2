use twitch_types::{UserId, UserIdRef, UserName, UserNameRef};

use super::errors::ValidationError;
#[cfg(feature = "client")]
use super::errors::{ImplicitUserTokenExchangeError, RefreshTokenError, UserTokenExchangeError};
#[cfg(feature = "client")]
use crate::client::Client;

use crate::tokens::{Scope, TwitchToken};
use crate::{ClientSecret, ValidatedToken};

use crate::types::{AccessToken, ClientId, RefreshToken};

/// An User Token from the [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#implicit-grant-flow) or [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow)
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
    /// Create a new token
    ///
    /// See [`UserToken::from_token`] and [`UserToken::from_existing`] for more ways to create a [`UserToken`]
    pub fn new(
        access_token: AccessToken,
        refresh_token: Option<RefreshToken>,
        validated: ValidatedToken,
        client_secret: impl Into<Option<ClientSecret>>,
    ) -> Result<UserToken, ValidationError<std::convert::Infallible>> {
        Ok(UserToken::from_existing_unchecked(
            access_token,
            refresh_token,
            validated.client_id,
            client_secret,
            validated.login.ok_or(ValidationError::InvalidToken(
                "validation did not include a `login`, token might be an app access token",
            ))?,
            validated.user_id.ok_or(ValidationError::InvalidToken(
                "validation did not include a `user_id`, token might be an app access token",
            ))?,
            validated.scopes,
            validated.expires_in,
        ))
    }

    /// Create a [UserToken] from an existing active user token. Retrieves [`login`](TwitchToken::login), [`client_id`](TwitchToken::client_id) and [`scopes`](TwitchToken::scopes)
    ///
    /// If the token is already expired, this function will fail to produce a [`UserToken`] and return [`ValidationError::NotAuthorized`]
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use twitch_oauth2::{AccessToken, UserToken};
    /// // Make sure you enable the feature "reqwest" for twitch_oauth2 if you want to use reqwest
    /// # async {let client = twitch_oauth2::client::DummyClient; stringify!(
    /// let client = reqwest::Client::builder()
    ///     .redirect(reqwest::redirect::Policy::none())
    ///     .build()?;
    /// # );
    /// let token = UserToken::from_token(&client, AccessToken::from("my_access_token")).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())};
    /// ```
    #[cfg(feature = "client")]
    pub async fn from_token<C>(
        http_client: &C,
        access_token: AccessToken,
    ) -> Result<UserToken, ValidationError<<C as Client>::Error>>
    where
        C: Client,
    {
        Self::from_existing(http_client, access_token, None, None).await
    }

    /// Create a [UserToken] from an existing active user token. Retrieves [`login`](TwitchToken::login), [`client_id`](TwitchToken::client_id) and [`scopes`](TwitchToken::scopes)
    ///
    /// If the token is already expired, this function will fail to produce a [`UserToken`] and return [`ValidationError::NotAuthorized`]
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use twitch_oauth2::{AccessToken, ClientSecret, RefreshToken, UserToken};
    /// // Make sure you enable the feature "reqwest" for twitch_oauth2 if you want to use reqwest
    /// # async {let client = twitch_oauth2::client::DummyClient; stringify!(
    /// let client = reqwest::Client::builder()
    ///     .redirect(reqwest::redirect::Policy::none())
    ///     .build()?;
    /// # );
    /// let token = UserToken::from_existing(
    ///     &client,
    ///     AccessToken::from("my_access_token"),
    ///     RefreshToken::from("my_refresh_token"),
    ///     ClientSecret::from("my_client_secret"),
    /// )
    /// .await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())};
    /// ```
    #[cfg(feature = "client")]
    pub async fn from_existing<C>(
        http_client: &C,
        access_token: AccessToken,
        refresh_token: impl Into<Option<RefreshToken>>,
        client_secret: impl Into<Option<ClientSecret>>,
    ) -> Result<UserToken, ValidationError<<C as Client>::Error>>
    where
        C: Client,
    {
        let validated = access_token.validate_token(http_client).await?;
        Self::new(access_token, refresh_token.into(), validated, client_secret)
            .map_err(|e| e.into_other())
    }

    /// Assemble token without checks.
    ///
    /// # Notes
    ///
    /// If `expires_in` is `None`, we'll assume [`token.is_elapsed`](TwitchToken::is_elapsed) is always false
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
            expires_in: expires_in.unwrap_or(std::time::Duration::MAX),
            struct_created: std::time::Instant::now(),
            scopes: scopes.unwrap_or_default(),
            never_expiring: expires_in.is_none(),
        }
    }

    /// Assemble token from twitch responses.
    pub fn from_response(
        response: crate::id::TwitchTokenResponse,
        validated: ValidatedToken,
        client_secret: impl Into<Option<ClientSecret>>,
    ) -> Result<UserToken, ValidationError<std::convert::Infallible>> {
        Self::new(
            response.access_token,
            response.refresh_token,
            validated,
            client_secret,
        )
    }

    #[doc(hidden)]
    /// Returns true if this token is never expiring.
    ///
    /// Hidden because it's not expected to be used.
    pub fn never_expires(&self) -> bool { self.never_expiring }

    /// Create a [`UserTokenBuilder`] to get a token with the [OAuth Authorization Code](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow)
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
    /// )
    /// .await?;
    /// # Ok(())}
    /// # fn main() {run();}
    /// ```
    #[cfg(all(feature = "mock_api", feature = "client"))]
    pub async fn mock_token<C>(
        http_client: &C,
        client_id: ClientId,
        client_secret: ClientSecret,
        user_id: impl AsRef<str>,
        scopes: Vec<Scope>,
    ) -> Result<UserToken, UserTokenExchangeError<<C as Client>::Error>>
    where
        C: Client,
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
        let response = crate::id::TwitchTokenResponse::from_response(&resp)?;

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

#[cfg_attr(feature = "client", async_trait::async_trait)]
impl TwitchToken for UserToken {
    fn token_type() -> super::BearerTokenType { super::BearerTokenType::UserToken }

    fn client_id(&self) -> &ClientId { &self.client_id }

    fn token(&self) -> &AccessToken { &self.access_token }

    fn login(&self) -> Option<&UserNameRef> { Some(&self.login) }

    fn user_id(&self) -> Option<&UserIdRef> { Some(&self.user_id) }

    #[cfg(feature = "client")]
    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client,
    {
        if let Some(client_secret) = self.client_secret.clone() {
            let (access_token, expires, refresh_token) =
                if let Some(token) = self.refresh_token.take() {
                    token
                        .refresh_token(http_client, &self.client_id, &client_secret)
                        .await?
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
            std::time::Duration::MAX
        }
    }

    fn scopes(&self) -> &[Scope] { self.scopes.as_slice() }
}

/// Builder for [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow)
///
/// See [`ImplicitUserTokenBuilder`] for the [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#implicit-grant-flow) (does not require Client Secret)
///
/// # Examples
///
/// See also [the auth flow example](https://github.com/twitch-rs/twitch_oauth2/blob/main/examples/auth_flow.rs)
///
/// To generate a user token with this auth flow, you need to:
///
/// 1. Initialize the [`UserTokenBuilder`] with [`UserTokenBuilder::new`](UserTokenBuilder::new), providing your client id, client secret, and a redirect URL.
/// Use [`set_scopes(vec![])`](UserTokenBuilder::set_scopes) to add any necessary scopes to the request. You can also use [`force_verify(true)`](UserTokenBuilder::force_verify) to force the user to
/// re-authorize your app’s access to their resources.
///
///     Make sure you've added the redirect URL to the app settings on [the Twitch Developer Console](https://dev.twitch.tv/console).
///
///     ```rust
///     use twitch_oauth2::{id::TwitchTokenResponse, tokens::UserTokenBuilder, Scope};
///     use url::Url;
///
///     // This is the URL the user will be redirected to after authorizing your application
///     let redirect_url = Url::parse("http://localhost/twitch/register")?;
///     let mut builder = UserTokenBuilder::new("myclientid", "myclientsecret", redirect_url);
///     builder = builder.set_scopes(vec![Scope::ChatRead, Scope::ChatEdit]);
///     builder = builder.force_verify(true); // Defaults to false
///     # Ok::<(), Box<dyn std::error::Error>>(())
///     ```
///
/// 2. Generate a URL for the user to visit using [`generate_url()`](UserTokenBuilder::generate_url). This method also returns a CSRF token that you need to save for later validation.
///
///     ```rust
///     # use twitch_oauth2::{id::TwitchTokenResponse, tokens::UserTokenBuilder, Scope};
///     # use url::Url;
///     # let redirect_url = Url::parse("http://localhost/twitch/register")?;
///     # let mut builder = UserTokenBuilder::new("myclientid", "myclientsecret", redirect_url);
///     let (url, csrf_token) = builder.generate_url();
///     // Make your user navigate to this URL, for example
///     println!("Visit this URL to authorize Twitch access: {}", url);
///     # Ok::<(), Box<dyn std::error::Error>>(())
///     ```
///
/// 3. Have the user visit the generated URL. They will be asked to authorize your application if they haven't previously done so
///    or if you've set [`force_verify`](UserTokenBuilder::force_verify) to `true`.
///
///      You can do this by providing the link in [a web page](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a), have the user [be directed](https://developer.mozilla.org/en-US/docs/Web/API/Location/assign),
///      the console, or by [opening it](https://docs.rs/webbrowser/0.8.10/webbrowser/) in a browser.
///
///      If this is a web server, you should store the [UserTokenBuilder] somewhere you can retrieve it later. A good place to store it is in a [`Cache`](https://docs.rs/retainer/0.3.0/retainer/cache/struct.Cache.html)
///      or a [`HashMap`](std::collections::HashMap) with the CSRF token as the key.
///
/// 4. When the user has been redirected to the redirect URL by twitch, extract the `state` and `code` query parameters from the URL.
///
///     ```rust
///     use std::borrow::Cow;
///     use std::collections::BTreeMap;
///
///     fn extract_pair<'a>(
///         query: &BTreeMap<Cow<'a, str>, Cow<'a, str>>,
///         key1: &str,
///         key2: &str,
///     ) -> Option<(Cow<'a, str>, Cow<'a, str>)> {
///         Some((query.get(key1)?.clone(), query.get(key2)?.clone()))
///     }
///
///     /// Extract the state and code from the URL a user was redirected to after authorizing the application.
///     fn extract_url<'a>(
///         url: &'a url::Url,
///     ) -> Result<(Cow<'a, str>, Cow<'a, str>), Option<(Cow<'a, str>, Cow<'a, str>)>> {
///         let query: BTreeMap<_, _> = url.query_pairs().collect();
///         if let Some((error, error_description)) = extract_pair(&query, "error", "error_description") {
///             Err(Some((error, error_description)))
///         } else if let Some((state, code)) = extract_pair(&query, "state", "code") {
///             Ok((state, code))
///         } else {
///             Err(None)
///         }
///     }
///     ```
/// 5. Finally, call [`get_user_token`](UserTokenBuilder::get_user_token) with the `state` and `code` query parameters to get the user's access token.
///
///     ```rust
///     # async move {
///     # use twitch_oauth2::{id::TwitchTokenResponse, tokens::UserTokenBuilder, Scope};
///     # use url::Url;
///     # use std::borrow::Cow;
///     # let redirect_url = Url::parse("http://localhost/twitch/register")?;
///     # let mut builder = UserTokenBuilder::new("myclientid", "myclientsecret", redirect_url);
///     # let (url, csrf_token) = builder.generate_url();
///     # fn extract_url<'a>(_: &'a url::Url) -> Result<(Cow<'a, str>, Cow<'a, str>), std::io::Error> { Ok((Cow::default(), Cow::default())) }
///     # let url = url::Url::parse("http://localhost/twitch/register?code=code&state=state")?;
///     # let client = twitch_oauth2::client::DummyClient; stringify!(
///     let client = reqwest::Client::builder()
///         .redirect(reqwest::redirect::Policy::none())
///         .build()?;
///     # );
///     let (state, code) = extract_url(&url)?;
///     let token = builder.get_user_token(&client, state.as_ref(), code.as_ref()).await?;
///     println!("User token: {:?}", token);
///     # Ok::<(), Box<dyn std::error::Error>>(())
///     # };
///     ```
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
    /// The `redirect_url` must be present, verbatim, on [the Twitch Developer Console](https://dev.twitch.tv/console).
    ///
    /// The `url` crate converts empty paths into "/" (such as `https://example.com` into `https://example.com/`),
    /// which means that you'll need to add `https://example.com/` to your redirect URIs (note the "trailing" slash) if you want to use an empty path.
    ///
    /// To avoid this, use a path such as `https://example.com/twitch/register` or similar instead, where the `url` crate would not add a trailing `/`.
    pub fn new(
        client_id: impl Into<ClientId>,
        client_secret: impl Into<ClientSecret>,
        redirect_url: url::Url,
    ) -> UserTokenBuilder {
        UserTokenBuilder {
            scopes: vec![],
            csrf: None,
            force_verify: false,
            redirect_url,
            client_id: client_id.into(),
            client_secret: client_secret.into(),
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
    /// First step in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#get-the-user-to-authorize-your-app)
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

    /// Check if the CSRF is valid
    pub fn csrf_is_valid(&self, csrf: &str) -> bool {
        if let Some(csrf2) = &self.csrf {
            csrf2.secret() == csrf
        } else {
            false
        }
    }

    /// Get the request for getting a [TwitchTokenResponse](crate::id::TwitchTokenResponse), to be used in [UserToken::from_response].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use twitch_oauth2::{tokens::UserTokenBuilder, id::TwitchTokenResponse};
    /// use url::Url;
    /// let callback_url = Url::parse("http://localhost/twitch/register")?;
    /// let mut builder = UserTokenBuilder::new("myclientid", "myclientsecret", callback_url);
    /// let (url, _csrf_code) = builder.generate_url();
    ///
    /// // Direct the user to this url.
    /// // Later when your server gets a response on `callback_url` with `?code=xxxxxxx&state=xxxxxxx&scope=aa%3Aaa+bb%3Abb`
    ///
    /// // validate the state
    /// # let state_in_query = _csrf_code.secret();
    /// if !builder.csrf_is_valid(state_in_query) {
    ///     panic!("state mismatched")
    /// }
    /// // and then get your token
    /// # let code_in_query = _csrf_code.secret();
    /// let request = builder.get_user_token_request(code_in_query);
    ///
    /// // use your favorite http client
    ///
    /// let response: http::Response<Vec<u8>> = client_req(request);
    /// let twitch_response = TwitchTokenResponse::from_response(&response)?;
    ///
    /// // you now have a access token, do what you want with it.
    /// // You're recommended to convert it into a `UserToken` via `UserToken::from_response`
    ///
    /// // You can validate the access_token like this
    /// let validated_req = twitch_response.access_token.validate_token_request();
    /// # fn client_req(_: http::Request<Vec<u8>>) -> http::Response<Vec<u8>> { http::Response::new(
    /// # r#"{"access_token":"rfx2uswqe8l4g1mkagrvg5tv0ks3","expires_in":14124,"refresh_token":"5b93chm6hdve3mycz05zfzatkfdenfspp1h1ar2xxdalen01","scope":["channel:moderate","chat:edit","chat:read"],"token_type":"bearer"}"#.bytes().collect()
    /// # ) }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_user_token_request(&self, code: &str) -> http::Request<Vec<u8>> {
        use http::{HeaderMap, Method};
        use std::collections::HashMap;
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.secret());
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", self.redirect_url.as_str());

        crate::construct_request(
            &crate::TOKEN_URL,
            &params,
            HeaderMap::new(),
            Method::POST,
            vec![],
        )
    }

    /// Generate the code with the help of the authorization code
    ///
    /// Last step in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#use-the-authorization-code-to-get-a-token)
    ///
    /// On failure to authenticate due to wrong redirect url or other errors, twitch redirects the user to `<redirect_url or first defined url in dev console>?error=<error type>&error_description=<description of error>`
    #[cfg(feature = "client")]
    pub async fn get_user_token<'a, C>(
        self,
        http_client: &'a C,
        state: &str,
        // TODO: Should be either str or AuthorizationCode
        code: &str,
    ) -> Result<UserToken, UserTokenExchangeError<<C as Client>::Error>>
    where
        C: Client,
    {
        if !self.csrf_is_valid(state) {
            return Err(UserTokenExchangeError::StateMismatch);
        }

        let req = self.get_user_token_request(code);

        let resp = http_client
            .req(req)
            .await
            .map_err(UserTokenExchangeError::RequestError)?;

        let response = crate::id::TwitchTokenResponse::from_response(&resp)?;
        let validated = response.access_token.validate_token(http_client).await?;

        UserToken::from_response(response, validated, self.client_secret)
            .map_err(|v| v.into_other().into())
    }
}

/// Builder for [OAuth implicit code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#implicit-grant-flow)
///
/// See [`UserTokenBuilder`] for the [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow) (requires Client Secret, generally more secure)
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
    /// The `redirect_url` must be present, verbatim, on [the Twitch Developer Console](https://dev.twitch.tv/console).
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
    /// First step in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#implicit-grant-flow)
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

    /// Check if the CSRF is valid
    pub fn csrf_is_valid(&self, csrf: &str) -> bool {
        if let Some(csrf2) = &self.csrf {
            csrf2.secret() == csrf
        } else {
            false
        }
    }

    /// Generate the code with the help of the hash.
    ///
    /// You can skip this method and instead use the token in the hash directly with [`UserToken::from_existing()`], but it's provided here for convenience.
    ///
    /// Last step in the [guide](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#implicit-grant-flow)
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
    #[cfg(feature = "client")]
    pub async fn get_user_token<'a, C>(
        self,
        http_client: &'a C,
        state: Option<&str>,
        access_token: Option<&str>,
        error: Option<&str>,
        error_description: Option<&str>,
    ) -> Result<UserToken, ImplicitUserTokenExchangeError<<C as Client>::Error>>
    where
        C: Client,
    {
        if !state.map(|s| self.csrf_is_valid(s)).unwrap_or_default() {
            return Err(ImplicitUserTokenExchangeError::StateMismatch);
        }

        match (access_token, error, error_description) {
            (Some(access_token), None, None) => UserToken::from_existing(
                http_client,
                crate::types::AccessToken::from(access_token),
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
    use crate::id::TwitchTokenResponse;

    pub use super::*;

    #[test]
    fn from_validated_and_token() {
        let body = br#"
        {
            "client_id": "wbmytr93xzw8zbg0p1izqyzzc5mbiz",
            "login": "twitchdev",
            "scopes": [
              "channel:read:subscriptions"
            ],
            "user_id": "141981764",
            "expires_in": 5520838
        }
        "#;
        let response = http::Response::builder().status(200).body(body).unwrap();
        let validated = ValidatedToken::from_response(&response).unwrap();
        let body = br#"
        {
            "access_token": "rfx2uswqe8l4g1mkagrvg5tv0ks3",
            "expires_in": 14124,
            "refresh_token": "5b93chm6hdve3mycz05zfzatkfdenfspp1h1ar2xxdalen01",
            "scope": [
                "channel:read:subscriptions"
            ],
            "token_type": "bearer"
          }
        "#;
        let response = http::Response::builder().status(200).body(body).unwrap();
        let response = TwitchTokenResponse::from_response(&response).unwrap();

        UserToken::from_response(response, validated, None).unwrap();
    }

    #[test]
    fn generate_url() {
        UserTokenBuilder::new(
            ClientId::from("random_client"),
            ClientSecret::from("random_secret"),
            url::Url::parse("https://localhost").unwrap(),
        )
        .force_verify(true)
        .generate_url()
        .0
        .to_string();
    }

    #[tokio::test]
    #[ignore]
    #[cfg(feature = "surf")]
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
        t.csrf = Some(crate::CsrfToken::from("random"));
        let token = t
            .get_user_token(&surf::Client::new(), "random", "authcode")
            .await
            .unwrap();
        println!("token: {:?} - {}", token, token.access_token.secret());
    }

    #[tokio::test]
    #[ignore]
    #[cfg(feature = "surf")]
    async fn get_implicit_token() {
        let mut t = ImplicitUserTokenBuilder::new(
            ClientId::new(
                std::env::var("TWITCH_CLIENT_ID").expect("no env:TWITCH_CLIENT_ID provided"),
            ),
            url::Url::parse(r#"http://localhost/twitch/register"#).unwrap(),
        )
        .force_verify(true);
        println!("{}", t.generate_url().0);
        t.csrf = Some(crate::CsrfToken::from("random"));
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
