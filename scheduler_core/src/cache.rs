use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use valkey::Client;

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
        let client = Client::connect(&config.url).await?;
        Ok(Self { client })
    }

    pub async fn push_to_queue(&self, queue_name: &str, value: &str) -> Result<()> {
        self.client.lpush(queue_name, value).await?;
        Ok(())
    }

    pub async fn pop_from_queue(&self, queue_name: &str) -> Result<Option<String>> {
        let value = self.client.rpop(queue_name).await?;
        Ok(value)
    }

    pub async fn push_to_priority_queue(
        &self,
        queue_name: &str,
        value: &str,
        priority: i32,
    ) -> Result<()> {
        let score = priority as f64;
        self.client.zadd(queue_name, score, value).await?;
        Ok(())
    }

    pub async fn pop_from_priority_queue(&self, queue_name: &str) -> Result<Option<String>> {
        let value = self.client.zpopmax(queue_name).await?;
        Ok(value)
    }

    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        self.client.set(key, value).await?;
        self.client.expire(key, ttl.as_secs() as i64).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let value = self.client.get(key).await?;
        Ok(value)
    }

    pub async fn delete(&self, key: &str) -> Result<bool> {
        let result = self.client.del(key).await?;
        Ok(result > 0)
    }

    pub async fn add_to_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let result = self.client.sadd(set_name, value).await?;
        Ok(result > 0)
    }

    pub async fn remove_from_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let result = self.client.srem(set_name, value).await?;
        Ok(result > 0)
    }

    pub async fn is_member_of_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let result = self.client.sismember(set_name, value).await?;
        Ok(result)
    }
}
