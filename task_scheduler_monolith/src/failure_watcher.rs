use anyhow::Result;
use log::info;
use task_failure_watcher::Watcher;
use tokio::sync::broadcast;

pub async fn start_watcher_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting Failure Watcher service");

    // Start the failure watcher
    let watcher = Watcher::new().await?;
    let watcher_handle = watcher.start().await?;

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Failure Watcher received shutdown signal");
        }
    }

    // Gracefully shutdown the watcher
    watcher_handle.shutdown().await?;
    info!("Failure Watcher shutdown complete");
    Ok(())
}
