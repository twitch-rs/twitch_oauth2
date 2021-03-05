//! Twitch token types

mod app_access_token;
pub mod errors;
mod user_token;

pub use app_access_token::AppAccessToken;
pub use user_token::{UserToken, UserTokenBuilder};

use crate::{scopes::Scope, validate_token};

use errors::*;

use oauth2::{AccessToken, ClientId};
use oauth2::{HttpRequest, HttpResponse};
use serde::Deserialize;
use std::future::Future;

/// Trait for twitch tokens to get fields and generalize over [AppAccessToken] and [UserToken]
#[async_trait::async_trait(?Send)]
pub trait TwitchToken {
    /// Client ID associated with the token. Twitch requires this in all helix API calls
    fn client_id(&self) -> &ClientId;
    /// Get the [AccessToken] for authenticating
    fn token(&self) -> &AccessToken;
    /// Get the username associated to this token
    fn login(&self) -> Option<&str>;
    /// Refresh this token, changing the token to a newer one
    async fn refresh_token<RE, C, F>(
        &mut self,
        http_client: C,
    ) -> Result<(), RefreshTokenError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>;
    /// Get current lifetime of token.
    fn expires_in(&self) -> Option<std::time::Duration>;
    /// Retrieve scopes attached to the token
    fn scopes(&self) -> Option<&[Scope]>;
    /// Validate this token. Should be checked on regularly, according to <https://dev.twitch.tv/docs/authentication#validating-requests>
    async fn validate_token<RE, C, F>(
        &self,
        http_client: C,
    ) -> Result<ValidatedToken, ValidationError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        validate_token(http_client, &self.token()).await
    }
    /// Revoke the token. See <https://dev.twitch.tv/docs/authentication#revoking-access-tokens>
    async fn revoke_token<RE, C, F>(self, http_client: C) -> Result<(), RevokeTokenError<RE>>
    where
        Self: Sized,
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>, {
        crate::revoke_token(http_client, self.token(), self.client_id()).await
    }
}

#[async_trait::async_trait(?Send)]
impl<T: TwitchToken> TwitchToken for Box<T> {
    fn client_id(&self) -> &ClientId { (**self).client_id() }

    fn token(&self) -> &AccessToken { (**self).token() }

    fn login(&self) -> Option<&str> { (**self).login() }

    async fn refresh_token<RE, C, F>(
        &mut self,
        http_client: C,
    ) -> Result<(), RefreshTokenError<RE>>
    where
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F,
        F: Future<Output = Result<HttpResponse, RE>>,
    {
        (**self).refresh_token(http_client).await
    }

    fn expires_in(&self) -> Option<std::time::Duration> { (**self).expires_in() }

    fn scopes(&self) -> Option<&[Scope]> { (**self).scopes() }
}

/// Token validation returned from `https://id.twitch.tv/oauth2/validate`
///
/// See <https://dev.twitch.tv/docs/authentication#validating-requests>
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatedToken {
    /// Client ID associated with the token. Twitch requires this in all helix API calls
    pub client_id: ClientId,
    /// Username associated with the token
    pub login: Option<String>,
    /// User ID associated with the token
    pub user_id: Option<String>,
    /// Scopes attached to the token.
    pub scopes: Option<Vec<Scope>>,
    /// Lifetime of the token
    #[serde(deserialize_with = "seconds_to_duration")]
    pub expires_in: Option<std::time::Duration>,
}

fn seconds_to_duration<'a, D: serde::de::Deserializer<'a>>(
    d: D,
) -> Result<Option<std::time::Duration>, D::Error> {
    let seconds = Option::<u64>::deserialize(d)?;
    Ok(seconds.map(std::time::Duration::from_secs))
}
