use tokio::signal;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::Result;

pub fn init_tracing() -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,pixel=debug,tower_http=info,hyper=warn,sea_orm=warn".into());

    Ok(tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_span_list(false)
                .with_file(false)
                .with_target(true)
                .with_line_number(false)
                .with_thread_ids(false)
                .with_thread_names(false),
        )
        .try_init()?)
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
