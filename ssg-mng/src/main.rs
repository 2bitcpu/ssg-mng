use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt};

use application::UseCaseModuleImpl;
use common::types::BoxError;
use config::CONFIG;
use domain::Repositories;
use infrastructure::RepositoriesImpl;
use presentation::create_router;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), BoxError> {
    if let Some(ref level) = CONFIG.log.level {
        let filter = EnvFilter::new(level);
        fmt().with_env_filter(filter).init();
    }

    let repos = RepositoriesImpl::new()?;
    let repos: Arc<dyn Repositories> = Arc::new(repos);
    let usecase = UseCaseModuleImpl::new(repos.clone());

    let app = create_router(Arc::new(usecase));

    let listener = tokio::net::TcpListener::bind(CONFIG.server.host.clone()).await?;
    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
