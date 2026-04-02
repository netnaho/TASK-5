use rocket::serde::json::Json;
use rocket::Route;
use serde::Serialize;

#[derive(Serialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub api_version: String,
}

#[get("/info")]
pub fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: "CampusLearn Operations Suite".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Enterprise-grade campus learning management and operations platform".to_string(),
        api_version: "v1".to_string(),
    })
}

pub fn routes() -> Vec<Route> {
    routes![info]
}
