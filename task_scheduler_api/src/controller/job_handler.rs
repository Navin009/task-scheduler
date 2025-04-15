use crate::model::job::Job;
use crate::state::job_service_state::JobServiceState;
use rocket::State;
use rocket::serde::json::Json;

#[post("/job/schedule", format = "json", data = "<job>")]
pub fn schedule_job(job: Json<Job>, job_service: &State<JobServiceState>) -> &'static str {
    job_service.add_job(job.into_inner());
    "Job scheduled successfully"
}

#[get("/job/retrieve/<id>")]
pub fn retrieve_job(id: String, job_service: &State<JobServiceState>) -> Option<Json<Job>> {
    job_service.get_job(&id).map(Json)
}