use crate::error::Error;
use regex::Regex;
use serde::Deserialize;
use serde_yaml::Value;
use sqlx::Pool;
use sqlx::Postgres;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub max_retries: u32,
    pub queue_names: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        let mut queue_names = Vec::new();
        let mut i = 0;
        while let Ok(queue_name) = env::var(format!("QUEUE_NAMES__{}", i)) {
            queue_names.push(queue_name);
            i += 1;
        }

        if queue_names.is_empty() {
            return Err(Error::ConfigError("No queue names found".to_string()));
        }

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| Error::ConfigError("Missing DATABASE_URL".to_string()))?,
            redis_url: env::var("REDIS_URL")
                .map_err(|_| Error::ConfigError("Missing REDIS_URL".to_string()))?,
            max_retries: env::var("MAX_RETRIES")
                .map_err(|_| Error::ConfigError("Missing MAX_RETRIES".to_string()))?
                .parse()
                .map_err(|_| Error::ConfigError("Invalid MAX_RETRIES".to_string()))?,
            queue_names,
        })
    }

    pub async fn init_db(config: &HashMap<String, Value>) -> Result<Pool<Postgres>, Error> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config["database"]["postgresql"]["user"]
                .as_str()
                .ok_or_else(|| Error::ConfigError("Missing database user".to_string()))?,
            config["database"]["postgresql"]["password"]
                .as_str()
                .ok_or_else(|| Error::ConfigError("Missing database password".to_string()))?,
            config["database"]["postgresql"]["host"]
                .as_str()
                .ok_or_else(|| Error::ConfigError("Missing database host".to_string()))?,
            config["database"]["postgresql"]["port"]
                .as_u64()
                .ok_or_else(|| Error::ConfigError("Missing database port".to_string()))?,
            "task_scheduler"
        );

        let pool = Pool::<Postgres>::connect(&database_url)
            .await
            .map_err(|e| Error::ConfigError(e.to_string()))?;
        Ok(pool)
    }

    pub async fn from_yaml(path: &str) -> Result<HashMap<String, Value>, Error> {
        let mut file = File::open(path)
            .map_err(|e| Error::ConfigError(format!("Failed to open config file: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::ConfigError(format!("Failed to read config file: {}", e)))?;

        let processed_contents = Self::process_placeholders(&contents);
        let config: HashMap<String, Value> = serde_yaml::from_str(&processed_contents)
            .map_err(|e| Error::ConfigError(format!("Failed to parse YAML: {}", e)))?;
        Ok(config)
    }

    fn process_placeholders(contents: &str) -> String {
        let re = Regex::new(r"\$\{([A-Za-z0-9_]+):([^\}]+)\}").unwrap();
        re.replace_all(contents, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let default_value = &caps[2];
            env::var(var_name).unwrap_or_else(|_| default_value.to_string())
        })
        .to_string()
    }
}
