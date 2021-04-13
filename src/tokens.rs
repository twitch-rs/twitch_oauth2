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

#[derive(Clone, Debug, PartialEq, Eq)]
/// Types of bearer tokens
pub enum BearerTokenType {
    /// Token for making requests in the context of an authenticated user.
    UserToken,
    /// Token for server-to-server requests.
    ///
    /// In some contexts (i.e [EventSub](https://dev.twitch.tv/docs/eventsub)) an App Access Token can be used in the context of users that have authenticated
    /// the specific Client ID
    AppAccessToken,
}

/// Trait for twitch tokens to get fields and generalize over [AppAccessToken] and [UserToken]
#[async_trait::async_trait]
pub trait TwitchToken {
    /// Get the type of token.
    fn token_type() -> BearerTokenType;
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
        Self: Sized,
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F + Send,
        F: Future<Output = Result<HttpResponse, RE>> + Send;
    /// Get current lifetime of token.
    fn expires_in(&self) -> std::time::Duration;

    /// Returns whether or not the token is expired.
    fn is_elapsed(&self) -> bool {
        let exp = self.expires_in();
        exp.as_secs() == 0 && exp.as_nanos() == 0
    }
    /// Retrieve scopes attached to the token
    fn scopes(&self) -> &[Scope];
    /// Validate this token. Should be checked on regularly, according to <https://dev.twitch.tv/docs/authentication#validating-requests>
    ///
    /// # Note
    ///
    /// This will not mutate any current data in the [TwitchToken]
    async fn validate_token<RE, C, F>(
        &self,
        http_client: C,
    ) -> Result<ValidatedToken, ValidationError<RE>>
    where
        Self: Sized,
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F + Send,
        F: Future<Output = Result<HttpResponse, RE>> + Send,
    {
        let token = &self.token();
        validate_token(http_client, &token).await
    }
    /// Revoke the token. See <https://dev.twitch.tv/docs/authentication#revoking-access-tokens>
    async fn revoke_token<RE, C, F>(self, http_client: C) -> Result<(), RevokeTokenError<RE>>
    where
        Self: Sized,
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F + Send,
        F: Future<Output = Result<HttpResponse, RE>> + Send, {
        let token = self.token();
        let client_id = self.client_id();
        crate::revoke_token(http_client, &token, &client_id).await
    }
}

#[async_trait::async_trait]
impl<T: TwitchToken + Send> TwitchToken for Box<T> {
    fn token_type() -> BearerTokenType { T::token_type() }

    fn client_id(&self) -> &ClientId { (**self).client_id() }

    fn token(&self) -> &AccessToken { (**self).token() }

    fn login(&self) -> Option<&str> { (**self).login() }

    async fn refresh_token<RE, C, F>(
        &mut self,
        http_client: C,
    ) -> Result<(), RefreshTokenError<RE>>
    where
        Self: Sized,
        RE: std::error::Error + Send + Sync + 'static,
        C: FnOnce(HttpRequest) -> F + Send,
        F: Future<Output = Result<HttpResponse, RE>> + Send,
    {
        (**self).refresh_token(http_client).await
    }

    fn expires_in(&self) -> std::time::Duration { (**self).expires_in() }

    fn scopes(&self) -> &[Scope] { (**self).scopes() }
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
    pub expires_in: std::time::Duration,
}

fn seconds_to_duration<'a, D: serde::de::Deserializer<'a>>(
    d: D,
) -> Result<std::time::Duration, D::Error> {
    Ok(std::time::Duration::from_secs(u64::deserialize(d)?))
}
