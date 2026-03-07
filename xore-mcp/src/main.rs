mod error;
mod helpers;
mod server;

use anyhow::Result;
use rmcp::ServiceExt;
use server::XoreMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // MCP 协议占用 stdout；所有日志必须写入 stderr
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("xore-mcp v{} starting", env!("CARGO_PKG_VERSION"));

    let service = XoreMcpServer::new()
        .serve(rmcp::transport::stdio())
        .await
        .inspect_err(|e| tracing::error!("server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
