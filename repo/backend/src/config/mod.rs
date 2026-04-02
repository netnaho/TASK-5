use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub db_max_connections: u32,
    pub server_port: u16,
    pub rate_limit_per_minute: u32,
    pub reauth_window_minutes: i64,
    pub hmac_nonce_expiry_seconds: i64,
    pub version_retention_days: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://campus:campus_pass@mysql:3306/campus_learn".into()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "campus-learn-dev-secret-change-in-production".into()),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()
                .unwrap_or(24),
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8000".into())
                .parse()
                .unwrap_or(8000),
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "120".into())
                .parse()
                .unwrap_or(120),
            reauth_window_minutes: env::var("REAUTH_WINDOW_MINUTES")
                .unwrap_or_else(|_| "15".into())
                .parse()
                .unwrap_or(15),
            hmac_nonce_expiry_seconds: env::var("HMAC_NONCE_EXPIRY_SECONDS")
                .unwrap_or_else(|_| "300".into())
                .parse()
                .unwrap_or(300),
            version_retention_days: env::var("VERSION_RETENTION_DAYS")
                .unwrap_or_else(|_| "180".into())
                .parse()
                .unwrap_or(180),
        }
    }
}
