use chrono::{Duration, Utc};
use chrono_tz::UTC;
use scheduler_core::{cache::RedisClient, config::Config, db::Database};
use task_recurrence_manager::RecurrenceManager;
use tokio::time;
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

    // Create recurrence manager
    let manager = RecurrenceManager::new(
        db,
        cache,
        UTC,                 // Using UTC as default timezone
        Duration::hours(24), // Look ahead 24 hours
    )
    .await?;

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
