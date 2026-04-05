// Application configuration loaded from environment variables.
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
    /// 64-character hex string (32 bytes) used exclusively for AES-256-GCM
    /// sensitive-field encryption. Never shares the JWT secret.
    pub data_encryption_key: String,
    /// Runtime environment: "development" | "staging" | "production".
    /// Controls whether missing encryption keys are fatal.
    pub app_env: String,
    /// Directory where uploaded media files are stored.
    /// Default: "/app/uploads" (env: `MEDIA_UPLOAD_DIR`).
    pub media_upload_dir: String,
    /// How often (in seconds) the main background job loop wakes up.
    ///
    /// On every tick the loop calls:
    ///   - `run_scheduled_transitions` — publishes all courses whose effective
    ///     date has passed (no further cadence gate).
    ///   - `run_risk_evaluation` — queries the DB for risk rules whose own
    ///     `schedule_interval_minutes` has elapsed; rules not yet due are
    ///     skipped without side-effects.
    ///   - `process_webhooks` — delivers all queue entries whose
    ///     `next_attempt_at <= NOW()` (no further cadence gate).
    ///
    /// Setting this lower than the smallest per-rule `schedule_interval_minutes`
    /// wastes a few DB round-trips but is otherwise harmless.
    /// Default: 60 (env: `JOB_TICK_SECONDS`).
    pub job_tick_seconds: u64,
    /// Max login attempts per IP per minute (env: `LOGIN_RATE_LIMIT_PER_MINUTE`, default: 30).
    pub login_rate_limit_per_minute: i32,
    /// Max login attempts per IP per hour (env: `LOGIN_RATE_LIMIT_PER_HOUR`, default: 200).
    pub login_rate_limit_per_hour: i32,
    /// Failed login attempts before account lockout (env: `LOGIN_LOCKOUT_THRESHOLD`, default: 10).
    pub login_lockout_threshold: i32,
    /// Account lockout duration in minutes (env: `LOGIN_LOCKOUT_MINUTES`, default: 15).
    pub login_lockout_minutes: i64,
}

/// Dev-only fallback key: bytes 0x01–0x20 expressed as hex (64 chars = 32 bytes).
/// This constant is committed intentionally so local development works without
/// environment setup. It must NEVER be used in staging or production.
const DEV_FALLBACK_ENCRYPTION_KEY: &str =
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

impl AppConfig {
    /// Load configuration from the process environment.
    ///
    /// # Errors
    /// Returns `Err` when `DATA_ENCRYPTION_KEY` is absent or malformed in any
    /// environment other than `APP_ENV=development`.
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        let data_encryption_key = resolve_data_encryption_key(&app_env)?;

        Ok(Self {
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
            job_tick_seconds: env::var("JOB_TICK_SECONDS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .unwrap_or(60),
            media_upload_dir: env::var("MEDIA_UPLOAD_DIR")
                .unwrap_or_else(|_| "/app/uploads".into()),
            login_rate_limit_per_minute: env::var("LOGIN_RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            login_rate_limit_per_hour: env::var("LOGIN_RATE_LIMIT_PER_HOUR")
                .unwrap_or_else(|_| "200".into())
                .parse()
                .unwrap_or(200),
            login_lockout_threshold: env::var("LOGIN_LOCKOUT_THRESHOLD")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            login_lockout_minutes: env::var("LOGIN_LOCKOUT_MINUTES")
                .unwrap_or_else(|_| "15".into())
                .parse()
                .unwrap_or(15),
            data_encryption_key,
            app_env,
        })
    }
}

/// Resolve the `DATA_ENCRYPTION_KEY` from the environment, applying the
/// dev-mode fallback policy.
fn resolve_data_encryption_key(app_env: &str) -> Result<String, String> {
    match env::var("DATA_ENCRYPTION_KEY") {
        Ok(key) => {
            validate_encryption_key_hex(&key).map_err(|e| {
                format!("Invalid DATA_ENCRYPTION_KEY: {}", e)
            })?;
            Ok(key)
        }
        Err(_) => {
            if app_env == "development" {
                // Log at warn level so the message appears in the startup output.
                // tracing may not yet be initialized here; use eprintln as belt-and-
                // suspenders so the warning is never silently dropped.
                eprintln!(
                    "WARNING: DATA_ENCRYPTION_KEY is not set. \
                     Using the insecure dev fallback key. \
                     Set APP_ENV != 'development' and provide DATA_ENCRYPTION_KEY \
                     before deploying to any shared or production environment."
                );
                Ok(DEV_FALLBACK_ENCRYPTION_KEY.to_string())
            } else {
                Err(format!(
                    "DATA_ENCRYPTION_KEY is required in the '{}' environment. \
                     Generate a key with: openssl rand -hex 32",
                    app_env
                ))
            }
        }
    }
}

/// Validate that a candidate encryption key is a 64-character, valid hex string
/// (representing 32 bytes suitable for AES-256-GCM).
///
/// Exposed publicly so `crypto_service` and tests can reuse the same rule.
pub fn validate_encryption_key_hex(key: &str) -> Result<(), String> {
    if key.len() != 64 {
        return Err(format!(
            "must be exactly 64 hex characters (32 bytes for AES-256); got {} characters",
            key.len()
        ));
    }
    hex::decode(key).map_err(|e| format!("not valid hex: {}", e))?;
    Ok(())
}
