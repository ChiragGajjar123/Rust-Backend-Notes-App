use crate::errors::AppError;
use crate::middleware::auth::{AppState, AuthUser};
use crate::models::{AuthResponse, LoginRequest, SignupRequest, User, UserResponse};
use crate::utils::jwt::generate_jwt;
use crate::utils::password::{hash_password, verify_password};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = payload.email.trim();
    if email.is_empty() || payload.password.is_empty() {
        return Err(AppError::BadRequest("Email and password are required".to_string()));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password, theme, created_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(state.pool.as_ref())
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let is_valid = verify_password(payload.password, user.password.clone()).await?;
    if !is_valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let token = generate_jwt(
        &user.username,
        &state.config.jwt_secret,
        state.config.jwt_expiration_ms,
    )?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            user: UserResponse::from(user),
        }),
    ))
}

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    let username = payload.username.trim();
    let email = payload.email.trim();

    if username.len() < 3 || username.len() > 100 {
        return Err(AppError::BadRequest("Username must be between 3 and 100 characters".to_string()));
    }
    if email.is_empty() || email.len() > 255 || !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }
    if payload.password.len() < 6 || payload.password.len() > 100 {
        return Err(AppError::BadRequest("Password must be between 6 and 100 characters".to_string()));
    }

    let existing_user = sqlx::query("SELECT id FROM users WHERE username = $1 OR email = $2")
        .bind(username)
        .bind(email)
        .fetch_optional(state.pool.as_ref())
        .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Username or email is already registered".to_string()));
    }

    let password_hash = hash_password(payload.password, state.config.bcrypt_cost).await?;

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (username, email, password, theme)
           VALUES ($1, $2, $3, 'light')
           RETURNING id, username, email, password, theme, created_at"#,
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(state.pool.as_ref())
    .await?;

    let token = generate_jwt(
        &user.username,
        &state.config.jwt_secret,
        state.config.jwt_expiration_ms,
    )?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            user: UserResponse::from(user),
        }),
    ))
}

pub async fn get_current_user(
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}