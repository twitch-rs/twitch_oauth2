//! Representation of oauth2 flow in `id.twitch.tv`

use serde_derive::{Deserialize, Serialize};

use crate::{AccessToken, RequestParseError};
use std::time::Duration;
/// Twitch's representation of the oauth flow.
///
/// Retrieve with
///
/// * [`UserTokenBuilder::get_user_token_request`](crate::tokens::UserTokenBuilder::get_user_token_request)
/// * [`AppAccessToken::::get_app_access_token_request`](crate::tokens::AppAccessToken::get_app_access_token_request)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TwitchTokenResponse {
    /// Access token
    pub access_token: AccessToken,
    /// Time (in seconds) until token expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    /// Token that can be used to refresh
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<crate::RefreshToken>,
    /// Scopes attached to token
    #[serde(rename = "scope", deserialize_with = "scope::deserialize")]
    #[serde(default)]
    pub scopes: Option<Vec<crate::Scope>>,
}

impl TwitchTokenResponse {
    /// Create a [TwitchTokenResponse] from a [http::Response]
    pub fn from_response<B: AsRef<[u8]>>(
        response: &http::Response<B>,
    ) -> Result<TwitchTokenResponse, RequestParseError> {
        crate::parse_response(response)
    }
}

/// Twitch's representation of the oauth flow for errors
#[derive(Clone, Debug, Deserialize, Serialize, thiserror::Error)]
pub struct TwitchTokenErrorResponse {
    /// Status code of error
    #[serde(with = "status_code")]
    pub status: http::StatusCode,
    /// Message attached to error
    pub message: String,
    /// Description of the error message.
    pub error: Option<String>,
}

impl std::fmt::Display for TwitchTokenErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{error} - {message}",
            error = self
                .error
                .as_deref()
                .unwrap_or_else(|| self.status.canonical_reason().unwrap_or("Error")),
            message = self.message
        )
    }
}

#[doc(hidden)]
pub mod status_code {
    use http::StatusCode;
    use serde::{
        de::{Deserialize, Error, Unexpected},
        Deserializer, Serializer,
    };

    pub fn deserialize<'de, D>(de: D) -> Result<StatusCode, D::Error>
    where D: Deserializer<'de> {
        let code: u16 = Deserialize::deserialize(de)?;
        match StatusCode::from_u16(code) {
            Ok(code) => Ok(code),
            Err(_) => Err(Error::invalid_value(
                Unexpected::Unsigned(code as u64),
                &"a value between 100 and 600",
            )),
        }
    }

    pub fn serialize<S>(status: &StatusCode, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        ser.serialize_u16(status.as_u16())
    }
}

#[doc(hidden)]
pub mod scope {
    use serde::{de::Deserialize, Deserializer};

    pub fn deserialize<'de, D>(de: D) -> Result<Option<Vec<crate::Scope>>, D::Error>
    where D: Deserializer<'de> {
        let scopes: Option<Vec<crate::Scope>> = Deserialize::deserialize(de)?;
        if let Some(scopes) = scopes {
            match scopes {
                scopes if scopes.is_empty() || scopes.len() > 1 => Ok(Some(scopes)),
                scopes if scopes.len() == 1 && scopes.get(0).unwrap().as_str() == "" => Ok(None),
                _ => Ok(Some(scopes)),
            }
        } else {
            Ok(None)
        }
    }
}

impl TwitchTokenResponse {
    /// Get the access token from this response
    pub fn access_token(&self) -> &crate::AccessTokenRef { &self.access_token }

    /// Get the expires in from this response
    pub fn expires_in(&self) -> Option<Duration> { self.expires_in.map(Duration::from_secs) }

    /// Get the refresh token from this response
    pub fn refresh_token(&self) -> Option<&crate::RefreshTokenRef> { self.refresh_token.as_deref() }

    /// Get the scopes from this response
    pub fn scopes(&self) -> Option<&[crate::Scope]> { self.scopes.as_deref() }
}
