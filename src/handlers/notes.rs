use crate::errors::AppError;
use crate::middleware::auth::{AppState, AuthUser};
use crate::models::{MessageResponse, Note, NoteRequest, NoteResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn list_notes(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let notes = sqlx::query_as::<_, Note>(
        r#"SELECT id, user_id, title, content, tags, color, pinned, created_at, updated_at
           FROM notes WHERE user_id = $1
           ORDER BY pinned DESC, updated_at DESC"#,
    )
    .bind(user.id)
    .fetch_all(state.pool.as_ref())
    .await?;

    let response: Vec<NoteResponse> = notes.into_iter().map(NoteResponse::from).collect();
    Ok((StatusCode::OK, Json(response)))
}

pub async fn create_note(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<NoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let title = payload.title.unwrap_or_default();
    let content = payload.content.unwrap_or_default();
    let tags = payload.tags.unwrap_or_default();
    let color = payload.color.unwrap_or_default();
    let pinned = payload.pinned.unwrap_or(false);

    let note = sqlx::query_as::<_, Note>(
        r#"INSERT INTO notes (user_id, title, content, tags, color, pinned)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at"#,
    )
    .bind(user.id)
    .bind(title)
    .bind(content)
    .bind(&tags)
    .bind(color)
    .bind(pinned)
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok((StatusCode::CREATED, Json(NoteResponse::from(note))))
}

pub async fn update_note(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<NoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let existing = sqlx::query_as::<_, Note>(
        r#"SELECT id, user_id, title, content, tags, color, pinned, created_at, updated_at
           FROM notes WHERE id = $1 AND user_id = $2"#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(state.pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Note not found or access denied".to_string()))?;

    let title = payload.title.unwrap_or(existing.title);
    let content = payload.content.unwrap_or(existing.content);
    let tags = payload.tags.unwrap_or(existing.tags);
    let color = payload.color.unwrap_or(existing.color);
    let pinned = payload.pinned.unwrap_or(existing.pinned);

    let updated_note = sqlx::query_as::<_, Note>(
        r#"UPDATE notes
           SET title = $1, content = $2, tags = $3, color = $4, pinned = $5, updated_at = NOW()
           WHERE id = $6 AND user_id = $7
           RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at"#,
    )
    .bind(title)
    .bind(content)
    .bind(&tags)
    .bind(color)
    .bind(pinned)
    .bind(id)
    .bind(user.id)
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok((StatusCode::OK, Json(NoteResponse::from(updated_note))))
}

pub async fn delete_note(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(state.pool.as_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Note not found or access denied".to_string()));
    }

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Note deleted successfully".to_string(),
        }),
    ))
}