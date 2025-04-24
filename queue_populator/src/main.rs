use anyhow::Result;
use scheduler_core::{
    cache::{Cache, CacheConfig},
    config::Config,
    models::{Job, JobType},
};
use sqlx::postgres::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::load()?;

    // Initialize database connection
    let db_pool = PgPool::connect(&config.database.url).await?;

    // Initialize cache (Valkey)
    let cache_config = CacheConfig {
        url: config.cache.url,
        max_connections: config.cache.max_connections,
    };
    let cache = Cache::new(cache_config).await?;

    info!("Queue populator service started");

    // Main loop
    loop {
        if let Err(e) = process_jobs(&db_pool, &cache).await {
            error!("Error processing jobs: {}", e);
        }

        // Sleep for a short duration before next iteration
        sleep(Duration::from_secs(1)).await;
    }
}

async fn process_jobs(db_pool: &PgPool, cache: &Cache) -> Result<()> {
    // Fetch due jobs from database
    let due_jobs = fetch_due_jobs(db_pool).await?;

    for job in due_jobs {
        // Push job to cache queue
        if let Err(e) = push_job_to_queue(cache, &job).await {
            error!("Failed to push job {} to queue: {}", job.id, e);
            continue;
        }

        // Update job status in database
        update_job_status(db_pool, &job).await?;
    }

    Ok(())
}

async fn fetch_due_jobs(db_pool: &PgPool) -> Result<Vec<Job>> {
    // Query to fetch jobs that are due for execution
    let jobs = sqlx::query_as!(
        Job,
        r#"
        SELECT * FROM jobs 
        WHERE status = 'pending' 
        AND scheduled_at <= NOW()
        AND (
            (job_type = 'one_time' AND parent_job_id IS NULL)
            OR (job_type = 'recurring' AND parent_job_id IS NOT NULL)
            OR (job_type = 'polling' AND attempts < max_attempts)
        )
        ORDER BY priority DESC, scheduled_at ASC
        LIMIT 100
        "#
    )
    .fetch_all(db_pool)
    .await?;

    Ok(jobs)
}

async fn push_job_to_queue(cache: &Cache, job: &Job) -> Result<()> {
    // Serialize job to JSON
    let job_json = serde_json::to_string(job)?;

    // Push to appropriate queue based on priority
    let queue_name = format!("jobs:{}", job.priority);
    cache.push_to_queue(&queue_name, &job_json).await?;

    Ok(())
}

async fn update_job_status(db_pool: &PgPool, job: &Job) -> Result<()> {
    // Update job status to queued
    sqlx::query!(
        r#"
        UPDATE jobs 
        SET status = 'queued',
            updated_at = NOW()
        WHERE id = $1
        "#,
        job.id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
