use async_trait::async_trait;
use scheduler_core::task::Job;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{error, info};

#[async_trait]
pub trait NotificationChannel: Send + Sync {
    async fn send(&self, message: &str) -> Result<(), anyhow::Error>;
}

pub struct AlertManager {
    channels: Vec<Box<dyn NotificationChannel>>,
    last_alert_time: Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>,
    cooldown_period: chrono::Duration,
}

impl AlertManager {
    pub fn new(cooldown_period: chrono::Duration) -> Self {
        Self {
            channels: Vec::new(),
            last_alert_time: Mutex::new(HashMap::new()),
            cooldown_period,
        }
    }

    pub fn add_channel(&mut self, channel: Box<dyn NotificationChannel>) {
        self.channels.push(channel);
    }

    pub async fn alert_job_failure(&self, job: &Job) {
        let message = format!(
            "Job {} failed (retry {}/{}). Job Type: {}",
            job.id, job.retries, job.max_retries, job.job_type
        );

        let mut last_alert_time = self.last_alert_time.lock().await;
        let now = chrono::Utc::now();

        if let Some(last_time) = last_alert_time.get(&job.id) {
            if now - *last_time < self.cooldown_period {
                info!("Skipping alert for job {} due to cooldown period", job.id);
                return;
            }
        }

        for channel in &self.channels {
            if let Err(e) = channel.send(&message).await {
                error!("Failed to send alert through channel: {}", e);
            }
        }

        last_alert_time.insert(job.id.clone(), now);
    }

    pub async fn alert_dead_letter(&self, job: &Job) {
        let message = format!(
            "Job {} moved to dead letter queue after {} retries. Job Type: {}",
            job.id, job.retries, job.job_type
        );

        for channel in &self.channels {
            if let Err(e) = channel.send(&message).await {
                error!("Failed to send dead letter alert through channel: {}", e);
            }
        }
    }
}

// Example implementation of a notification channel
pub struct LogNotificationChannel;

#[async_trait]
impl NotificationChannel for LogNotificationChannel {
    async fn send(&self, message: &str) -> Result<(), anyhow::Error> {
        info!("ALERT: {}", message);
        Ok(())
    }
}
