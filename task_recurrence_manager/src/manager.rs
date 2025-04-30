use anyhow;
use chrono::{DateTime, Duration, Utc};
use scheduler_core::{
    cache::Cache,
    db::{Database, JobData},
    models::Template,
};
use tracing::{info, warn};

use crate::{
    error::Error,
    schedule::{ScheduleExpander, ScheduleOptimizer},
};

pub struct RecurrenceManager {
    db: Database,
    cache: Cache,
    expander: ScheduleExpander,
    optimizer: ScheduleOptimizer,
    look_ahead_window: Duration,
}

impl RecurrenceManager {
    pub async fn new(
        db: Database,
        cache: Cache,
        timezone: chrono_tz::Tz,
        look_ahead_window: Duration,
    ) -> Result<Self, Error> {
        Ok(Self {
            db,
            cache,
            expander: ScheduleExpander::new(timezone),
            optimizer: ScheduleOptimizer::new(100, Duration::hours(1)),
            look_ahead_window,
        })
    }
    pub async fn process_templates(&self) -> Result<(), Error> {
        let templates = self.db.get_active_templates().await.map_err(Error::from)?;
        let now = Utc::now();
        let end_time = now + self.look_ahead_window;

        for template in templates {
            if let Err(e) = self.process_template(&template, now, end_time).await {
                warn!("Failed to process template {}: {}", template.id, e);
            }
        }

        Ok(())
    }

    async fn process_template(
        &self,
        template: &Template,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<(), Error> {
        // Expand the template into jobs
        let mut jobs = self
            .expander
            .expand_template(template, start_time, end_time)?;

        // Optimize the jobs (deduplicate, batch)
        jobs = self.optimizer.optimize_jobs(jobs);
        let job_batches = self.optimizer.batch_jobs(jobs);

        // Store jobs in database and queue
        let job_batches_clone = job_batches.clone();
        for batch in job_batches {
            for job in &batch {
                let job_data = JobData {
                    job_type: job.schedule_type.clone(),
                    status: job.status,
                    priority: 0, // Default priority since it's not in the Job struct
                    schedule_at: Some(job.schedule),
                    cron: None,
                    interval: None,
                    parent_job_id: None,
                    max_retries: job.max_retries,
                    retries: job.retries,
                    payload: serde_json::to_value(&job.payload)
                        .map_err(anyhow::Error::from)
                        .map_err(Error::from)?,
                    active: true,
                    description: None,
                    max_attempts: 3,
                    metadata: None,
                    name: None,
                };
                self.db.create_job(job_data).await?;
            }

            // Queue jobs for execution
            for job in batch {
                if let Err(e) = self.cache.push_to_queue("default", &job.id).await {
                    warn!("Failed to queue job {}: {}", job.id, e);
                }
            }
        }

        info!(
            "Processed template {}: created {} jobs",
            template.id,
            job_batches_clone.iter().map(|b| b.len()).sum::<usize>()
        );

        Ok(())
    }

    pub async fn handle_daylight_saving_transition(&self) -> Result<(), Error> {
        let now = Utc::now();
        let templates = self.db.get_active_templates().await?;

        for template in templates {
            let jobs = self.expander.expand_template(
                &template,
                now - Duration::hours(1),
                now + Duration::hours(1),
            )?;

            for job in jobs {
                let adjusted_time = self.expander.handle_daylight_saving(job.created_at);
                if adjusted_time != job.created_at {
                    let mut updates = std::collections::HashMap::new();
                    updates.insert("scheduled_at", adjusted_time.to_rfc3339());
                    self.db.update_job(&job.id, &updates).await?;
                }
            }
        }

        Ok(())
    }
}
