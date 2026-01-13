use std::{net::SocketAddr, sync::Arc};

use pixel_archives::{
    AppState, build_router,
    config::Config,
    error::Result,
    infrastructure::{cache::Cache, db::Database},
    services::{auth::JwtService, solana::SolanaClient},
    shutdown_signal,
    ws::RoomManager,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "pixel_archives=info,sqlx=error,sea_orm_migration=error".into()
            }),
        )
        .init();

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

    let state = AppState {
        config: Arc::new(config.clone()),
        db: Arc::new(db),
        cache: Arc::new(cache),
        jwt_service: Arc::new(jwt_service),
        solana_client: Arc::new(solana_client),
        ws_rooms: Arc::new(ws_rooms),
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
