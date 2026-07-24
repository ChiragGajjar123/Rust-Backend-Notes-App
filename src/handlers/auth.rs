use crate::models::{AuthResponse, LoginRequest, SignupRequest, UserResponse};
use crate::utils::jwt::generate_jwt;
use crate::utils::password::{hash_password, verify_password};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::Pool;
use std::sync::Arc;

pub async fn login(
    State(state): State<crate::middleware::auth::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = sqlx::query_as!(
        crate::models::User,
        "SELECT id, username, email, password, theme, created_at FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::BAD_REQUEST)?;

    verify_password(&payload.password, &user.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .then_some(())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let token = generate_jwt(&user.username, &state.config.jwt_secret, state.config.jwt_expiration_ms)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        user: UserResponse::from(user),
    })))
}

pub async fn signup(
    State(state): State<crate::middleware::auth::AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if payload.username.trim().len() < 3
        || payload.username.len() > 100
        || payload.email.trim().is_empty()
        || payload.email.len() > 50
        || !payload.email.contains('@')
        || payload.password.len() < 6
        || payload.password.len() > 40
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        payload.username,
        payload.email
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let password_hash = hash_password(&payload.password, state.config.bcrypt_cost)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = sqlx::query_as!(
        crate::models::User,
        r#"INSERT INTO users (username, email, password, theme) VALUES ($1, $2, $3, 'light')
           RETURNING id, username, email, password, theme, created_at"#,
        payload.username,
        payload.email,
        password_hash
    )
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = generate_jwt(&user.username, &state.config.jwt_secret, state.config.jwt_expiration_ms)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        user: UserResponse::from(user),
    })))
}

pub async fn get_current_user(
    State(state): State<crate::middleware::auth::AppState>,
    crate::middleware::auth::AuthUser(user): crate::middleware::auth::AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}