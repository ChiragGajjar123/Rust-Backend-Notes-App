use crate::config::Config;
use crate::errors::AppError;
use crate::models::User;
use crate::utils::jwt::validate_jwt;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use sqlx::Pool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: Arc<Pool<sqlx::Postgres>>,
}

impl AppState {
    pub fn new(config: Arc<Config>, pool: Arc<Pool<sqlx::Postgres>>) -> Self {
        Self { config, pool }
    }
}

#[derive(Clone)]
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let config = &app_state.config;
        let pool = &app_state.pool;

        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("Missing authorization token".to_string()))?;

        let claims = validate_jwt(token, &config.jwt_secret)?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password, theme, created_at FROM users WHERE username = $1",
        )
        .bind(&claims.sub)
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found or account deactivated".to_string()))?;

        Ok(AuthUser(user))
    }
}