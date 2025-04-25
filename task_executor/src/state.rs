use chrono::Utc;
use scheduler_core::models::{Job, JobStatus};
use tracing::{error, info};

use crate::error::Error;

pub struct ExecutionState {
    pub job: Job,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub output: Option<String>,
    pub error: Option<String>,
}

impl ExecutionState {
    pub fn new(job: Job) -> Self {
        Self {
            job,
            start_time: Utc::now(),
            end_time: None,
            output: None,
            error: None,
        }
    }

    pub fn mark_running(&mut self) -> Result<(), Error> {
        if self.job.status != JobStatus::Pending {
            return Err(Error::StateTransition(format!(
                "Cannot transition to running from {:?}",
                self.job.status
            )));
        }

        self.job.status = JobStatus::Running;
        info!("Job {} marked as running", self.job.id);
        Ok(())
    }

    pub fn mark_completed(&mut self, output: String) -> Result<(), Error> {
        if self.job.status != JobStatus::Running {
            return Err(Error::StateTransition(format!(
                "Cannot transition to completed from {:?}",
                self.job.status
            )));
        }

        self.job.status = JobStatus::Completed;
        self.end_time = Some(Utc::now());
        self.output = Some(output);
        info!("Job {} marked as completed", self.job.id);
        Ok(())
    }

    pub fn mark_failed(&mut self, error: String) -> Result<(), Error> {
        if self.job.status != JobStatus::Running {
            return Err(Error::StateTransition(format!(
                "Cannot transition to failed from {:?}",
                self.job.status
            )));
        }

        self.job.status = JobStatus::Failed;
        self.end_time = Some(Utc::now());
        self.error = Some(error.clone());
        error!("Job {} failed: {}", self.job.id, error);
        Ok(())
    }

    pub fn mark_retrying(&mut self) -> Result<(), Error> {
        if self.job.status != JobStatus::Failed {
            return Err(Error::StateTransition(format!(
                "Cannot transition to retrying from {:?}",
                self.job.status
            )));
        }

        if self.job.retries >= self.job.max_retries {
            return Err(Error::StateTransition(format!(
                "Max retries ({}) exceeded for job {}",
                self.job.max_retries, self.job.id
            )));
        }

        self.job.status = JobStatus::Pending;
        self.job.retries += 1;
        info!(
            "Job {} marked for retry (attempt {}/{})",
            self.job.id, self.job.retries, self.job.max_retries
        );
        Ok(())
    }
}
