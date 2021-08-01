//! Errors

use crate::id::TwitchTokenErrorResponse;
/// General errors for talking with twitch, used in [AppAccessToken::get_app_access_token][crate::tokens::AppAccessToken::get_app_access_token]
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum AppAccessTokenError<RE: std::error::Error + Send + Sync + 'static> {
    /// request for token failed. {0}
    Request(#[source] RE),
    /// twitch returned an unexpected status: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// could not get validation for token
    ValidationError(#[from] ValidationError<RE>),
    /// deserializations failed
    DeserializeError(#[from] serde_json::Error),
}

/// Errors for [validate_token][crate::validate_token]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum ValidationError<RE: std::error::Error + Send + Sync + 'static> {
    /// deserializations failed
    DeserializeError(#[from] serde_json::Error),
    /// token is not authorized for use
    NotAuthorized,
    /// twitch returned an unexpected status: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// failed to request validation: {0}
    Request(#[source] RE),
    // TODO: This should be in it's own error enum specifically for UserToken validation
    /// validation did not return a login when it was expected
    NoLogin,
}

/// Errors for [revoke_token][crate::revoke_token]
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum RevokeTokenError<RE: std::error::Error + Send + Sync + 'static> {
    /// twitch returned an unexpected error: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// failed to do revokation: {0}
    RequestError(#[source] RE),
}

/// Errors for [TwitchToken::refresh_token][crate::TwitchToken::refresh_token]
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum RefreshTokenError<RE: std::error::Error + Send + Sync + 'static> {
    /// request for token failed. {0}
    RequestError(#[source] RE),
    /// twitch returned an unexpected error: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// deserializations failed
    DeserializeError(#[from] serde_json::Error),
    /// no client secret found
    // TODO: Include this in doc
    // A client secret is needed to request a refreshed token.
    NoClientSecretFound,
    /// no refresh token found
    NoRefreshToken,
    /// no expiration found on new token
    NoExpiration,
}

/// Errors for [`UserTokenBuilder::get_user_token`](crate::tokens::UserTokenBuilder::get_user_token) and [`UserToken::mock_token`](crate::tokens::UserToken::mock_token)
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum UserTokenExchangeError<RE: std::error::Error + Send + Sync + 'static> {
    /// request for token failed. {0}
    RequestError(#[source] RE),
    /// twitch returned an unexpected error: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// deserializations failed
    DeserializeError(#[from] serde_json::Error),
    /// State CSRF does not match.
    StateMismatch,
    /// could not get validation for token
    ValidationError(#[from] ValidationError<RE>),
}

/// Errors for [ImplicitUserTokenBuilder::get_user_token][crate::tokens::ImplicitUserTokenBuilder::get_user_token]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum ImplicitUserTokenExchangeError<RE: std::error::Error + Send + Sync + 'static> {
    // FIXME: should be TwitchTokenErrorResponse
    /// twitch returned an error: {error:?} - {description:?}
    TwitchError {
        /// Error type
        error: Option<String>,
        /// Description of error
        description: Option<String>,
    },
    /// State CSRF does not match.
    StateMismatch,
    /// could not get validation for token
    ValidationError(#[from] ValidationError<RE>),
}
