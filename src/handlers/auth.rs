use crate::errors::AppError;
use crate::middleware::auth::{AppState, AuthUser};
use crate::models::{
    AuthResponse, ForgotPasswordRequest, LoginRequest, MessageResponse, PasswordReset,
    ResetPasswordRequest, SignupRequest, User, UserResponse, VerifyResetCodeRequest,
};
use crate::utils::aws_email::send_password_reset_email;
use crate::utils::jwt::generate_jwt;
use crate::utils::password::{hash_password, verify_password};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rand::Rng;

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

#[axum::debug_handler]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    let email = payload.email.trim();
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    // 1. Ensure password_resets table and index exist in database
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS password_resets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) NOT NULL,
            otp_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            expires_at TIMESTAMPTZ NOT NULL,
            used BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
    )
    .execute(state.pool.as_ref())
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_password_resets_email_created ON password_resets(email, created_at DESC)",
    )
    .execute(state.pool.as_ref())
    .await?;

    // 2. Check if account exists with this email address
    let user_exists = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(state.pool.as_ref())
        .await?;

    if user_exists.is_none() {
        return Err(AppError::NotFound(
            "No account found with this email address. Please check your email or sign up.".to_string(),
        ));
    }

    // 3. Time interval rate limiting check
    let last_reset = sqlx::query_as::<_, PasswordReset>(
        "SELECT id, email, otp_hash, created_at, expires_at, used FROM password_resets WHERE email = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(email)
    .fetch_optional(state.pool.as_ref())
    .await?;

    if let Some(reset) = last_reset {
        let elapsed_secs = (chrono::Utc::now() - reset.created_at).num_seconds();
        let interval_secs = state.config.password_reset_interval_secs;
        if elapsed_secs < interval_secs {
            let wait_time = interval_secs - elapsed_secs;
            return Err(AppError::TooManyRequests(format!(
                "Please wait {} seconds before requesting another reset code",
                wait_time
            )));
        }
    }

    // 4. Generate 6-digit OTP code
    let otp_code: u32 = rand::thread_rng().gen_range(100000..=999999);
    let otp_str = otp_code.to_string();

    // 5. Hash OTP code for storage
    let otp_hash = hash_password(otp_str.clone(), state.config.bcrypt_cost).await?;
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(state.config.password_reset_expiration_mins);

    // 6. Store OTP record in database
    sqlx::query(
        "INSERT INTO password_resets (email, otp_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(email)
    .bind(otp_hash)
    .bind(expires_at)
    .execute(state.pool.as_ref())
    .await?;

    // 7. Dispatch AWS SES email
    send_password_reset_email(email, &otp_str, &state.config).await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "A 6-digit reset code has been sent to your email. (Please check your spam/junk folder if you don't see it).".to_string(),
        }),
    ))
}

pub async fn verify_reset_code(
    State(state): State<AppState>,
    Json(payload): Json<VerifyResetCodeRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    let email = payload.email.trim();
    let code = payload.code.trim();

    if email.is_empty() || code.is_empty() {
        return Err(AppError::BadRequest("Email and code are required".to_string()));
    }

    let reset_record = sqlx::query_as::<_, PasswordReset>(
        "SELECT id, email, otp_hash, created_at, expires_at, used FROM password_resets WHERE email = $1 AND used = FALSE AND expires_at > NOW() ORDER BY created_at DESC LIMIT 1",
    )
    .bind(email)
    .fetch_optional(state.pool.as_ref())
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid or expired password reset code".to_string()))?;

    let is_valid = verify_password(code.to_string(), reset_record.otp_hash).await?;
    if !is_valid {
        return Err(AppError::BadRequest("Invalid or expired password reset code".to_string()));
    }

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Reset code is valid.".to_string(),
        }),
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    let email = payload.email.trim();
    let code = payload.code.trim();

    if email.is_empty() || code.is_empty() {
        return Err(AppError::BadRequest("Email and code are required".to_string()));
    }
    if payload.new_password.len() < 6 || payload.new_password.len() > 100 {
        return Err(AppError::BadRequest("Password must be between 6 and 100 characters".to_string()));
    }

    let reset_record = sqlx::query_as::<_, PasswordReset>(
        "SELECT id, email, otp_hash, created_at, expires_at, used FROM password_resets WHERE email = $1 AND used = FALSE AND expires_at > NOW() ORDER BY created_at DESC LIMIT 1",
    )
    .bind(email)
    .fetch_optional(state.pool.as_ref())
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid or expired password reset code".to_string()))?;

    let is_valid = verify_password(code.to_string(), reset_record.otp_hash).await?;
    if !is_valid {
        return Err(AppError::BadRequest("Invalid or expired password reset code".to_string()));
    }

    let new_password_hash = hash_password(payload.new_password, state.config.bcrypt_cost).await?;

    // Update user's password
    sqlx::query("UPDATE users SET password = $1 WHERE email = $2")
        .bind(new_password_hash)
        .bind(email)
        .execute(state.pool.as_ref())
        .await?;

    // Mark reset code as used
    sqlx::query("UPDATE password_resets SET used = TRUE WHERE id = $1")
        .bind(reset_record.id)
        .execute(state.pool.as_ref())
        .await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Password has been reset successfully.".to_string(),
        }),
    ))
}