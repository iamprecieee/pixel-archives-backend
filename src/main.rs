use pixel_archives::{
    config::Config,
    error::Result,
    infrastructure::{cache::Cache, db::Database},
};

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

    let _cache = Cache::init(&config).await?;
    tracing::info!("Cache initialized");

    Ok(())
}
