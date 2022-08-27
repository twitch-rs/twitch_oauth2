#![allow(unknown_lints, renamed_and_removed_lints)]
#![deny(missing_docs, broken_intra_doc_links)] // This will be weird until 1.52, see https://github.com/rust-lang/rust/pull/80527
#![cfg_attr(nightly, deny(rustdoc::broken_intra_doc_links))]
#![cfg_attr(nightly, feature(doc_cfg))]
#![doc(html_root_url = "https://docs.rs/twitch_oauth2/0.8.0")]
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
//! # use twitch_oauth2::{TwitchToken, UserToken, AccessToken, tokens::errors::ValidationError};
//! # #[tokio::main]
//! # async fn run() {
//! # let reqwest_http_client = twitch_oauth2::client::DummyClient; // This is only here to fool doc tests
//!     let token = AccessToken::new("sometokenherewhichisvalidornot".to_string());
//!
//!     match UserToken::from_existing(&reqwest_http_client, token, None, None).await {
//!         Ok(t) => println!("user_token: {}", t.token().secret()),
//!         Err(e) => panic!("got error: {}", e),
//!     }
//! # }
//! # fn main() {run()}
//! ```
pub mod client;
pub mod id;
pub mod scopes;
pub mod tokens;
pub mod types;

use http::StatusCode;
use id::TwitchTokenErrorResponse;
use tokens::errors::{RefreshTokenError, RevokeTokenError, ValidationError};

#[doc(inline)]
pub use scopes::Scope;
#[doc(inline)]
pub use tokens::{AppAccessToken, TwitchToken, UserToken, ValidatedToken};

pub use url;

pub use types::{AccessToken, ClientId, ClientSecret, CsrfToken, RefreshToken};

#[doc(hidden)]
pub use types::{AccessTokenRef, ClientIdRef, ClientSecretRef, CsrfTokenRef, RefreshTokenRef};

use self::client::Client;

type HttpRequest = http::Request<Vec<u8>>;
type HttpResponse = http::Response<Vec<u8>>;

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

/// Validate this token.
///
/// Should be checked on regularly, according to <https://dev.twitch.tv/docs/authentication#validating-requests>
pub async fn validate_token<'a, C>(
    client: &'a C,
    token: &AccessTokenRef,
) -> Result<ValidatedToken, ValidationError<<C as Client<'a>>::Error>>
where
    C: Client<'a>,
{
    use http::{header::AUTHORIZATION, HeaderMap, Method};

    let auth_header = format!("OAuth {}", token.secret());
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        auth_header
            .parse()
            .expect("Failed to parse header for validation"),
    );

    let req = crate::construct_request::<&[(String, String)], _, _>(
        &crate::VALIDATE_URL,
        &[],
        headers,
        Method::GET,
        vec![],
    );

    let resp = client.req(req).await.map_err(ValidationError::Request)?;
    if resp.status() == StatusCode::UNAUTHORIZED {
        return Err(ValidationError::NotAuthorized);
    }
    match crate::parse_response(&resp) {
        Ok(ok) => Ok(ok),
        Err(err) => match err {
            RequestParseError::TwitchError(TwitchTokenErrorResponse { status, .. })
                if status == StatusCode::UNAUTHORIZED =>
            {
                Err(ValidationError::NotAuthorized)
            }
            err => Err(err.into()),
        },
    }
}

/// Revoke the token.
///
/// See <https://dev.twitch.tv/docs/authentication#revoking-access-tokens>
pub async fn revoke_token<'a, C>(
    http_client: &'a C,
    token: &AccessToken,
    client_id: &ClientId,
) -> Result<(), RevokeTokenError<<C as Client<'a>>::Error>>
where
    C: Client<'a>,
{
    use http::{HeaderMap, Method};
    use std::collections::HashMap;
    let mut params = HashMap::new();
    params.insert("client_id", client_id.as_str());
    params.insert("token", token.secret());

    let req = construct_request(
        &crate::REVOKE_URL,
        &params,
        HeaderMap::new(),
        Method::POST,
        vec![],
    );

    let resp = http_client
        .req(req)
        .await
        .map_err(RevokeTokenError::RequestError)?;

    let _ = parse_token_response_raw(&resp)?;
    Ok(())
}

/// Refresh the token, call if it has expired.
///
/// See <https://dev.twitch.tv/docs/authentication#refreshing-access-tokens>
pub async fn refresh_token<'a, C>(
    http_client: &'a C,
    refresh_token: &RefreshTokenRef,
    client_id: &ClientId,
    client_secret: &ClientSecret,
) -> Result<
    (AccessToken, std::time::Duration, Option<RefreshToken>),
    RefreshTokenError<<C as Client<'a>>::Error>,
>
where
    C: Client<'a>,
{
    use http::{HeaderMap, Method};
    use std::collections::HashMap;

    let mut params = HashMap::new();
    params.insert("client_id", client_id.as_str());
    params.insert("client_secret", client_secret.secret());
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token.secret());

    let req = construct_request(
        &crate::TOKEN_URL,
        &params,
        HeaderMap::new(),
        Method::POST,
        vec![],
    );

    let resp = http_client
        .req(req)
        .await
        .map_err(RefreshTokenError::RequestError)?;
    let res: id::TwitchTokenResponse = parse_response(&resp)?;

    let expires_in = res.expires_in().ok_or(RefreshTokenError::NoExpiration)?;
    let refresh_token = res.refresh_token;
    let access_token = res.access_token;
    Ok((access_token, expires_in, refresh_token))
}

/// Construct a request that accepts `application/json` on default
fn construct_request<I, K, V>(
    url: &url::Url,
    params: I,
    headers: http::HeaderMap,
    method: http::Method,
    body: Vec<u8>,
) -> HttpRequest
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
pub(crate) fn parse_token_response_raw(
    resp: &HttpResponse,
) -> Result<&HttpResponse, RequestParseError> {
    match serde_json::from_slice::<TwitchTokenErrorResponse>(resp.body()) {
        Err(_) => match resp.status() {
            StatusCode::OK => Ok(resp),
            _ => Err(RequestParseError::Other(resp.status())),
        },
        Ok(twitch_err) => Err(RequestParseError::TwitchError(twitch_err)),
    }
}

/// Parses a response, validating it and returning json deserialized response
pub(crate) fn parse_response<T: serde::de::DeserializeOwned>(
    resp: &HttpResponse,
) -> Result<T, RequestParseError> {
    let body = parse_token_response_raw(resp)?.body();
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
