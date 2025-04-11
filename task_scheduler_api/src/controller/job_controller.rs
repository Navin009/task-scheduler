use crate::model::job::Job;
use crate::service::job_service::JobService;
use rocket::State;
use rocket::serde::json::Json;

#[post("/schedule", format = "json", data = "<job>")]
pub fn schedule_job(job: Json<Job>, job_service: &State<JobService>) -> &'static str {
    job_service.add_job(job.into_inner());
    "Job scheduled successfully"
}

#[get("/retrieve/<id>")]
pub fn retrieve_job(id: String, job_service: &State<JobService>) -> Option<Json<Job>> {
    job_service.get_job(&id).map(Json)
}
