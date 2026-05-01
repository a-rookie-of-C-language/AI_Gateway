use anyhow::Result;

mod application;
mod bootstrap;
mod config;
mod domain;
mod infrastructure;
mod interfaces;
mod shared;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let app = bootstrap::build_app::build_app().await?;
    app.run().await
}
