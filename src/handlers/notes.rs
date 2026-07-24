use crate::models::{NoteRequest, NoteResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sqlx::Pool;
use uuid::Uuid;

pub async fn list_notes(
    State(state): State<crate::middleware::auth::AppState>,
    crate::middleware::auth::AuthUser(user): crate::middleware::auth::AuthUser,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let notes = sqlx::query_as!(
        crate::models::Note,
        r#"SELECT id, user_id, title, content, tags, color, pinned, created_at, updated_at
           FROM notes WHERE user_id = $1
           ORDER BY pinned DESC, updated_at DESC"#,
        user.id
    )
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<NoteResponse> = notes.into_iter().map(NoteResponse::from).collect();
    Ok((StatusCode::OK, Json(response)))
}

pub async fn create_note(
    State(state): State<crate::middleware::auth::AppState>,
    crate::middleware::auth::AuthUser(user): crate::middleware::auth::AuthUser,
    Json(payload): Json<NoteRequest>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let note = sqlx::query_as!(
        crate::models::Note,
        r#"INSERT INTO notes (user_id, title, content, tags, color, pinned)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at"#,
        user.id,
        payload.title.unwrap_or_default(),
        payload.content.unwrap_or_default(),
        payload.tags.unwrap_or_default(),
        payload.color.unwrap_or_default(),
        payload.pinned.unwrap_or(false)
    )
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(NoteResponse::from(note))))
}

pub async fn update_note(
    State(state): State<crate::middleware::auth::AppState>,
    crate::middleware::auth::AuthUser(user): crate::middleware::auth::AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<NoteRequest>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let existing = sqlx::query!(
        "SELECT id FROM notes WHERE id = $1 AND user_id = $2",
        id,
        user.id
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let note = sqlx::query_as!(
        crate::models::Note,
        r#"UPDATE notes SET title = $1, content = $2, tags = $3, color = $4, pinned = $5, updated_at = NOW()
           WHERE id = $6
           RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at"#,
        payload.title.unwrap_or_default(),
        payload.content.unwrap_or_default(),
        payload.tags.unwrap_or_default(),
        payload.color.unwrap_or_default(),
        payload.pinned.unwrap_or(false),
        id
    )
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(NoteResponse::from(note))))
}

pub async fn delete_note(
    State(state): State<crate::middleware::auth::AppState>,
    crate::middleware::auth::AuthUser(user): crate::middleware::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM notes WHERE id = $1 AND user_id = $2",
        id,
        user.id
    )
    .execute(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok((StatusCode::OK, Json(crate::models::MessageResponse {
        message: "Note deleted successfully".to_string(),
    })))
}