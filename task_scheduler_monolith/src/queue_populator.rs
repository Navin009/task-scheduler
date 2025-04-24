use anyhow::Result;
use log::info;
use queue_populator::start_populator;
use tokio::sync::broadcast;

pub async fn start_populator_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting Queue Populator service");
    
    // Start the queue populator
    let populator = start_populator().await?;
    
    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Queue Populator received shutdown signal");
        }
    }
    
    // Gracefully shutdown the populator
    populator.shutdown().await;
    info!("Queue Populator shutdown complete");
    Ok(())
} 