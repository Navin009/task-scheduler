use anyhow::Result;
use log::{error, info};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

mod api;
mod executor;
mod failure_watcher;
mod queue_populator;
mod recurrence_manager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting Task Scheduler Monolith");

    // Create a shutdown channel
    let (shutdown_tx, _) = broadcast::channel(1);
    let shutdown_rx = shutdown_tx.subscribe();

    // Start all services
    let mut handles: Vec<JoinHandle<Result<()>>> = Vec::new();

    // Start API service
    let api_handle = tokio::spawn(api::start_api_service(shutdown_rx.resubscribe()));
    handles.push(api_handle);

    // Start Task Executor
    let executor_handle = tokio::spawn(executor::start_executor_service(shutdown_rx.resubscribe()));
    handles.push(executor_handle);

    // Start Failure Watcher
    let watcher_handle = tokio::spawn(failure_watcher::start_watcher_service(shutdown_rx.resubscribe()));
    handles.push(watcher_handle);

    // Start Recurrence Manager
    let recurrence_handle = tokio::spawn(recurrence_manager::start_recurrence_service(shutdown_rx.resubscribe()));
    handles.push(recurrence_handle);

    // Start Queue Populator
    let populator_handle = tokio::spawn(queue_populator::start_populator_service(shutdown_rx.resubscribe()));
    handles.push(populator_handle);

    // Handle shutdown signals
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
            let _ = shutdown_tx.send(());
        }
    }

    // Wait for all services to shutdown
    for handle in handles {
        if let Err(e) = handle.await {
            error!("Error in service shutdown: {}", e);
        }
    }

    info!("Task Scheduler Monolith shutdown complete");
    Ok(())
} 