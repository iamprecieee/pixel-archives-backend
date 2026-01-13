pub mod api;
pub mod config;
pub mod error;
pub mod infrastructure;
pub mod services;
pub mod ws;

use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
};
use tokio::signal;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::CorsLayer;

use crate::{
    config::Config,
    infrastructure::{cache::Cache, db::Database},
    services::{auth::JwtService, solana::SolanaClient},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub cache: Arc<Cache>,
    pub jwt_service: Arc<JwtService>,
    pub solana_client: Arc<SolanaClient>,
    pub ws_rooms: Arc<ws::RoomManager>,
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
        .nest("/ws", ws::router())
        .layer(cors)
        .layer(ConcurrencyLimitLayer::new(
            state.config.server.max_concurrent_requests,
        ))
        .with_state(state)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        signal(SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::warn!("Received Ctrl+C, initiating shutdown");
        }
        _ = terminate => {
            tracing::warn!("Received SIGTERM, initiating shutdown");
        }
    }
}
