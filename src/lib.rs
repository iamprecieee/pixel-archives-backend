pub mod api;
pub mod config;
pub mod error;
pub mod infrastructure;
pub mod middleware;
pub mod services;
pub mod utils;
pub mod ws;

use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::{
    api::nft_metadata,
    config::Config,
    infrastructure::{cache::Cache, db::Database},
    middleware::rate_limit::RateLimiter,
    services::{auth::JwtService, solana::SolanaClient},
};

#[derive(Clone)]
pub struct RateLimiters {
    pub pixel: RateLimiter,
    pub auth: RateLimiter,
    pub canvas: RateLimiter,
    pub solana: RateLimiter,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub cache: Arc<Cache>,
    pub jwt_service: Arc<JwtService>,
    pub solana_client: Arc<SolanaClient>,
    pub ws_rooms: Arc<ws::RoomManager>,
    pub rate_limiters: Arc<RateLimiters>,
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state
                .config
                .server
                .cors_allowed_origins
                .iter()
                .filter_map(|origin| origin.parse().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true);

    Router::new()
        .nest("/api", api::router())
        .nest("/nft", nft_metadata::router())
        .nest("/ws", ws::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(middleware::logging::make_log_span)
                .on_request(())
                .on_eos(()),
        )
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(ConcurrencyLimitLayer::new(
            state.config.server.max_concurrent_requests,
        ))
        .with_state(state)
}
