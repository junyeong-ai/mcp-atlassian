mod config;
mod mcp;
mod tools;
mod utils;

use anyhow::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr
    utils::logging::init_logging();

    // Load configuration
    let config = config::Config::from_env()?;
    config.validate()?;

    utils::logging::log_startup(&config);

    // Create and run MCP server
    let server = mcp::server::McpServer::new(config).await?;

    // Run server with graceful shutdown
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Received interrupt signal, shutting down...");
        }
    }

    utils::logging::log_shutdown();
    Ok(())
}