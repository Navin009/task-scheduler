use chrono::{DateTime, Duration, Utc};
use scheduler_core::{
    cache::RedisClient,
    db::Database,
    models::{Job, Template},
};
use tracing::{info, warn};

use crate::{
    error::Error,
    schedule::{ScheduleExpander, ScheduleOptimizer},
};

pub struct RecurrenceManager {
    db: Database,
    cache: RedisClient,
    expander: ScheduleExpander,
    optimizer: ScheduleOptimizer,
    look_ahead_window: Duration,
}

impl RecurrenceManager {
    pub async fn new(
        db: Database,
        cache: RedisClient,
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
        let templates = self.db.get_active_templates().await?;
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
        for batch in job_batches {
            self.db.create_jobs(&batch).await?;

            // Queue jobs for execution
            for job in batch {
                if let Err(e) = self.cache.push_job(&job).await {
                    warn!("Failed to queue job {}: {}", job.id, e);
                }
            }
        }

        info!(
            "Processed template {}: created {} jobs",
            template.id,
            job_batches.iter().map(|b| b.len()).sum::<usize>()
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
                    self.db.update_job_schedule(&job.id, &adjusted_time).await?;
                }
            }
        }

        Ok(())
    }
}
