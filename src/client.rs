//! Provides different http clients
//!

// This module is heavily inspired (read: copied) by twitch_api2::client.

use std::error::Error;
use std::future::Future;

/// The User-Agent `product` of this crate.
pub static TWITCH_OAUTH2_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// A boxed future, mimics `futures::future::BoxFuture`
type BoxedFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A client that can do OAUTH2 requests
pub trait Client<'a>: Sync + Send + 'a {
    /// Error returned by the client
    type Error: Error + Send + Sync + 'static;
    /// Send a request
    fn req(
        &'a self,
        request: crate::HttpRequest,
    ) -> BoxedFuture<'a, Result<crate::HttpResponse, <Self as Client>::Error>>;
}

#[doc(hidden)]
#[derive(Debug, thiserror::Error, Clone)]
#[error("this client does not do anything, only used for documentation test that only checks code integrity")]
pub struct DummyClient;

#[cfg(feature = "reqwest")]
#[cfg_attr(nightly, doc(cfg(feature = "reqwest_client")))] // FIXME: This doc_cfg does nothing
impl<'a> Client<'a> for DummyClient {
    type Error = DummyClient;

    fn req(
        &'a self,
        _: crate::HttpRequest,
    ) -> BoxedFuture<'a, Result<crate::HttpResponse, Self::Error>> {
        Box::pin(async move { Err(self.clone()) })
    }
}
#[cfg(feature = "reqwest")]
use reqwest::Client as ReqwestClient;

#[cfg(feature = "reqwest")]
#[cfg_attr(nightly, doc(cfg(feature = "reqwest_client")))] // FIXME: This doc_cfg does nothing
impl<'a> Client<'a> for ReqwestClient {
    type Error = reqwest::Error;

    fn req(
        &'a self,
        request: crate::HttpRequest,
    ) -> BoxedFuture<'a, Result<crate::HttpResponse, Self::Error>> {
        // Reqwest plays really nice here and has a try_from on `http::Request` -> `reqwest::Request`
        use std::convert::TryFrom;
        let req = match reqwest::Request::try_from(request) {
            Ok(req) => req,
            Err(e) => return Box::pin(async { Err(e) }),
        };
        // We need to "call" the execute outside the async closure to not capture self.
        let fut = self.execute(req);
        Box::pin(async move {
            // Await the request and translate to `http::Response`
            let mut response = fut.await?;
            let mut result = http::Response::builder();
            let headers = result
                .headers_mut()
                // This should not fail, we just created the response.
                .expect("expected to get headers mut when building response");
            std::mem::swap(headers, response.headers_mut());
            let result = result.version(response.version());
            Ok(result
                .body(response.bytes().await?.as_ref().to_vec())
                .expect("mismatch reqwest -> http conversion should not fail"))
        })
    }
}

#[cfg(feature = "surf")]
use surf::Client as SurfClient;

/// Possible errors from [`Client::req()`] when using the [surf](https://crates.io/crates/surf) client
#[cfg(feature = "surf")]
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum SurfError {
    /// surf failed to do the request: {0}
    Surf(surf::Error),
    /// could not construct header value
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    /// could not construct header name
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    /// uri could not be translated into an url.
    UrlError(#[from] url::ParseError),
}

#[cfg(feature = "surf")]
#[cfg_attr(nightly, doc(cfg(feature = "surf_client")))] // FIXME: This doc_cfg does nothing
impl<'a> Client<'a> for SurfClient {
    type Error = SurfError;

    fn req(
        &'a self,
        request: crate::HttpRequest,
    ) -> BoxedFuture<'a, Result<crate::HttpResponse, Self::Error>> {
        // First we translate the `http::Request` method and uri into types that surf understands.

        let method: surf::http::Method = request.method().clone().into();

        let url = match url::Url::parse(&request.uri().to_string()) {
            Ok(url) => url,
            Err(err) => return Box::pin(async move { Err(err.into()) }),
        };
        // Construct the request
        let mut req = surf::Request::new(method, url);

        // move the headers into the surf request
        for (name, value) in request.headers().iter() {
            let value =
                match surf::http::headers::HeaderValue::from_bytes(value.as_bytes().to_vec())
                    .map_err(SurfError::Surf)
                {
                    Ok(val) => val,
                    Err(err) => return Box::pin(async { Err(err) }),
                };
            req.append_header(name.as_str(), value);
        }

        // assembly the request, now we can send that to our `surf::Client`
        req.body_bytes(&request.body());

        let client = self.clone();
        Box::pin(async move {
            // Send the request and translate the response into a `http::Response`
            let mut response = client.send(req).await.map_err(SurfError::Surf)?;
            let mut result = http::Response::builder();

            let mut response_headers: http::header::HeaderMap = response
                .iter()
                .map(|(k, v)| {
                    Ok((
                        http::header::HeaderName::from_bytes(k.as_str().as_bytes())?,
                        http::HeaderValue::from_str(v.as_str())?,
                    ))
                })
                .collect::<Result<_, SurfError>>()?;

            let _ = std::mem::replace(&mut result.headers_mut(), Some(&mut response_headers));
            let result = if let Some(v) = response.version() {
                result.version(match v {
                    surf::http::Version::Http0_9 => http::Version::HTTP_09,
                    surf::http::Version::Http1_0 => http::Version::HTTP_10,
                    surf::http::Version::Http1_1 => http::Version::HTTP_11,
                    surf::http::Version::Http2_0 => http::Version::HTTP_2,
                    surf::http::Version::Http3_0 => http::Version::HTTP_3,
                    // TODO: Log this somewhere...
                    _ => http::Version::HTTP_3,
                })
            } else {
                result
            };
            Ok(result
                .body(response.body_bytes().await.map_err(SurfError::Surf)?)
                .expect("mismatch surf -> http conversion should not fail"))
        })
    }
}
