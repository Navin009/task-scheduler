use anyhow::Result;
use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use std::time::Duration;

#[derive(Debug)]
pub struct CacheConfig {
    pub url: String,
    pub max_connections: u32,
}

pub struct Cache {
    client: Client,
}

impl Cache {
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let client = Client::open(config.url)?;
        Ok(Self { client })
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }

    pub async fn push_to_queue(&self, queue_name: &str, value: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let _: i64 = conn.lpush(queue_name, value).await?;
        Ok(())
    }

    pub async fn pop_from_queue(&self, queue_name: &str) -> Result<Option<String>> {
        let mut conn = self.get_conn().await?;
        let value: Option<String> = conn.rpop(queue_name, None).await?;
        Ok(value)
    }

    pub async fn push_to_priority_queue(
        &self,
        queue_name: &str,
        value: &str,
        priority: i32,
    ) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let _: i64 = conn.zadd(queue_name, value, priority as f64).await?;
        Ok(())
    }

    pub async fn pop_from_priority_queue(&self, queue_name: &str) -> Result<Option<String>> {
        let mut conn = self.get_conn().await?;
        let values: Vec<(String, f64)> = conn.zpopmax(queue_name, 1).await?;
        Ok(values.into_iter().next().map(|(value, _)| value))
    }

    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let _: () = conn.set_ex(key, value, ttl.as_secs() as u64).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_conn().await?;
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_conn().await?;
        let result: i64 = conn.del(key).await?;
        Ok(result > 0)
    }

    pub async fn add_to_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let mut conn = self.get_conn().await?;
        let result: i64 = conn.sadd(set_name, value).await?;
        Ok(result > 0)
    }

    pub async fn remove_from_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let mut conn = self.get_conn().await?;
        let result: i64 = conn.srem(set_name, value).await?;
        Ok(result > 0)
    }

    pub async fn is_member_of_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let mut conn = self.get_conn().await?;
        let result: bool = conn.sismember(set_name, value).await?;
        Ok(result)
    }
}
