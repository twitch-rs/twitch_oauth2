//! Errors

use crate::id::TwitchTokenErrorResponse;
use oauth2::HttpResponse as OAuth2HttpResponse;
use oauth2::RequestTokenError;
/// General errors for talking with twitch, used in [AppAccessToken::get_app_access_token][crate::tokens::AppAccessToken::get_app_access_token]
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum TokenError<RE: std::error::Error + Send + Sync + 'static> {
    /// request for token failed. {0}
    Request(RequestTokenError<RE, TwitchTokenErrorResponse>),
    /// could not parse url
    ParseError(#[from] oauth2::url::ParseError),
    /// could not get validation for token
    ValidationError(#[from] ValidationError<RE>),
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
    /// validation did not return a login name when it was expected
    NoLogin,
}

/// Errors for [revoke_token][crate::revoke_token]
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum RevokeTokenError<RE: std::error::Error + Send + Sync + 'static> {
    /// 400 Bad Request: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// failed to do revokation: {0}
    RequestError(#[source] RE),
    /// got unexpected return: {0:?}
    Other(OAuth2HttpResponse),
}

/// Errors for [TwitchToken::refresh_token][crate::TwitchToken::refresh_token]
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum RefreshTokenError<RE: std::error::Error + Send + Sync + 'static> {
    /// request for token failed. {0}
    RequestError(#[source] RequestTokenError<RE, TwitchTokenErrorResponse>),
    /// could not parse url
    ParseError(#[from] oauth2::url::ParseError),
    /// no client secret found
    ///
    /// A client secret is needed to request a refreshed token.
    NoClientSecretFound,
    /// no refresh token found
    NoRefreshToken,
}

/// Errors for [UserTokenBuilder::get_user_token][crate::tokens::UserTokenBuilder::get_user_token]
#[derive(thiserror::Error, Debug, displaydoc::Display)]
pub enum UserTokenExchangeError<RE: std::error::Error + Send + Sync + 'static> {
    /// request for token failed. {0}
    RequestError(#[source] RE),
    /// could not parse url
    ParseError(#[from] oauth2::url::ParseError),
    /// twitch returned an unexpected status: {0}
    TwitchError(TwitchTokenErrorResponse),
    /// deserializations failed
    DeserializeError(#[from] serde_json::Error),
    /// State CSRF does not match.
    StateMismatch,
    /// could not get validation for token
    ValidationError(#[from] ValidationError<RE>),
}
