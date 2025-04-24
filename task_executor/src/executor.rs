use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use scheduler_core::{
    cache::RedisClient,
    db::Database,
    models::{Job, JobStatus},
};

use crate::{error::Error, process::ProcessManager, state::ExecutionState};

pub struct TaskExecutor {
    db: Database,
    cache: RedisClient,
    process_manager: ProcessManager,
    concurrency_limit: usize,
    semaphore: Semaphore,
}

impl TaskExecutor {
    pub async fn new(
        db: Database,
        cache: RedisClient,
        timeout: Duration,
        max_memory_mb: u64,
        max_cpu_percent: u32,
        concurrency_limit: usize,
    ) -> Result<Self, Error> {
        let process_manager = ProcessManager::new(timeout, max_memory_mb, max_cpu_percent);
        process_manager.validate_resources()?;

        Ok(Self {
            db,
            cache,
            process_manager,
            concurrency_limit,
            semaphore: Semaphore::new(concurrency_limit),
        })
    }

    pub async fn start(&self) -> Result<(), Error> {
        info!(
            "Starting task executor with concurrency limit: {}",
            self.concurrency_limit
        );

        loop {
            // Wait for a permit before processing next job
            let _permit =
                self.semaphore.acquire().await.map_err(|e| {
                    Error::ResourceLimit(format!("Failed to acquire semaphore: {}", e))
                })?;

            // Try to get next job from queue
            match self.cache.pop_job().await {
                Ok(Some(job)) => {
                    let executor = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = executor.execute_job(job).await {
                            error!("Failed to execute job: {}", e);
                        }
                    });
                }
                Ok(None) => {
                    // No jobs available, wait a bit before trying again
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error getting job from queue: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn execute_job(&self, mut job: Job) -> Result<(), Error> {
        let mut state = ExecutionState::new(job);

        // Mark job as running
        state.mark_running()?;
        self.db
            .update_job_status(&state.job.id, JobStatus::Running)
            .await?;

        // Parse job payload
        let payload: serde_json::Value = serde_json::from_str(&state.job.payload.to_string())
            .map_err(|e| Error::Process(format!("Invalid job payload: {}", e)))?;

        // Extract command and arguments
        let command = payload["command"]
            .as_str()
            .ok_or_else(|| Error::Process("Missing command in payload".into()))?;
        let args: Vec<String> = payload["args"]
            .as_array()
            .ok_or_else(|| Error::Process("Missing args in payload".into()))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        // Execute command
        match self
            .process_manager
            .execute_command(command, &args, &[])
            .await
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout).into();
                state.mark_completed(output_str)?;
                self.db
                    .update_job_status(&state.job.id, JobStatus::Completed)
                    .await?;
            }
            Err(e) => {
                state.mark_failed(e.to_string())?;
                self.db
                    .update_job_status(&state.job.id, JobStatus::Failed)
                    .await?;

                // Check if we should retry
                if state.job.retries < state.job.max_retries {
                    if let Ok(()) = state.mark_retrying() {
                        self.db
                            .update_job_status(&state.job.id, JobStatus::Retrying)
                            .await?;
                        self.cache.push_job(&state.job).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Clone for TaskExecutor {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            cache: self.cache.clone(),
            process_manager: ProcessManager::new(
                self.process_manager.timeout,
                self.process_manager.max_memory_mb,
                self.process_manager.max_cpu_percent,
            ),
            concurrency_limit: self.concurrency_limit,
            semaphore: self.semaphore.clone(),
        }
    }
}
