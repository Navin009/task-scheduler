use chrono::{DateTime, Duration, Utc};
use cron_parser::ParseError;
use cron_parser::parse;
use scheduler_core::models::{Job, ScheduleType, Template};
use std::collections::HashSet;

pub struct ScheduleExpander {
    pub timezone: chrono_tz::Tz,
}

impl ScheduleExpander {
    pub fn new(timezone: chrono_tz::Tz) -> Self {
        Self { timezone }
    }

    pub fn expand_template(
        &self,
        template: &Template,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Job>, crate::error::Error> {
        let mut jobs = Vec::new();
        let mut current_time = start_time;

        while current_time <= end_time {
            match parse(&template.cron_pattern, &current_time) {
                Ok(next_time) => {
                    if next_time > end_time {
                        break;
                    }

                    let job = Job {
                        id: uuid::Uuid::new_v4().to_string(),
                        schedule_type: ScheduleType::Recurring,
                        schedule: template.cron_pattern.clone(),
                        payload: template.payload_template.clone(),
                        status: scheduler_core::models::JobStatus::Pending,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        retries: 0,
                        max_retries: 3,
                    };

                    jobs.push(job);
                    current_time = next_time + Duration::seconds(1);
                }
                Err(e) => return Err(crate::error::Error::ScheduleParse(e.to_string())),
            }
        }

        Ok(jobs)
    }

    pub fn adjust_for_timezone(&self, time: DateTime<Utc>) -> DateTime<Utc> {
        time.with_timezone(&self.timezone).with_timezone(&Utc)
    }

    pub fn handle_daylight_saving(&self, time: DateTime<Utc>) -> DateTime<Utc> {
        // This is a simplified implementation. In a production system,
        // you would need to handle edge cases and transitions more carefully
        self.adjust_for_timezone(time)
    }
}

pub struct ScheduleOptimizer {
    pub batch_size: usize,
    pub deduplication_window: Duration,
}

impl ScheduleOptimizer {
    pub fn new(batch_size: usize, deduplication_window: Duration) -> Self {
        Self {
            batch_size,
            deduplication_window,
        }
    }

    pub fn optimize_jobs(&self, jobs: Vec<Job>) -> Vec<Job> {
        let mut optimized = Vec::new();
        let mut seen = HashSet::new();

        for job in jobs {
            let key = format!("{}-{}", job.schedule, job.payload);
            if !seen.contains(&key) {
                seen.insert(key);
                optimized.push(job);
            }
        }

        optimized
    }

    pub fn batch_jobs(&self, jobs: Vec<Job>) -> Vec<Vec<Job>> {
        jobs.chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}
