use crate::errors::AppError;
use crate::middleware::auth::{AppState, AuthUser};
use crate::models::{MessageResponse, ThemeRequest};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn update_theme(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<ThemeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let theme = payload.theme.trim().to_lowercase();
    if theme != "light" && theme != "dark" {
        return Err(AppError::BadRequest("Theme must be 'light' or 'dark'".to_string()));
    }

    sqlx::query("UPDATE users SET theme = $1 WHERE id = $2")
        .bind(theme)
        .bind(user.id)
        .execute(state.pool.as_ref())
        .await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Theme updated successfully".to_string(),
        }),
    ))
}