pub mod jobs;
pub mod ping;
pub mod templates;

pub fn ping_routes() -> Vec<rocket::Route> {
    routes![ping::ping, ping::db_check, ping::metrics, ping::prometheus]
}
