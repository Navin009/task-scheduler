// src/service/job_service.rs
use crate::model::job::Job;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct JobService {
    jobs: Mutex<HashMap<String, Job>>,
}

impl JobService {
    pub fn new() -> Self {
        JobService {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_job(&self, job: Job) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.insert(job.id.clone(), job);
    }

    pub fn get_job(&self, id: &str) -> Option<Job> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(id).cloned()
    }
}
