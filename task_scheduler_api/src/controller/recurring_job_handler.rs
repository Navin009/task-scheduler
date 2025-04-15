use crate::model::recurring_job::RecurringJob;
use crate::state::job_service_state::JobServiceState;
use rocket::State;
use rocket::serde::json::Json;

#[post("/recurring_job/schedule", format = "json", data = "<recurring_job>")]
pub fn schedule_recurring_job(
    recurring_job: Json<RecurringJob>,
    job_service: &State<JobServiceState>,
) -> &'static str {
    job_service.add_recurring_job(recurring_job.into_inner());
    "Recurring job scheduled successfully"
}

#[get("/recurring_job/retrieve/<id>")]
pub fn retrieve_recurring_job(
    id: String,
    job_service: &State<JobServiceState>,
) -> Option<Json<RecurringJob>> {
    job_service.get_recurring_job(&id).map(Json)
}