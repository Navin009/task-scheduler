use crate::{Config, Error, Job};
use redis::{AsyncCommands, Client};

pub struct RedisQueue {
    client: Client,
}

impl RedisQueue {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let client = Client::open(config.redis_url.clone())?;
        Ok(Self { client })
    }

    pub async fn push_job(&self, queue: &str, job: &Job) -> Result<(), Error> {
        let mut conn = self.client.get_async_connection().await?;
        let serialized = serde_json::to_string(job)?;
        conn.lpush(queue, serialized).await?;
        Ok(())
    }

    pub async fn pop_job(&self, queues: &[&str]) -> Result<Option<Job>, Error> {
        let mut conn = self.client.get_async_connection().await?;
        let (queue_name, serialized): (String, String) = conn.blpop(queues, 0).await?;

        let job: Job = serde_json::from_str(&serialized)?;
        Ok(Some(job))
    }
}
