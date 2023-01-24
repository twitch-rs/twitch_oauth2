#![allow(clippy::extra_unused_lifetimes)]
//! Types used in OAUTH2 flow.

use std::fmt;

use base64::Engine;

/// A Client Id
#[aliri_braid::braid(serde)]
pub struct ClientId;

/// A Client Secret
#[aliri_braid::braid(display = "owned", debug = "owned", serde)]
pub struct ClientSecret;

impl fmt::Debug for ClientSecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted client secret]")
    }
}
impl fmt::Display for ClientSecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted client secret]")
    }
}

/// An Access Token
#[aliri_braid::braid(display = "owned", debug = "owned", serde)]
pub struct AccessToken;

impl fmt::Debug for AccessTokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted access token]")
    }
}
impl fmt::Display for AccessTokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted access token]")
    }
}

/// A Refresh Token
#[aliri_braid::braid(display = "owned", debug = "owned", serde)]
pub struct RefreshToken;

impl fmt::Debug for RefreshTokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted refresh token]")
    }
}
impl fmt::Display for RefreshTokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted refresh token]")
    }
}

/// A Csrf Token
#[aliri_braid::braid(display = "owned", debug = "owned", serde)]
pub struct CsrfToken;

impl fmt::Debug for CsrfTokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted csrf token]")
    }
}
impl fmt::Display for CsrfTokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted csrf token]")
    }
}

impl CsrfToken {
    /// Make a new random CSRF token.
    pub fn new_random() -> CsrfToken { Self::new_random_len(16) }

    /// Make a new random CSRF token with given amount of bytes
    pub fn new_random_len(len: u32) -> CsrfToken {
        use rand::Rng as _;
        let random_bytes: Vec<u8> = (0..len).map(|_| rand::thread_rng().gen::<u8>()).collect();
        CsrfToken::new(base64::engine::general_purpose::STANDARD.encode(random_bytes))
    }
}

impl ClientSecretRef {
    /// Get the secret from this string.
    ///
    /// This function is the same as [`ClientSecret::as_str`](ClientSecretRef::as_str), but has another name for searchability, prefer to use this function.
    pub fn secret(&self) -> &str { self.as_str() }
}

impl AccessTokenRef {
    /// Get the secret from this string.
    ///
    /// This function is the same as [`AccessToken::as_str`](AccessTokenRef::as_str), but has another name for searchability, prefer to use this function.
    pub fn secret(&self) -> &str { self.as_str() }
}
impl RefreshTokenRef {
    /// Get the secret from this string.
    ///
    /// This function is the same as [`RefreshToken::as_str`](RefreshTokenRef::as_str), but has another name for searchability, prefer to use this function.
    pub fn secret(&self) -> &str { self.as_str() }
}
impl CsrfTokenRef {
    /// Get the secret from this string.
    ///
    /// This function is the same as [`CsrfToken::as_str`](CsrfTokenRef::as_str), but has another name for searchability, prefer to use this function.
    pub fn secret(&self) -> &str { self.as_str() }
}
