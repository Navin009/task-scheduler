use anyhow::Result;
use chrono::Duration;
use scheduler_core::{
    cache::Cache, config::Config, db::Database, models::JobStatus, task::TaskManager,
};
use std::time::Duration as StdDuration;
use task_failure_watcher::{
    alerting::{AlertManager, LogNotificationChannel},
    cleanup::CleanupManager,
    watcher::TaskFailureWatcher,
};
use tokio::signal;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod alerting;
mod cleanup;
mod watcher;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .pretty()
        .init();

    info!("Starting Task Failure Watcher");

    // Load configuration
    let config = Config::from_env()?;
    let db_config = config.database_url;
    let redis_config = config.redis_url;

    // Initialize database and cache
    let db = Database::new(&db_config).await?;
    let cache = Cache::new(redis_config).await?;
    let task_manager = TaskManager::new(db.clone());

    // Initialize alert manager
    let mut alert_manager = AlertManager::new(Duration::hours(1));
    alert_manager.add_channel(Box::new(LogNotificationChannel));

    // Initialize failure watcher with core library types
    let failure_watcher = TaskFailureWatcher::new(
        task_manager.clone(),
        cache.clone(),
        StdDuration::from_secs(60),   // Check every minute
        config.max_retries as i32,    // Use configurable max retries
        StdDuration::from_secs(60),   // Initial backoff of 1 minute
        StdDuration::from_secs(3600), // Max backoff of 1 hour
    );

    // Initialize cleanup manager
    let cleanup_manager = CleanupManager::new(
        task_manager,
        Duration::hours(1), // Cleanup every hour
        Duration::days(30), // Keep tasks for 30 days
    );

    // Start all components
    let failure_watcher_handle = tokio::spawn(async move {
        if let Err(e) = failure_watcher.start().await {
            error!("Failure watcher error: {}", e);
        }
    });

    let cleanup_manager_handle = tokio::spawn(async move {
        if let Err(e) = cleanup_manager.start().await {
            error!("Cleanup manager error: {}", e);
        }
    });

    // Handle shutdown signals
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = terminate => {
            info!("Received termination signal, shutting down...");
        }
    }

    // Wait for all components to finish
    failure_watcher_handle.abort();
    cleanup_manager_handle.abort();

    info!("Task Failure Watcher shutdown complete");
    Ok(())
}
