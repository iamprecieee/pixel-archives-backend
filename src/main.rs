use pixel_archives::{config::Config, db::Database, error::Result};

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
    tracing::info!("Configuration loaded");

    let db = Database::init_db(&config.database).await?;
    tracing::info!("Database connected");

    db.run_migrations().await?;
    tracing::info!("Migrations completed");

    Ok(())
}
