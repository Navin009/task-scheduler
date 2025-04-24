use scheduler_core::{cache::RedisClient, config::Config, db::Database};
use std::time::Duration;
use task_executor::TaskExecutor;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::load()?;

    // Initialize database and cache
    let db = Database::new(&config.database_url).await?;
    let cache = RedisClient::new(&config.redis_url)?;

    // Create task executor
    let executor = TaskExecutor::new(
        db,
        cache,
        Duration::from_secs(300), // 5 minute timeout
        1024,                     // 1GB memory limit
        50,                       // 50% CPU limit
        10,                       // 10 concurrent jobs
    )
    .await?;

    info!("Starting task executor");

    // Start the executor
    if let Err(e) = executor.start().await {
        error!("Task executor failed: {}", e);
        return Err(e.into());
    }

    Ok(())
}
