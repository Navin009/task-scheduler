pub mod jobs;
pub mod ping;
pub mod templates;

pub fn ping_routes() -> Vec<rocket::Route> {
    routes![ping::ping, ping::db_check, ping::metrics, ping::prometheus]
}

pub fn jobs_routes() -> Vec<rocket::Route> {
    routes![
        jobs::create_job,
        jobs::get_job,
        jobs::list_jobs,
        jobs::update_job,
        jobs::delete_job
    ]
}

pub fn templates_routes() -> Vec<rocket::Route> {
    routes![
        templates::create_template,
        templates::get_template,
        templates::list_templates,
        templates::update_template,
        templates::delete_template
    ]
}
