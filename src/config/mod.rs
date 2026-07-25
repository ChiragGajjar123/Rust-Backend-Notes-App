use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_ms: i64,
    pub cors_allowed_origins: Vec<String>,
    pub server_host: String,
    pub server_port: u16,
    pub bcrypt_cost: u32,
    pub max_connections: u32,
    pub aws_region: String,
    pub aws_ses_from_email: Option<String>,
    pub password_reset_interval_secs: i64,
    pub password_reset_expiration_mins: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .or_else(|_| env::var("POSTGRES_URL"))
            .map_err(|_| "DATABASE_URL or POSTGRES_URL environment variable must be set".to_string())?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET environment variable must be set".to_string())?;

        let jwt_expiration_ms = env::var("JWT_EXPIRATION_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86_400_000); // Default to 24 hours

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        // Render sets PORT dynamically in production
        let server_port = env::var("PORT")
            .or_else(|_| env::var("SERVER_PORT"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        let bcrypt_cost = env::var("BCRYPT_COST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let max_connections = env::var("MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);

        let aws_region = env::var("AWS_REGION")
            .or_else(|_| env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string());

        let aws_ses_from_email = env::var("AWS_SES_FROM_EMAIL").ok().filter(|s| !s.trim().is_empty());

        let password_reset_interval_secs = env::var("PASSWORD_RESET_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let password_reset_expiration_mins = env::var("PASSWORD_RESET_EXPIRATION_MINS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_expiration_ms,
            cors_allowed_origins,
            server_host,
            server_port,
            bcrypt_cost,
            max_connections,
            aws_region,
            aws_ses_from_email,
            password_reset_interval_secs,
            password_reset_expiration_mins,
        })
    }
}