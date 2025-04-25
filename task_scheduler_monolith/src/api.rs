use anyhow::Result;
use log::info;
use task_scheduler_api::Server;
use tokio::sync::broadcast;

pub async fn start_api_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting API service");

    // Start the API server
    let server = Server::new().await?;
    let server_handle = server.start().await?;

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("API service received shutdown signal");
        }
    }

    // Gracefully shutdown the server
    server_handle.shutdown().await?;
    info!("API service shutdown complete");
    Ok(())
}
