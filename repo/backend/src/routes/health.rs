use rocket::serde::json::Json;
use rocket::Route;
use serde::Serialize;

use crate::utils::response::ApiResponse;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[get("/health")]
pub fn health_check() -> Json<ApiResponse<HealthResponse>> {
    ApiResponse::ok(HealthResponse {
        status: "ok".to_string(),
        service: "campus-learn-backend".to_string(),
    })
}

pub fn routes() -> Vec<Route> {
    routes![health_check]
}
