// src/lib.rs

// Declare modules
pub mod cache;
pub mod db;
pub mod error;
pub mod service;
pub mod task;
pub mod utils;

// Re-export key types and functions for easier use by consumers of the crate
pub use error::{Result, SchedulerError};
pub use task::{Task, TaskStatus};

// Example of re-exporting connection functions directly
pub use cache::connect_redis;
pub use db::connect_postgres;

// --- Placeholder for Core Scheduler Logic ---
// This is where the main loop, task fetching, execution,
// and state management logic would reside.

/*
use chrono::Utc;
use db::postgres::PgPool;
use store::redis::ConnectionManager;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, instrument}; // Example logging

pub struct Scheduler {
    pg_pool: PgPool,
    redis_conn: ConnectionManager,
    // other config like poll interval, lock TTL etc.
    poll_interval: Duration,
    lock_ttl_ms: usize,
}

impl Scheduler {
    pub async fn new(database_url: &str, redis_url: &str) -> Result<Self> {
        let pg_pool = db::connect_postgres(database_url).await?;
        let redis_conn = store::connect_redis(redis_url).await?;
        Ok(Scheduler {
            pg_pool,
            redis_conn,
            poll_interval: Duration::from_secs(5), // Default poll interval
            lock_ttl_ms: 30000, // Default lock TTL: 30 seconds
        })
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<()> {
        info!("Scheduler started.");
        loop {
            if let Err(e) = self.tick().await {
                error!("Scheduler tick failed: {}", e);
                // Decide on error handling: continue, panic, exponential backoff?
            }
            sleep(self.poll_interval).await;
        }
        // info!("Scheduler stopped."); // This line might be unreachable in a loop {}
        // Ok(())
    }

    #[instrument(skip(self))]
    async fn tick(&mut self) -> Result<()> {
        let now = Utc::now();
        let due_tasks = db::get_due_tasks(&self.pg_pool, now, 10).await?; // Fetch up to 10 due tasks

        info!("Found {} due tasks", due_tasks.len());

        for task in due_tasks {
            let task_id = task.id;
            info!("Processing task: {} ({})", task.name, task_id);

            // 1. Try to acquire lock
            let lock_acquired = store::acquire_lock(&mut self.redis_conn, task_id, self.lock_ttl_ms).await?;

            if lock_acquired {
                info!("Acquired lock for task {}", task_id);
                // Use tokio::spawn to run the task concurrently without blocking the main loop
                let pool_clone = self.pg_pool.clone();
                let mut redis_clone = self.redis_conn.clone(); // Clone connection manager

                tokio::spawn(async move {
                    if let Err(e) = Self::execute_task(pool_clone, &mut redis_clone, task).await {
                         error!("Failed to execute task {}: {}", task_id, e);
                         // Error handling is done within execute_task (updating status etc)
                    }
                    // Release lock regardless of outcome (within execute_task or here)
                    if let Err(e) = store::release_lock(&mut redis_clone, task_id).await {
                        error!("Failed to release lock for task {}: {}", task_id, e);
                    } else {
                        info!("Released lock for task {}", task_id);
                    }
                });

            } else {
                info!("Could not acquire lock for task {} (already locked by another instance?)", task_id);
                // Optionally update next_run_time slightly in the future to avoid immediate re-fetch?
                // Or just let it be picked up in the next cycle.
            }
        }
        Ok(())
    }

    #[instrument(skip(pg_pool, redis_conn, task), fields(task_id = %task.id, task_name = %task.name))]
    async fn execute_task(pg_pool: PgPool, redis_conn: &mut ConnectionManager, task: Task) -> Result<()> {
        info!("Executing task {}", task.id);
        // 1. Mark task as running in DB
        db::mark_task_as_running(&pg_pool, task.id).await?;

        // 2. TODO: Actual task execution logic here!
        // This would involve interpreting task.payload and performing the action.
        // Simulating work:
        sleep(Duration::from_secs(2)).await;
        let execution_result: std::result::Result<(), String> = Ok(()); // Simulate success
        // let execution_result: std::result::Result<(), String> = Err("Simulated execution failure".to_string()); // Simulate failure

        // 3. Update task status based on result
        let (final_status, error_msg) = match execution_result {
            Ok(_) => {
                info!("Task {} completed successfully", task.id);
                (TaskStatus::Completed, None)
            }
            Err(e) => {
                error!("Task {} failed: {}", task.id, e);
                if task.current_retries + 1 >= task.max_retries {
                    (TaskStatus::Failed, Some(e))
                } else {
                    (TaskStatus::Retry, Some(e))
                }
            }
        };

        db::update_task_status(&pg_pool, task.id, final_status.clone(), error_msg).await?;

        // 4. Calculate and set next run time (only if completed or retry)
        if final_status == TaskStatus::Completed || final_status == TaskStatus::Retry {
             // Calculate next run time based on schedule or retry logic
             let next_run = if final_status == TaskStatus::Retry {
                 // Implement backoff strategy for retries
                 Some(Utc::now() + chrono::Duration::seconds(60 * (task.current_retries + 1) as i64)) // Exponential backoff example
             } else {
                 task.calculate_next_run(Utc::now())? // Use the task's schedule
             };

             let reset_retries = final_status == TaskStatus::Completed;
             db::update_task_next_run(&pg_pool, task.id, next_run, reset_retries).await?;
             info!("Scheduled next run for task {} at {:?}", task.id, next_run);
        }

        // Lock is released by the caller (tick function) after this spawn finishes

        Ok(())
    }
}
*/

// Remove the old test or adapt it
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // Add new tests for DB, Redis, Task creation etc.
        // Example: Test task creation
        // let task = Task::new("Test Task".to_string(), "* * * * *".to_string(), json!({"data": 1}), 3);
        // assert_eq!(task.name, "Test Task");
        assert_eq!(2 + 2, 4); // Placeholder test
    }
}
