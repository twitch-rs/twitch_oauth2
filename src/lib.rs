#![allow(unknown_lints, renamed_and_removed_lints)]
#![deny(missing_docs, broken_intra_doc_links)] // This will be weird until 1.52, see https://github.com/rust-lang/rust/pull/80527
#![cfg_attr(nightly, deny(rustdoc::broken_intra_doc_links))]
#![cfg_attr(nightly, feature(doc_cfg))]
#![cfg_attr(nightly, feature(doc_auto_cfg))]
//! [![github]](https://github.com/twitch-rs/twitch_oauth2)&ensp;[![crates-io]](https://crates.io/crates/twitch_oauth2)&ensp;[![docs-rs]](https://docs.rs/twitch_oauth2/0.8.0/twitch_oauth2)
//!
//! [github]: https://img.shields.io/badge/github-twitch--rs/twitch__oauth2-8da0cb?style=for-the-badge&labelColor=555555&logo=github"
//! [crates-io]: https://img.shields.io/crates/v/twitch_oauth2.svg?style=for-the-badge&color=fc8d62&logo=rust"
//! [docs-rs]: https://img.shields.io/badge/docs.rs-twitch__oauth2-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K"
//!
//! <br>
//!
//! <h5>OAuth2 for Twitch endpoints</h5>
//!
//! ```rust,no_run
//! use twitch_oauth2::{tokens::errors::ValidationError, AccessToken, TwitchToken, UserToken};
//! // Make sure you enable the feature "reqwest" for twitch_oauth2 if you want to use reqwest
//! # async {let client = twitch_oauth2::client::DummyClient; stringify!(
//! let client = reqwest::Client::builder()
//!     .redirect(reqwest::redirect::Policy::none())
//!     .build()?;
//! # );
//! let token = AccessToken::new("sometokenherewhichisvalidornot".to_string());
//! let token = UserToken::from_token(&client, token).await?;
//! println!("token: {:?}", token.token()); // prints `[redacted access token]`
//! # Ok::<(), Box<dyn std::error::Error>>(())};
//! ```
//!
//! # About
//!
//! ## Scopes
//!
//! The library contains all known twitch oauth2 scopes in [`Scope`].
//!
//! ## User Access Tokens
//!
//! For most basic use cases with user authorization, [`UserToken::from_token`] will be your main way
//! to create user tokens in this library.
//!
//! Things like [`UserTokenBuilder`] can be used to create a token from scratch, via the [OAuth authorization code flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow)
//!
//! ## App access token
//!
//! Similar to [`UserToken`], a token with authorization as the twitch application can be created with
//! [`AppAccessToken::get_app_access_token`].
//!
//! ## HTTP Requests
//!
//! To enable client features with a supported http library, enable the http library feature in `twitch_oauth2`, like `twitch_oauth2 = { features = ["reqwest"], version = "0.13.0" }`.
//! If you're using [twitch_api](https://crates.io/crates/twitch_api), you can use its [`HelixClient`](https://docs.rs/twitch_api/latest/twitch_api/struct.HelixClient.html) instead of the underlying http client.
//!
//!
//! This library can be used without any specific http client library (like if you don't want to use `await`),
//! using methods like [`AppAccessToken::from_response`] and [`AppAccessToken::get_app_access_token_request`]
//! or [`UserTokenBuilder::get_user_token_request`] and [`UserToken::from_response`]
#[cfg(feature = "client")]
pub mod client;
pub mod id;
pub mod scopes;
pub mod tokens;
pub mod types;

use http::StatusCode;
use id::TwitchTokenErrorResponse;
#[cfg(feature = "client")]
use tokens::errors::{RefreshTokenError, RevokeTokenError, ValidationError};

#[doc(inline)]
pub use scopes::{Scope, Validator};
#[doc(inline)]
pub use tokens::{
    AppAccessToken, ImplicitUserTokenBuilder, TwitchToken, UserToken, UserTokenBuilder,
    ValidatedToken,
};

