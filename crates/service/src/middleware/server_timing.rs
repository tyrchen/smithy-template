// code from: https://github.com/JensWalter/axum-server-timing/blob/main/src/lib.rs

use std::{
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
    time::Instant,
};

use axum::http::{HeaderValue, Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct ServerTimingLayer<'a> {
    app: &'a str,
    description: Option<&'a str>,
}

impl<'a> ServerTimingLayer<'a> {
    pub fn new(app: &'a str) -> Self {
        ServerTimingLayer {
            app,
            description: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_description(&mut self, description: &'a str) -> Self {
        let mut new_self = self.clone();
        new_self.description = Some(description);
        new_self
    }
}

impl<'a, S> Layer<S> for ServerTimingLayer<'a> {
    type Service = ServerTimingService<'a, S>;

    fn layer(&self, service: S) -> Self::Service {
        ServerTimingService {
            service,
            app: self.app,
            description: self.description,
        }
    }
}

#[derive(Clone)]
pub struct ServerTimingService<'a, S> {
    service: S,
    app: &'a str,
    description: Option<&'a str>,
}

impl<'a, S, ReqBody, ResBody> Service<Request<ReqBody>> for ServerTimingService<'a, S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<'a, S::Future>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (parts, body) = req.into_parts();

        let req = Request::from_parts(parts, body);
        ResponseFuture {
            inner: self.service.call(req),
            request_time: Instant::now(),
            app: self.app,
            description: self.description,
        }
    }
}

pin_project! {
  pub struct ResponseFuture<'a, F> {
      #[pin]
      inner: F,
      request_time: Instant,
      app: &'a str,
      description: Option<&'a str>,
  }
}

impl<F, B, E> Future for ResponseFuture<'_, F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Default,
{
    type Output = Result<Response<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let time = self.request_time;
        let app = self.app;
        let description = self.description;
        let mut response: Response<B> = ready!(self.project().inner.poll(cx))?;
        let hdr = response.headers_mut();
        let x = time.elapsed().as_millis();
        let header_value = match description {
            Some(val) => format!("{app};desc=\"{val}\";dur={x}"),
            None => format!("{app};dur={x}"),
        };
        match hdr.try_entry("Server-Timing") {
            Ok(entry) => {
                match entry {
                    axum::http::header::Entry::Occupied(mut val) => {
                        //has val
                        let old_val = val.get();
                        let new_val = format!("{header_value}, {}", old_val.to_str().unwrap());
                        val.insert(HeaderValue::from_str(&new_val).unwrap());
                    }
                    axum::http::header::Entry::Vacant(val) => {
                        val.insert(HeaderValue::from_str(&header_value).unwrap());
                    }
                }
            }
            Err(_) => {
                hdr.append(
                    "Server-Timing",
                    HeaderValue::from_str(&header_value).unwrap(),
                );
            }
        }

        Poll::Ready(Ok(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{HeaderMap, HeaderValue},
        routing::get,
        Router,
    };
    use std::{net::SocketAddr, time::Duration};

    #[test]
    fn service_name() {
        let name = "svc1";
        let obj = ServerTimingLayer::new(name);
        assert_eq!(obj.app, name);
    }

    #[test]
    fn service_desc() {
        let name = "svc1";
        let desc = "desc1";
        let obj = ServerTimingLayer::new(name).with_description(desc);
        assert_eq!(obj.app, name);
        assert_eq!(obj.description, Some(desc));
    }

    #[tokio::test]
    async fn header_exists_on_response() {
        let name = "svc1";
        let app = Router::new()
            .route("/", get(|| async move { "" }))
            .layer(ServerTimingLayer::new(name));

        tokio::spawn(async {
            let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap()
        });
        //test request
        let resp = reqwest::get("http://localhost:3001/").await.unwrap();
        let hdr = resp.headers().get("server-timing");
        assert!(hdr.is_some());
    }

    #[tokio::test]
    async fn header_value() {
        let name = "svc1";
        let app = Router::new()
            .route(
                "/",
                get(|| async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    ""
                }),
            )
            .layer(ServerTimingLayer::new(name));

        tokio::spawn(async {
            let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap()
        });

        //test request
        let resp = reqwest::get("http://localhost:3002/").await.unwrap();
        if let Some(hdr) = resp.headers().get("server-timing") {
            let val = &hdr.to_str().unwrap()[9..];
            let val_num: f32 = val.parse().unwrap();
            assert!(val_num >= 100_f32);
        } else {
            panic!("no header found");
        }
    }

    #[tokio::test]
    async fn support_existing_header() {
        let name = "svc1";
        let app = Router::new()
            .route(
                "/",
                get(|| async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let mut hdr = HeaderMap::new();
                    hdr.insert("server-timing", HeaderValue::from_static("inner;dur=23"));
                    (hdr, "")
                }),
            )
            .layer(ServerTimingLayer::new(name));

        tokio::spawn(async {
            let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap()
        });

        //test request
        let resp = reqwest::get("http://localhost:3003/").await.unwrap();
        let hdr = resp.headers().get("server-timing").unwrap();
        let hdr_str = hdr.to_str().unwrap();
        assert!(hdr_str.contains("svc1"));
        assert!(hdr_str.contains("inner"));
        println!("{hdr:?}");
    }
}
