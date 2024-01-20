use crate::AppState;
use aws_smithy_http_server::body::BoxBody;
use axum::http::{Request, Response, StatusCode};
use echo_server_sdk::server::response::IntoResponse;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;
use tower::{Layer, Service};

/// The server request ID has not been added to the [`Request`](http::Request) or has been previously removed.
#[non_exhaustive]
#[derive(Debug, Error)]
#[error("the `Authorization` header is not present in the `http::Request`")]
pub enum BearTokenError {
    #[error("the `Authorization` header is not present in the `http::Request`")]
    Missing,
    #[error("the `Authorization` header is not valid")]
    Invalid,
}

#[derive(Clone)]
pub struct BearerTokenProvider<S> {
    inner: S,
}

/// A layer that provides services with bearer token
#[derive(Debug)]
#[non_exhaustive]
pub struct BearerTokenProviderLayer {}

impl BearerTokenProviderLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for BearerTokenProviderLayer {
    type Service = BearerTokenProvider<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BearerTokenProvider { inner }
    }
}

impl<Body, S> Service<Request<Body>> for BearerTokenProvider<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>>,
    S::Future: std::marker::Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Send + Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match self.process(req) {
            Ok(req) => {
                let fut = self.inner.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(e) => {
                let res = <BearTokenError as IntoResponse<()>>::into_response(e);
                Box::pin(async move { Ok(res) })
            }
        }
    }
}

impl<S> BearerTokenProvider<S> {
    fn process<Body>(&self, mut req: Request<Body>) -> Result<Request<Body>, BearTokenError> {
        // TODO: how to read the smithy auth trait to see if the auth is required?
        let path = req.uri().path();
        if path.starts_with("/signin") || path.starts_with("/echo") {
            return Ok(req);
        }

        let v = req
            .headers_mut()
            .remove("Authorization")
            .ok_or(BearTokenError::Missing)?;
        let v = v.to_str().map_err(|_| BearTokenError::Invalid)?;
        let token = v.trim_start_matches("Bearer ").to_string();

        let verifier = &req.extensions().get::<Arc<AppState>>().unwrap().verifier;
        match verifier.verify(token) {
            Ok(claim) => {
                req.extensions_mut().insert(claim);

                Ok(req)
            }
            Err(_) => Err(BearTokenError::Invalid),
        }
    }
}

impl<Protocol> IntoResponse<Protocol> for BearTokenError {
    fn into_response(self) -> Response<BoxBody> {
        match self {
            BearTokenError::Missing => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(BoxBody::default())
                .unwrap(),
            BearTokenError::Invalid => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(BoxBody::default())
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {}
