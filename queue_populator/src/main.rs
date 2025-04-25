use anyhow::Result;
use scheduler_core::{
    cache::{Cache, CacheConfig},
    config::Config,
};
use sqlx::postgres::PgPool;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info};

mod config;
mod error;
mod job_processor;

use config::QueuePopulatorConfig;
use job_processor::JobProcessor;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let core_config = Config::from_env()?;
    let config = QueuePopulatorConfig::from_core_config(&core_config);

    // Initialize database connection
    let db_pool = PgPool::connect(&config.database_url).await?;

    // Initialize cache
    let cache_config = CacheConfig {
        url: config.cache_url,
        max_connections: config.max_connections,
    };
    let cache = Cache::new(cache_config).await?;

    // Initialize job processor
    let job_processor = JobProcessor::new(db_pool, cache, &config.database_url).await?;

    info!("Queue populator service started");

    // Main loop with graceful shutdown
    let mut shutdown = false;
    while !shutdown {
        match job_processor.process_jobs().await {
            Ok(_) => {
                // Sleep for configured interval before next iteration
                sleep(Duration::from_secs(config.poll_interval_seconds)).await;
            }
            Err(e) => {
                error!("Error processing jobs: {}", e);
                // Sleep for a shorter interval on error to prevent tight loops
                sleep(Duration::from_secs(1)).await;
            }
        }

        // Check for shutdown signal
        if let Ok(_) = signal::ctrl_c().await {
            info!("Received shutdown signal, stopping gracefully...");
            shutdown = true;
        }
    }

    info!("Queue populator service stopped");
    Ok(())
}
