//! Twitch token types

mod app_access_token;
pub mod errors;
mod user_token;

pub use app_access_token::AppAccessToken;
use twitch_types::{UserId, UserName, UserIdRef, UserNameRef};
pub use user_token::{ImplicitUserTokenBuilder, UserToken, UserTokenBuilder};

use crate::client::Client;
use crate::{scopes::Scope, validate_token};

use errors::*;

use crate::types::{AccessToken, ClientId};
use serde::Deserialize;

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
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # use twitch_oauth2::UserToken;
    /// # fn t() -> UserToken {todo!()}
    /// # let user_token = t();
    /// use twitch_oauth2::TwitchToken;
    /// println!("token: {}", user_token.token().secret());
    /// ```
    fn token(&self) -> &AccessToken;
    /// Get the username associated to this token
    fn login(&self) -> Option<&UserNameRef>;
    /// Get the user id associated to this token
    fn user_id(&self) -> Option<&UserIdRef>;
    /// Refresh this token, changing the token to a newer one
    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client<'a>>::Error>>
    where
        Self: Sized,
        C: Client<'a>;
    /// Get current lifetime of token.
    fn expires_in(&self) -> std::time::Duration;

    /// Returns whether or not the token is expired.
    ///
    /// ```rust, no_run
    /// # use twitch_oauth2::UserToken;
    /// # fn t() -> UserToken {todo!()}
    /// # #[tokio::main]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error + 'static>>{
    /// # let mut user_token = t();
    /// use twitch_oauth2::{UserToken, TwitchToken};
    /// if user_token.is_elapsed() {
    ///     user_token.refresh_token(&reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build()?).await?;
    /// }
    /// # Ok(()) }
    /// # fn main() {run();}
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
    async fn validate_token<'a, C>(
        &self,
        http_client: &'a C,
    ) -> Result<ValidatedToken, ValidationError<<C as Client<'a>>::Error>>
    where
        Self: Sized,
        C: Client<'a>,
    {
        let token = &self.token();
        validate_token(http_client, token).await
    }

    /// Revoke the token. See <https://dev.twitch.tv/docs/authentication#revoking-access-tokens>
    async fn revoke_token<'a, C>(
        self,
        http_client: &'a C,
    ) -> Result<(), RevokeTokenError<<C as Client<'a>>::Error>>
    where
        Self: Sized,
        C: Client<'a>,
    {
        let token = self.token();
        let client_id = self.client_id();
        crate::revoke_token(http_client, token, client_id).await
    }
}

#[async_trait::async_trait]
impl<T: TwitchToken + Send> TwitchToken for Box<T> {
    fn token_type() -> BearerTokenType { T::token_type() }

    fn client_id(&self) -> &ClientId { (**self).client_id() }

    fn token(&self) -> &AccessToken { (**self).token() }

    fn login(&self) -> Option<&UserNameRef> { (**self).login() }

    fn user_id(&self) -> Option<&UserIdRef> { (**self).user_id() }

    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client<'a>>::Error>>
    where
        Self: Sized,
        C: Client<'a>,
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
    pub login: Option<UserName>,
    /// User ID associated with the token
    pub user_id: Option<UserId>,
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
