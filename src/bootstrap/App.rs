use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;

pub struct App {
    pub addr: String,
    pub router: Router,
}

impl App {
    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        tracing::info!("aigateway listening on {}", self.addr);
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
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
        _ = ctrl_c => { tracing::info!("received Ctrl+C, starting graceful shutdown"); },
        _ = terminate => { tracing::info!("received SIGTERM, starting graceful shutdown"); },
    }
}
