use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn auth_routes() -> Router<crate::middleware::auth::AppState> {
    Router::new()
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/signup", post(crate::handlers::auth::signup))
}

pub fn notes_routes() -> Router<crate::middleware::auth::AppState> {
    Router::new()
        .route("/notes", get(crate::handlers::notes::list_notes))
        .route("/notes", post(crate::handlers::notes::create_note))
        .route("/notes/:id", put(crate::handlers::notes::update_note))
        .route("/notes/:id", delete(crate::handlers::notes::delete_note))
        .layer(middleware::from_fn(crate::middleware::auth::auth_middleware))
}

pub fn user_routes() -> Router<crate::middleware::auth::AppState> {
    Router::new()
        .route("/users/theme", put(crate::handlers::users::update_theme))
        .layer(middleware::from_fn(crate::middleware::auth::auth_middleware))
}

pub fn all_routes() -> Router<crate::middleware::auth::AppState> {
    Router::new()
        .merge(auth_routes())
        .merge(notes_routes())
        .merge(user_routes())
}