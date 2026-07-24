use crate::models::{MessageResponse, ThemeRequest};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};

pub async fn update_theme(
    State(state): State<crate::middleware::auth::AppState>,
    crate::middleware::auth::AuthUser(user): crate::middleware::auth::AuthUser,
    Json(payload): Json<ThemeRequest>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    if payload.theme != "light" && payload.theme != "dark" {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query!("UPDATE users SET theme = $1 WHERE id = $2", payload.theme, user.id)
        .execute(state.pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(MessageResponse {
        message: "Theme updated successfully".to_string(),
    })))
}