#[macro_use] extern crate rocket;

mod auth;
mod config;
mod dto;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;
mod utils;

use config::AppConfig;
use middleware::correlation::CorrelationId;
use rocket::fairing::AdHoc;
use sqlx::mysql::MySqlPoolOptions;
use tracing_subscriber::{fmt, EnvFilter};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting CampusLearn Operations Suite backend");

    let config = AppConfig::from_env();

    let pool = MySqlPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Database connection pool established");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations applied");

    services::seed::seed_default_users(&pool).await?;
    services::seed::seed_resources_and_rules(&pool).await?;

    let cors = rocket_cors::CorsOptions::default()
        .allowed_origins(rocket_cors::AllowedOrigins::all())
        .allowed_methods(
            vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
                .into_iter()
                .map(|s| s.parse().unwrap())
                .collect(),
        )
        .allowed_headers(rocket_cors::AllowedHeaders::all())
        .allow_credentials(true)
        .to_cors()?;

    rocket::build()
        .manage(pool)
        .manage(config.clone())
        .attach(cors)
        .attach(CorrelationId)
        .attach(AdHoc::on_response("Request Logger", |req, res| {
            Box::pin(async move {
                tracing::info!(
                    method = %req.method(),
                    uri = %req.uri(),
                    status = res.status().code,
                    "Request completed"
                );
            })
        }))
        .mount("/", routes::health::routes())
        .mount("/api/v1", routes::info::routes())
        .mount("/api/v1/auth", routes::auth::routes())
        .mount("/api/v1/courses", routes::courses::routes())
        .mount("/api/v1/approvals", routes::approvals::routes())
        .mount("/api/v1/audit", routes::audit::routes())
        .mount("/api/v1/tags", routes::tags::routes())
        .mount("/api/v1/bookings", routes::bookings::routes())
        .mount("/api/v1/risk", routes::risk::routes())
        .mount("/api/v1/privacy", routes::privacy::routes())
        .launch()
        .await?;

    Ok(())
}
