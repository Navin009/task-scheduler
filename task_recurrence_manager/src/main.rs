use chrono::Duration;
use chrono_tz::UTC;
use scheduler_core::{cache::Cache, config::Config, db::Database};
use task_recurrence_manager::RecurrenceManager;
use tokio::time;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize database and cache
    let db = Database::new(&config.database_url).await?;
    let cache_config = scheduler_core::cache::CacheConfig {
        url: config.redis_url.clone(),
        max_connections: 10,
    };
    let cache = Cache::new(cache_config).await?;

    // Create recurrence manager
    let manager = RecurrenceManager::new(db, cache, UTC, Duration::hours(24)).await?;

    info!("Starting task recurrence manager");

    // Main processing loop
    let mut interval = time::interval(time::Duration::from_secs(60));
    loop {
        interval.tick().await;

        if let Err(e) = manager.process_templates().await {
            error!("Error processing templates: {}", e);
        }

        // Check for daylight saving transitions
        if let Err(e) = manager.handle_daylight_saving_transition().await {
            error!("Error handling daylight saving transition: {}", e);
        }
    }
}
