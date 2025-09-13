use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tracing::info;

mod config;
mod proxy;
mod filter;
mod classifier;
mod ui;
mod db;

use config::Config;

#[derive(Parser)]
#[command(author, version, about = "NoRot - Social Media Content Filter", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Port to run the proxy server on
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(log_level))
        .init();
    
    info!("Starting NoRot social media content filter");
    
    // Load configuration
    let config = Config::load(&args.config)?;
    
    // Initialize database
    let db_pool = db::init_database().await?;
    
    // Create application state
    let app_state = proxy::AppState::new(config, db_pool);
    
    // Build the application
    let app = proxy::create_app(app_state).await?;
    
    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
