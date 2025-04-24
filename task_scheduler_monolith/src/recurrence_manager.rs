use anyhow::Result;
use log::info;
use task_recurrence_manager::start_manager;
use tokio::sync::broadcast;

pub async fn start_recurrence_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting Recurrence Manager service");
    
    // Start the recurrence manager
    let manager = start_manager().await?;
    
    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Recurrence Manager received shutdown signal");
        }
    }
    
    // Gracefully shutdown the manager
    manager.shutdown().await;
    info!("Recurrence Manager shutdown complete");
    Ok(())
} 