use dotenv::dotenv;
use env_logger::Builder;
use log::LevelFilter;
use regex::Regex;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use scheduler_core::{cache::Cache, config::Config, db::Database, task::TaskManager};
use serde_yaml::Value;
use sqlx::postgres::PgPool;
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, env, fs::File, io::Read};

use crate::guard::basic_auth::BasicAuth;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub task_manager: TaskManager,
    pub config: HashMap<String, Value>,
}

impl AppConfig {
    pub fn new(db: Database, cache: Cache) -> Self {
        Self {
            task_manager: TaskManager::new(db),
            config: HashMap::new(),
        }
    }

    pub async fn init_db(
        config: &HashMap<String, Value>,
    ) -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config["database"]["postgresql"]["user"].as_str().unwrap(),
            config["database"]["postgresql"]["password"]
                .as_str()
                .unwrap(),
            config["database"]["postgresql"]["host"].as_str().unwrap(),
            config["database"]["postgresql"]["port"].as_u64().unwrap(),
            "task_scheduler"
        );

        log::info!("PostgreSQL URL: {}", database_url);

        let pool = Pool::<Postgres>::connect(&database_url).await?;
        Ok(pool)
    }

    pub fn init_logger() {
        Builder::new().filter_level(LevelFilter::Info).init();
    }

    pub async fn init_app_config() -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let mut file = File::open("task_scheduler_api/config/application.yaml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let processed_contents = AppConfig::process_placeholders(&contents);

        let config: HashMap<String, Value> = serde_yaml::from_str(&processed_contents)?;
        Ok(config)
    }

    fn process_placeholders(contents: &str) -> String {
        // Regex to match placeholders of the form ${VAR:default_value}
        let re = Regex::new(r"\$\{([A-Za-z0-9_]+):([^\}]+)\}").unwrap();

        // Replace each placeholder with its corresponding value
        re.replace_all(contents, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let default_value = &caps[2];

            // Get the value of the environment variable or use the default value
            env::var(var_name).unwrap_or_else(|_| default_value.to_string())
        })
        .to_string()
    }

    pub fn get_basic_auth(&self) -> Result<BasicAuth, Box<dyn std::error::Error>> {
        let username = self
            .config
            .get("server")
            .and_then(|server| server.get("authentication"))
            .and_then(|auth| auth.get("basic"))
            .and_then(|auth| auth.get("username"))
            .and_then(|u| u.as_str())
            .ok_or("Missing or invalid 'username' field")?;

        let password = self
            .config
            .get("server")
            .and_then(|server| server.get("authentication"))
            .and_then(|auth| auth.get("basic"))
            .and_then(|auth| auth.get("password"))
            .and_then(|p| p.as_str())
            .ok_or("Missing or invalid 'password' field")?;

        // Return the BasicAuth struct
        Ok(BasicAuth {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}
