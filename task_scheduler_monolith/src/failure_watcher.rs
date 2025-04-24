use anyhow::Result;
use log::info;
use task_failure_watcher::start_watcher;
use tokio::sync::broadcast;

pub async fn start_watcher_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting Failure Watcher service");
    
    // Start the failure watcher
    let watcher = start_watcher().await?;
    
    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Failure Watcher received shutdown signal");
        }
    }
    
    // Gracefully shutdown the watcher
    watcher.shutdown().await;
    info!("Failure Watcher shutdown complete");
    Ok(())
} 