mod api;
mod auth;
mod error;
mod middleware;

use auth::{AuthConfig, AuthSigner, AuthVerifier};
use aws_smithy_http_server::{
    plugin::IdentityPlugin, request::request_id::ServerRequestIdProviderLayer, AddExtensionLayer,
};
use axum::{
    http::{HeaderName, Method},
    response::Html,
    routing::get,
    Router,
};
use axum_swagger_ui::swagger_ui;
use derive_more::Debug;
use echo_server_sdk::{EchoService, EchoServiceConfig};
use middleware::{BearerTokenProviderLayer, ServerTimingLayer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug)]
pub struct AppState {
    #[allow(dead_code)]
    config: AppConfig,
    pub(crate) verifier: AuthVerifier,
    #[allow(dead_code)]
    pub(crate) signer: AuthSigner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_name: String,
    pub port: u16,
    pub auth: AuthConfig,
}

pub async fn get_router(conf: AppConfig) -> Router {
    // make name with static lifetime
    let name = Box::leak(Box::new(conf.server_name.clone()));

    let state = Arc::new(AppState::new(conf));

    let config = EchoServiceConfig::builder()
        // IdentityPlugin is a plugin that adds a middleware to the service, it just shows how to use plugins
        .http_plugin(IdentityPlugin)
        .layer(AddExtensionLayer::new(state.clone()))
        .layer(BearerTokenProviderLayer::new())
        .layer(ServerRequestIdProviderLayer::new_with_response_header(
            HeaderName::from_static("x-request-id"),
        ))
        .build();
    let api = EchoService::builder(config)
        .echo_message(api::echo_message)
        .signin(api::signin)
        .build()
        .expect("failed to build an instance of Echo Service");

    let doc_url = "/swagger/openapi.json";
    let doc = include_str!("../../../smithy/build/smithy/source/openapi/EchoService.openapi.json");

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        // allow requests from any origin
        .allow_origin(Any)
        .allow_private_network(true);

    Router::new()
        .route("/swagger", get(|| async { Html(swagger_ui(doc_url)) }))
        .route(doc_url, get(move || async move { doc }))
        .nest_service("/api/", api)
        .layer(ServerTimingLayer::new(name))
        .layer(cors)
        .with_state(state)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_name: "echo-service".to_string(),
            port: 3000,
            auth: AuthConfig::default(),
        }
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let signer = AuthSigner::try_new(&config.server_name, &config.auth.sk).unwrap();
        let verifier = AuthVerifier::try_new(&config.server_name, &config.auth.pk).unwrap();
        Self {
            config,
            verifier,
            signer,
        }
    }
}
