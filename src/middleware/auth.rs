use crate::config::Config;
use crate::models::User;
use crate::utils::jwt::validate_jwt;
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    RequestPartsExt,
    response::Response,
};
use sqlx::Pool;
use std::sync::Arc;

pub struct AppState {
    pub config: Arc<Config>,
    pub pool: Arc<Pool<sqlx::Postgres>>,
}

#[derive(Clone)]
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRequestParts<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_request_parts(parts, state).await?;
        let config = &app_state.config;
        let pool = &app_state.pool;

        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization token".to_string()))?;

        let claims = validate_jwt(token, &config.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password, theme, created_at FROM users WHERE username = $1"
        )
        .bind(&claims.sub)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

        Ok(AuthUser(user))
    }
}

impl AppState {
    pub fn new(config: Arc<Config>, pool: Arc<Pool<sqlx::Postgres>>) -> Self {
        Self { config, pool }
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization token".to_string()))?;

    let claims = validate_jwt(token, &state.config.jwt_secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password, theme, created_at FROM users WHERE username = $1"
    )
    .bind(&claims.sub)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    request.extensions_mut().insert(AuthUser(user));

    Ok(next.run(request).await)
}