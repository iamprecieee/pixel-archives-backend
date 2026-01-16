use std::{net::SocketAddr, sync::Arc};

use pixel_archives::{
    AppState, RateLimiters, build_router,
    config::Config,
    error::Result,
    infrastructure::{cache::Cache, db::Database},
    middleware::rate_limit::create_limiter,
    services::{auth::JwtService, solana::SolanaClient},
    utils::server::{init_tracing, shutdown_signal},
    ws::RoomManager,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let config = Config::from_env()?;
    config.validate()?;
    tracing::info!("Configuration loaded");

    let db = Database::init_db(&config.database).await?;
    tracing::info!("Database initialized");

    db.run_migrations().await?;
    tracing::info!("Migrations completed");

    let cache = Cache::init(&config).await?;
    tracing::info!("Cache initialized");

    let jwt_service = JwtService::new(&config.jwt);
    tracing::info!("JWT service initialized");

    let solana_client = SolanaClient::initialize(&config.solana);
    tracing::info!("Solana client initialized");

    let ws_rooms = RoomManager::initialize(config.canvas.max_collaborators);
    tracing::info!("WebSocket rooms initialized");

    let rate_limit_redis_cache = Arc::new(cache.redis.clone());

    let rate_limiters = RateLimiters {
        pixel: create_limiter(
            rate_limit_redis_cache.clone(),
            config.rate_limit.pixel_limit,
            "pixel",
        ),
        auth: create_limiter(
            rate_limit_redis_cache.clone(),
            config.rate_limit.auth_limit,
            "auth",
        ),
        canvas: create_limiter(
            rate_limit_redis_cache.clone(),
            config.rate_limit.canvas_limit,
            "canvas",
        ),
        solana: create_limiter(
            rate_limit_redis_cache.clone(),
            config.rate_limit.solana_limit,
            "solana",
        ),
    };

    let state = AppState {
        config: Arc::new(config.clone()),
        db: Arc::new(db),
        cache: Arc::new(cache),
        jwt_service: Arc::new(jwt_service),
        solana_client: Arc::new(solana_client),
        ws_rooms: Arc::new(ws_rooms),
        rate_limiters: Arc::new(rate_limiters),
    };

    let app = build_router(state);

    let server_addr = format!("{}:{}", config.server.host, config.server.port);

    let listener = TcpListener::bind(server_addr).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    tracing::info!("Server shutdown complete");

    Ok(())
}
