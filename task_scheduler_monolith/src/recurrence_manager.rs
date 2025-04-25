use anyhow::Result;
use log::info;
use task_recurrence_manager::Manager;
use tokio::sync::broadcast;

pub async fn start_recurrence_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting Recurrence Manager service");

    // Start the recurrence manager
    let manager = Manager::new().await?;
    let manager_handle = manager.start().await?;

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Recurrence Manager received shutdown signal");
        }
    }

    // Gracefully shutdown the manager
    manager_handle.shutdown().await?;
    info!("Recurrence Manager shutdown complete");
    Ok(())
}