pub use url;

pub use types::{AccessToken, ClientId, ClientSecret, CsrfToken, RefreshToken};

#[doc(hidden)]
pub use types::{AccessTokenRef, ClientIdRef, ClientSecretRef, CsrfTokenRef, RefreshTokenRef};

#[cfg(feature = "client")]
use self::client::Client;

/// Generate a url with a default if `mock_api` feature is disabled, or env var is not defined or is invalid utf8
macro_rules! mock_env_url {
    ($var:literal, $default:expr $(,)?) => {
        once_cell::sync::Lazy::new(move || {
            #[cfg(feature = "mock_api")]
            if let Ok(url) = std::env::var($var) {
                return url::Url::parse(&url).expect(concat!(
                    "URL could not be made from `env:",
                    $var,
                    "`."
                ));
            };
            url::Url::parse(&$default).unwrap()
        })
    };
}

/// Defines the root path to twitch auth endpoints
static TWITCH_OAUTH2_URL: once_cell::sync::Lazy<url::Url> =
    mock_env_url!("TWITCH_OAUTH2_URL", "https://id.twitch.tv/oauth2/");

/// Authorization URL (`https://id.twitch.tv/oauth2/authorize`) for `id.twitch.tv`
///
/// Can be overridden when feature `mock_api` is enabled with environment variable `TWITCH_OAUTH2_URL` to set the root path, or with `TWITCH_OAUTH2_AUTH_URL` to override the base (`https://id.twitch.tv/oauth2/`) url.
///
/// # Examples
///
/// Set the environment variable `TWITCH_OAUTH2_URL` to `http://localhost:8080/auth/` to use [`twitch-cli` mock](https://github.com/twitchdev/twitch-cli/blob/main/docs/mock-api.md) endpoints.
pub static AUTH_URL: once_cell::sync::Lazy<url::Url> = mock_env_url!("TWITCH_OAUTH2_AUTH_URL", {
    TWITCH_OAUTH2_URL.to_string() + "authorize"
},);
/// Token URL (`https://id.twitch.tv/oauth2/token`) for `id.twitch.tv`
///
/// Can be overridden when feature `mock_api` is enabled with environment variable `TWITCH_OAUTH2_URL` to set the root path, or with `TWITCH_OAUTH2_TOKEN_URL` to override the base (`https://id.twitch.tv/oauth2/`) url.
///
/// # Examples
///
/// Set the environment variable `TWITCH_OAUTH2_URL` to `http://localhost:8080/auth/` to use [`twitch-cli` mock](https://github.com/twitchdev/twitch-cli/blob/main/docs/mock-api.md) endpoints.
pub static TOKEN_URL: once_cell::sync::Lazy<url::Url> = mock_env_url!("TWITCH_OAUTH2_TOKEN_URL", {
    TWITCH_OAUTH2_URL.to_string() + "token"
},);
/// Validation URL (`https://id.twitch.tv/oauth2/validate`) for `id.twitch.tv`
///
/// Can be overridden when feature `mock_api` is enabled with environment variable `TWITCH_OAUTH2_URL` to set the root path, or with `TWITCH_OAUTH2_VALIDATE_URL` to override the base (`https://id.twitch.tv/oauth2/`) url.
///
/// # Examples
///
/// Set the environment variable `TWITCH_OAUTH2_URL` to `http://localhost:8080/auth/` to use [`twitch-cli` mock](https://github.com/twitchdev/twitch-cli/blob/main/docs/mock-api.md) endpoints.
pub static VALIDATE_URL: once_cell::sync::Lazy<url::Url> =
    mock_env_url!("TWITCH_OAUTH2_VALIDATE_URL", {
        TWITCH_OAUTH2_URL.to_string() + "validate"
    },);
