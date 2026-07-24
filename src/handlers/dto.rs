use crate::models::Note;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub theme: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ThemeRequest {
    pub theme: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct NoteCreateRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NoteUpdateRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub color: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Note> for NoteResponse {
    fn from(note: Note) -> Self {
        Self {
            id: note.id.to_string(),
            user_id: note.user_id.to_string(),
            title: note.title,
            content: note.content,
            tags: note.tags,
            color: note.color,
            pinned: note.pinned,
            created_at: note.created_at.to_rfc3339(),
            updated_at: note.updated_at.to_rfc3339(),
        }
    }
}