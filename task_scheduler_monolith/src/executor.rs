use anyhow::Result;
use log::info;
use task_executor::Executor;
use tokio::sync::broadcast;

pub async fn start_executor_service(mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Starting Task Executor service");

    // Start the executor
    let executor = Executor::new().await?;
    let executor_handle = executor.start().await?;

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Task Executor received shutdown signal");
        }
    }

    // Gracefully shutdown the executor
    executor_handle.shutdown().await?;
    info!("Task Executor shutdown complete");
    Ok(())
}