/// Revokation URL (`https://id.twitch.tv/oauth2/revoke`) for `id.twitch.tv`
///
/// Can be overridden when feature `mock_api` is enabled with environment variable `TWITCH_OAUTH2_URL` to set the root path, or with `TWITCH_OAUTH2_REVOKE_URL` to override the base (`https://id.twitch.tv/oauth2/`) url.
///
/// # Examples
///
/// Set the environment variable `TWITCH_OAUTH2_URL` to `http://localhost:8080/auth/` to use [`twitch-cli` mock](https://github.com/twitchdev/twitch-cli/blob/main/docs/mock-api.md) endpoints.
pub static REVOKE_URL: once_cell::sync::Lazy<url::Url> =
    mock_env_url!("TWITCH_OAUTH2_REVOKE_URL", {
        TWITCH_OAUTH2_URL.to_string() + "revoke"
    },);

impl AccessTokenRef {
    /// Get the request needed to validate this token.
    ///
    /// Parse the response from this endpoint with [ValidatedToken::from_response](crate::ValidatedToken::from_response)
    pub fn validate_token_request(&self) -> http::Request<Vec<u8>> {
        use http::{header::AUTHORIZATION, HeaderMap, Method};

        let auth_header = format!("OAuth {}", self.secret());
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            auth_header
                .parse()
                .expect("Failed to parse header for validation"),
        );

        crate::construct_request::<&[(String, String)], _, _>(
            &crate::VALIDATE_URL,
            &[],
            headers,
            Method::GET,
            vec![],
        )
    }

    /// Validate this token.
    ///
    /// Should be checked on regularly, according to <https://dev.twitch.tv/docs/authentication/validate-tokens/>
    #[cfg(feature = "client")]
    pub async fn validate_token<'a, C>(
        &self,
        client: &'a C,
    ) -> Result<ValidatedToken, ValidationError<<C as Client>::Error>>
    where
        C: Client,
    {
        let req = self.validate_token_request();

        let resp = client.req(req).await.map_err(ValidationError::Request)?;
        if resp.status() == StatusCode::UNAUTHORIZED {
            return Err(ValidationError::NotAuthorized);
        }
        ValidatedToken::from_response(&resp).map_err(|v| v.into_other())
    }

    /// Get the request needed to revoke this token.
    pub fn revoke_token_request(&self, client_id: &ClientId) -> http::Request<Vec<u8>> {
        use http::{HeaderMap, Method};
        use std::collections::HashMap;
        let mut params = HashMap::new();
        params.insert("client_id", client_id.as_str());
        params.insert("token", self.secret());

        construct_request(
            &crate::REVOKE_URL,
            &params,
            HeaderMap::new(),
            Method::POST,
            vec![],
        )
    }

    /// Revoke the token.
    ///
    /// See <https://dev.twitch.tv/docs/authentication/revoke-tokens/>
    #[cfg(feature = "client")]
    pub async fn revoke_token<'a, C>(
        &self,
        http_client: &'a C,
        client_id: &ClientId,
    ) -> Result<(), RevokeTokenError<<C as Client>::Error>>
    where
        C: Client,
    {
        let req = self.revoke_token_request(client_id);

        let resp = http_client
            .req(req)
            .await
            .map_err(RevokeTokenError::RequestError)?;

        let _ = parse_token_response_raw(&resp)?;
        Ok(())
    }
}

impl RefreshTokenRef {
    /// Get the request needed to refresh this token.
    ///
    /// Parse the response from this endpoint with [TwitchTokenResponse::from_response](crate::id::TwitchTokenResponse::from_response)
    pub fn refresh_token_request(
        &self,
        client_id: &ClientId,
        client_secret: &ClientSecret,
    ) -> http::Request<Vec<u8>> {
        use http::{HeaderMap, Method};
        use std::collections::HashMap;

        let mut params = HashMap::new();
        params.insert("client_id", client_id.as_str());
        params.insert("client_secret", client_secret.secret());
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", self.secret());

        construct_request(
            &crate::TOKEN_URL,
            &params,
            HeaderMap::new(),
            Method::POST,
            vec![],
        )
    }

