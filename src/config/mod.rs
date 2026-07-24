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
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .or_else(|_| env::var("POSTGRES_URL"))
            .map_err(|_| "DATABASE_URL or POSTGRES_URL must be set".to_string())?;

        let jwt_secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set".to_string())?;
        let jwt_expiration_ms = env::var("JWT_EXPIRATION_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86_400_000);

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
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
            .unwrap_or(100);

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_expiration_ms,
            cors_allowed_origins,
            server_host,
            server_port,
            bcrypt_cost,
            max_connections,
        })
    }
}