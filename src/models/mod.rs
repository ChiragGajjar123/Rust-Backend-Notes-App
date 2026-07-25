use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub theme: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub color: String,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub theme: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            theme: user.theme,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ThemeRequest {
    pub theme: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct NoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasswordReset {
    pub id: Uuid,
    pub email: String,
    pub otp_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyResetCodeRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}