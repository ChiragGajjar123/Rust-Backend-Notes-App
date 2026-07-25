use crate::middleware::auth::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(crate::handlers::health::health_check))
        .route("/health", get(crate::handlers::health::health_check))
}

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/signup", post(crate::handlers::auth::signup))
        .route("/auth/me", get(crate::handlers::auth::get_current_user))
        .route("/auth/forgot-password", post(crate::handlers::auth::forgot_password))
        .route("/auth/verify-reset-code", post(crate::handlers::auth::verify_reset_code))
        .route("/auth/reset-password", post(crate::handlers::auth::reset_password))
}

pub fn notes_routes() -> Router<AppState> {
    Router::new()
        .route("/notes", get(crate::handlers::notes::list_notes))
        .route("/notes", post(crate::handlers::notes::create_note))
        .route("/notes/:id", put(crate::handlers::notes::update_note))
        .route("/notes/:id", delete(crate::handlers::notes::delete_note))
}

pub fn user_routes() -> Router<AppState> {
    Router::new().route("/users/theme", put(crate::handlers::users::update_theme))
}

pub fn all_routes() -> Router<AppState> {
    Router::new()
        .merge(health_routes())
        .merge(auth_routes())
        .merge(notes_routes())
        .merge(user_routes())
}