    /// Refresh the token, call if it has expired.
    ///
    /// See <https://dev.twitch.tv/docs/authentication/refresh-tokens>
    #[cfg(feature = "client")]
    pub async fn refresh_token<'a, C>(
        &self,
        http_client: &'a C,
        client_id: &ClientId,
        client_secret: &ClientSecret,
    ) -> Result<
        (AccessToken, std::time::Duration, Option<RefreshToken>),
        RefreshTokenError<<C as Client>::Error>,
    >
    where
        C: Client,
    {
        let req = self.refresh_token_request(client_id, client_secret);

        let resp = http_client
            .req(req)
            .await
            .map_err(RefreshTokenError::RequestError)?;
        let res = id::TwitchTokenResponse::from_response(&resp)?;

        let expires_in = res.expires_in().ok_or(RefreshTokenError::NoExpiration)?;
        let refresh_token = res.refresh_token;
        let access_token = res.access_token;
        Ok((access_token, expires_in, refresh_token))
    }
}

/// Construct a request that accepts `application/json` on default
fn construct_request<I, K, V>(
    url: &url::Url,
    params: I,
    headers: http::HeaderMap,
    method: http::Method,
    body: Vec<u8>,
) -> http::Request<Vec<u8>>
where
    I: std::iter::IntoIterator,
    I::Item: std::borrow::Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut url = url.clone();
    url.query_pairs_mut().extend_pairs(params);
    let url: String = url.into();
    let mut req = http::Request::builder().method(method).uri(url);
    req.headers_mut()
        .map(|h| h.extend(headers.into_iter()))
        .unwrap();
    req.headers_mut()
        .map(|h| {
            if !h.contains_key(http::header::ACCEPT) {
                h.append(http::header::ACCEPT, "application/json".parse().unwrap());
            }
        })
        .unwrap();
    req.body(body).unwrap()
}

/// Parses a response, validating it and returning the response if all ok.
pub(crate) fn parse_token_response_raw<B: AsRef<[u8]>>(
    resp: &http::Response<B>,
) -> Result<&http::Response<B>, RequestParseError> {
    match serde_json::from_slice::<TwitchTokenErrorResponse>(resp.body().as_ref()) {
        Err(_) => match resp.status() {
            StatusCode::OK => Ok(resp),
            _ => Err(RequestParseError::Other(resp.status())),
        },
        Ok(twitch_err) => Err(RequestParseError::TwitchError(twitch_err)),
    }
}

/// Parses a response, validating it and returning json deserialized response
pub(crate) fn parse_response<T: serde::de::DeserializeOwned, B: AsRef<[u8]>>(
    resp: &http::Response<B>,
) -> Result<T, RequestParseError> {
    let body = parse_token_response_raw(resp)?.body().as_ref();
    if let Some(_content) = resp.headers().get(http::header::CONTENT_TYPE) {
        // TODO: Remove this cfg, see issue https://github.com/twitchdev/twitch-cli/issues/81
        #[cfg(not(feature = "mock_api"))]
        if _content != "application/json" {
            return Err(RequestParseError::NotJson {
                found: String::from_utf8_lossy(_content.as_bytes()).into_owned(),
            });
        }
    }
    serde_json::from_slice(body).map_err(Into::into)
}

/// Errors from parsing responses
#[derive(Debug, thiserror::Error, displaydoc::Display)]
#[non_exhaustive]
pub enum RequestParseError {
    /// deserialization failed
    DeserializeError(#[from] serde_json::Error),
    /// twitch returned an error
    TwitchError(#[from] TwitchTokenErrorResponse),
    /// returned content is not `application/json`, found `{found}`
    NotJson {
        /// Found `Content-Type` header
        found: String,
    },
    /// twitch returned an unexpected status code: {0}
    Other(StatusCode),
}
