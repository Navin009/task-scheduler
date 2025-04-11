// src/model/job.rs
use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Job {
    pub id: String,
    pub description: String,
    pub schedule_time: String,
}
