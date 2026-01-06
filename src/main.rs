use pixel_archives::{config::Config, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let _config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    Ok(())
}
