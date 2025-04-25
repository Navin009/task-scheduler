use env_logger::Builder;
use log::LevelFilter;
use scheduler_core::{cache::Cache, db::Database, task::TaskManager};
use serde_yaml::Value;
use std::collections::HashMap;

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

    pub fn init_logger() {
        Builder::new().filter_level(LevelFilter::Info).init();
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
