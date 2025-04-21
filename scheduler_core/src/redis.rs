use redis::{Client, AsyncCommands};
use serde_json;
use crate::error::Error;

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = Client::open(url)?;
        Ok(RedisClient { client })
    }

    pub async fn push_job(&self, queue: &str, job: &impl serde::Serialize) -> Result<(), Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(job)?;
        conn.lpush::<_, _, ()>(queue, serialized).await?;
        Ok(())
    }

    pub async fn pop_job(&self, queues: &[&str]) -> Result<Option<(String, String)>, Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: Option<(String, String)> = conn.blpop(queues, 0.0).await?;
        Ok(result)
    }
}
