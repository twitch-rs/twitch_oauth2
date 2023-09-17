//! Twitch token types

mod app_access_token;
pub mod errors;
mod user_token;

pub use app_access_token::AppAccessToken;
use twitch_types::{UserId, UserIdRef, UserName, UserNameRef};
pub use user_token::{ImplicitUserTokenBuilder, UserToken, UserTokenBuilder};

#[cfg(feature = "client")]
use crate::client::Client;
use crate::{id::TwitchTokenErrorResponse, scopes::Scope, RequestParseError};

use errors::ValidationError;
#[cfg(feature = "client")]
use errors::{RefreshTokenError, RevokeTokenError};

use crate::types::{AccessToken, ClientId};
use serde_derive::Deserialize;

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
#[cfg_attr(feature = "client", async_trait::async_trait)]
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
    #[cfg(feature = "client")]
    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client;
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
    /// Validate this token. Should be checked on regularly, according to <https://dev.twitch.tv/docs/authentication/validate-tokens/>
    ///
    /// # Note
    ///
    /// This will not mutate any current data in the [TwitchToken]
    #[cfg(feature = "client")]
    async fn validate_token<'a, C>(
        &self,
        http_client: &'a C,
    ) -> Result<ValidatedToken, ValidationError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client,
    {
        let token = &self.token();
        token.validate_token(http_client).await
    }

    /// Revoke the token. See <https://dev.twitch.tv/docs/authentication/revoke-tokens>
    #[cfg(feature = "client")]
    async fn revoke_token<'a, C>(
        self,
        http_client: &'a C,
    ) -> Result<(), RevokeTokenError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client,
    {
        let token = self.token();
        let client_id = self.client_id();
        token.revoke_token(http_client, client_id).await
    }
}

#[cfg_attr(feature = "client", async_trait::async_trait)]
impl<T: TwitchToken + Send> TwitchToken for Box<T> {
    fn token_type() -> BearerTokenType { T::token_type() }

    fn client_id(&self) -> &ClientId { (**self).client_id() }

    fn token(&self) -> &AccessToken { (**self).token() }

    fn login(&self) -> Option<&UserNameRef> { (**self).login() }

    fn user_id(&self) -> Option<&UserIdRef> { (**self).user_id() }

    #[cfg(feature = "client")]
    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client,
    {
        (**self).refresh_token(http_client).await
    }

    fn expires_in(&self) -> std::time::Duration { (**self).expires_in() }

    fn scopes(&self) -> &[Scope] { (**self).scopes() }
}

/// Token validation returned from `https://id.twitch.tv/oauth2/validate`
///
/// See <https://dev.twitch.tv/docs/authentication/validate-tokens/>
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
    #[serde(deserialize_with = "expires_in")]
    pub expires_in: Option<std::time::Duration>,
}

fn expires_in<'a, D: serde::de::Deserializer<'a>>(
    d: D,
) -> Result<Option<std::time::Duration>, D::Error> {
    use serde::Deserialize;
    let num = u64::deserialize(d)?;
    if num == 0 {
        Ok(None)
    } else {
        Ok(Some(std::time::Duration::from_secs(num)))
    }
}

impl ValidatedToken {
    /// Assemble a a validated token from a response.
    ///
    /// Get the request that generates this response with [`AccessToken::validate_token_request`][crate::types::AccessTokenRef::validate_token_request]
    pub fn from_response<B: AsRef<[u8]>>(
        response: &http::Response<B>,
    ) -> Result<ValidatedToken, ValidationError<std::convert::Infallible>> {
        match crate::parse_response(response) {
            Ok(ok) => Ok(ok),
            Err(err) => match err {
                RequestParseError::TwitchError(TwitchTokenErrorResponse { status, .. })
                    if status == http::StatusCode::UNAUTHORIZED =>
                {
                    Err(ValidationError::NotAuthorized)
                }
                err => Err(err.into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ValidatedToken;

    use super::errors::ValidationError;

    #[test]
    fn validated_token() {
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
        ValidatedToken::from_response(&response).unwrap();
    }

    #[test]
    fn validated_non_expiring_token() {
        let body = br#"
        {
            "client_id": "wbmytr93xzw8zbg0p1izqyzzc5mbiz",
            "login": "twitchdev",
            "scopes": [
              "channel:read:subscriptions"
            ],
            "user_id": "141981764",
            "expires_in": 0
        }
        "#;
        let response = http::Response::builder().status(200).body(body).unwrap();
        let token = ValidatedToken::from_response(&response).unwrap();
        assert!(token.expires_in.is_none());
    }

    #[test]
    fn validated_error_response() {
        let body = br#"
        {
            "status": 401,
            "message": "missing authorization token",
        }
        "#;
        let response = http::Response::builder().status(401).body(body).unwrap();
        let error = ValidatedToken::from_response(&response).unwrap_err();
        assert!(matches!(error, ValidationError::RequestParseError(_)))
    }
}
