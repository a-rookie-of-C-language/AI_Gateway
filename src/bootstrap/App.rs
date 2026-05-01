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
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}
