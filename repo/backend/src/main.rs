#[macro_use] extern crate rocket;

mod auth;
mod config;
mod dto;
mod jobs;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;
mod utils;

use config::AppConfig;
use middleware::correlation::CorrelationId;
use middleware::csrf_guard::CsrfOriginCheck;
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

    let config = AppConfig::from_env().unwrap_or_else(|e| {
        eprintln!("FATAL: {}", e);
        std::process::exit(1);
    });

    let pool = MySqlPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Database connection pool established");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations applied");

    services::seed::seed_default_users(&pool).await?;
    services::seed::seed_resources_and_rules(&pool).await?;

    // Background job loop — wakes every JOB_TICK_SECONDS (default 60).
    //
    // Each call applies its own internal cadence gate:
    //   - run_scheduled_transitions: publishes every course whose effective date
    //     has passed; no additional gate.
    //   - run_risk_evaluation: evaluates only rules whose per-rule
    //     schedule_interval_minutes has elapsed since last_run_at; rules not
    //     yet due are skipped by the DB query.
    //   - process_webhooks: delivers every queue entry whose next_attempt_at
    //     <= NOW(); respects exponential back-off stored in the DB row.
    let job_tick = config.job_tick_seconds;
    let pool_jobs = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(job_tick));
        loop {
            interval.tick().await;
            let _ = jobs::run_scheduled_transitions(&pool_jobs).await;
            let _ = jobs::run_risk_evaluation(&pool_jobs).await;
            let _ = jobs::process_webhooks(&pool_jobs).await;
        }
    });

    // Cleanup job: expired nonces, rate limits, old versions (every hour)
    let pool_cleanup = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let _ = jobs::cleanup_expired_data(&pool_cleanup).await;
        }
    });

    let allowed_origin = std::env::var("ALLOWED_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let cors = rocket_cors::CorsOptions::default()
        .allowed_origins(rocket_cors::AllowedOrigins::some_exact(&[&allowed_origin]))
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
        .attach(CsrfOriginCheck)
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
        .mount("/api/v1/terms", routes::terms::routes())
        .mount("/api/v1/notifications", routes::notifications::routes())
        .register("/", catchers![
            utils::errors::forbidden,
            utils::errors::not_found,
            utils::errors::unauthorized,
            utils::errors::unprocessable,
            utils::errors::too_many_requests,
            utils::errors::internal_error,
        ])
        .launch()
        .await?;

    Ok(())
}
