//! Provides different http clients

// This module is heavily inspired (read: copied) by twitch_api2::client.

use futures::future::Either;
use std::error::Error;
use std::future::Future;

/// The User-Agent `product` of this crate.
pub static TWITCH_OAUTH2_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// A client that can do OAUTH2 requests
pub trait Client: Sync + Send {
    /// Error returned by the client
    type Error: Error + Send + Sync + 'static;
    /// Send a request
    fn req(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> impl Future<Output = Result<http::Response<Vec<u8>>, <Self as Client>::Error>> + Send + use<Self>;
}

#[doc(hidden)]
#[derive(Debug, thiserror::Error, Clone)]
#[error("this client does not do anything, only used for documentation test that only checks code integrity")]
pub struct DummyClient;

#[cfg(feature = "reqwest")]
impl Client for DummyClient {
    type Error = DummyClient;

    fn req(
        &self,
        _: http::Request<Vec<u8>>,
    ) -> impl Future<Output = Result<http::Response<Vec<u8>>, Self::Error>> + Send + use<> {
        std::future::ready(Err(self.clone()))
    }
}
#[cfg(feature = "reqwest")]
use reqwest::Client as ReqwestClient;

#[cfg(feature = "reqwest")]
impl Client for ReqwestClient {
    type Error = reqwest::Error;

    fn req(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> impl Future<Output = Result<http::Response<Vec<u8>>, Self::Error>> + Send + use<> {
        // Reqwest plays really nice here and has a try_from on `http::Request` -> `reqwest::Request`
        let req = match reqwest::Request::try_from(request) {
            Ok(req) => req,
            Err(e) => return Either::Right(async move { Err(e) }),
        };
        // We need to "call" the execute outside the async closure to not capture self.
        let fut = self.execute(req);
        // Await the request and translate to `http::Response`
        let fut = async move {
            let mut response = fut.await?;
            let mut result = http::Response::builder().status(response.status());
            let headers = result
                .headers_mut()
                // This should not fail, we just created the response.
                .expect("expected to get headers mut when building response");
            std::mem::swap(headers, response.headers_mut());
            let result = result.version(response.version());
            Ok(result
                .body(response.bytes().await?.as_ref().to_vec())
                .expect("mismatch reqwest -> http conversion should not fail"))
        };
        Either::Left(fut)
    }
}

#[cfg(all(feature = "reqwest", test))]
mod tests {
    use super::*;

    #[test]
    /// Test that the returned future does not depend on the lifetime of `self`
    fn reqwest_capture() {
        fn inner() -> impl Future<Output = Result<http::Response<Vec<u8>>, reqwest::Error>> + Send {
            let client = ReqwestClient::new();
            client.req(http::Request::new(vec![]))
        }
        let _fut = inner();
    }
}
