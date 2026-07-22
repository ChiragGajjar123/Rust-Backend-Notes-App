use std::env;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use native_tls::TlsConnector;
use postgres::NoTls;
use postgres_native_tls::MakeTlsConnector;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
use vercel_runtime::{Request, Response, ResponseBody};

static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

#[derive(Clone)]
struct AppConfig {
    jwt_secret: String,
    jwt_expiration_ms: i64,
    cors_allowed_origins: Vec<String>,
}

enum DbPool {
    Tls(Pool<PostgresConnectionManager<MakeTlsConnector>>),
    Plain(Pool<PostgresConnectionManager<NoTls>>),
}

enum DbConn {
    Tls(r2d2::PooledConnection<PostgresConnectionManager<MakeTlsConnector>>),
    Plain(r2d2::PooledConnection<PostgresConnectionManager<NoTls>>),
}

impl DbPool {
    fn get(&self) -> Result<DbConn, r2d2::Error> {
        match self {
            DbPool::Tls(pool) => pool.get().map(DbConn::Tls),
            DbPool::Plain(pool) => pool.get().map(DbConn::Plain),
        }
    }
}

impl DbConn {
    fn client(&mut self) -> &mut postgres::Client {
        match self {
            DbConn::Tls(conn) => conn,
            DbConn::Plain(conn) => conn,
        }
    }
}

struct AppState {
    db: DbPool,
    config: AppConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
}

#[derive(Debug)]
struct ApiError {
    status: u16,
    message: String,
    json_message: bool,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: 400,
            message: message.into(),
            json_message: true,
        }
    }

    fn unauthorized() -> Self {
        Self {
            status: 401,
            message: "Unauthorized".to_string(),
            json_message: false,
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: 403,
            message: message.into(),
            json_message: false,
        }
    }

    fn not_found() -> Self {
        Self {
            status: 404,
            message: String::new(),
            json_message: false,
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: 500,
            message: message.into(),
            json_message: true,
        }
    }
}

struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password: String,
    theme: String,
}

struct NoteRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    content: String,
    tags: Vec<String>,
    color: String,
    pinned: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct SignupRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct ThemeRequest {
    theme: String,
}

#[derive(Deserialize)]
struct NoteRequest {
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    color: Option<String>,
    pinned: Option<bool>,
}

struct NoteFields {
    title: String,
    content: String,
    tags: Vec<String>,
    color: String,
    pinned: bool,
}

fn now_seconds() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

fn status_text(code: u16) -> &'static str {
    match code {
        200 => "Ok",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Internal Server Error",
    }
}

fn get_cors_origin(headers: &http::HeaderMap, state: &AppState) -> String {
    let origin = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if origin.is_empty() {
        return state
            .config
            .cors_allowed_origins
            .first()
            .cloned()
            .unwrap_or_else(|| "*".to_string());
    }

    for allowed in &state.config.cors_allowed_origins {
        if allowed == "*" || allowed == origin {
            return origin.to_string();
        }
        if let Some(suffix) = allowed.strip_prefix('*') {
            if origin.ends_with(suffix) {
                return origin.to_string();
            }
        }
    }

    state
        .config
        .cors_allowed_origins
        .first()
        .cloned()
        .unwrap_or_else(|| origin.to_string())
}

fn response_builder(
    cors_origin: &str,
    status: u16,
    content_type: Option<&str>,
) -> http::response::Builder {
    let mut builder = Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", cors_origin)
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Authorization, Content-Type, Accept",
        )
        .header("Access-Control-Allow-Credentials", "true")
        .header("Connection", "close");

    if let Some(content_type) = content_type {
        builder = builder.header("Content-Type", content_type);
    }

    builder
}

fn json_response(
    cors_origin: &str,
    status: u16,
    value: Value,
) -> Result<Response<ResponseBody>, vercel_runtime::Error> {
    let body = serde_json::to_vec(&value)?;
    Ok(
        response_builder(cors_origin, status, Some("application/json"))
            .body(ResponseBody::from(body))?,
    )
}

fn text_response(
    cors_origin: &str,
    status: u16,
    text: &str,
) -> Result<Response<ResponseBody>, vercel_runtime::Error> {
    Ok(
        response_builder(cors_origin, status, Some("text/plain; charset=utf-8"))
            .body(ResponseBody::from(text.to_string()))?,
    )
}

fn empty_response(
    cors_origin: &str,
    status: u16,
) -> Result<Response<ResponseBody>, vercel_runtime::Error> {
    Ok(response_builder(cors_origin, status, None).body(ResponseBody::from(""))?)
}

fn error_response(
    cors_origin: &str,
    err: ApiError,
) -> Result<Response<ResponseBody>, vercel_runtime::Error> {
    if err.message.is_empty() {
        return empty_response(cors_origin, err.status);
    }

    if err.json_message {
        json_response(cors_origin, err.status, json!({ "message": err.message }))
    } else {
        text_response(cors_origin, err.status, &err.message)
    }
}

fn parse_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(bytes).map_err(|_| ApiError::bad_request("Invalid JSON request body"))
}

fn auth_header(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
}

fn generate_jwt(username: &str, config: &AppConfig) -> Result<String, ApiError> {
    let iat = now_seconds();
    let exp = iat + (config.jwt_expiration_ms.max(0) as usize / 1000);
    let claims = Claims {
        sub: username.to_string(),
        iat,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|_| ApiError::internal("Failed to create token"))
}

fn db_conn(state: &AppState) -> Result<DbConn, ApiError> {
    state
        .db
        .get()
        .map_err(|_| ApiError::internal("Database connection error"))
}

fn map_user(row: &postgres::Row) -> UserRow {
    UserRow {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password: row.get("password"),
        theme: row.get("theme"),
    }
}

fn map_note(row: &postgres::Row) -> NoteRow {
    NoteRow {
        id: row.get("id"),
        user_id: row.get("user_id"),
        title: row.get("title"),
        content: row.get("content"),
        tags: row.get("tags"),
        color: row.get("color"),
        pinned: row.get("pinned"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn find_user_by_username(state: &AppState, username: &str) -> Result<Option<UserRow>, ApiError> {
    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_opt(
            "SELECT id, username, email, password, theme FROM users WHERE username = $1",
            &[&username],
        )
        .map_err(|_| ApiError::internal("Database error"))?;
    Ok(row.map(|r| map_user(&r)))
}

fn find_user_by_email(state: &AppState, email: &str) -> Result<Option<UserRow>, ApiError> {
    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_opt(
            "SELECT id, username, email, password, theme FROM users WHERE email = $1",
            &[&email],
        )
        .map_err(|_| ApiError::internal("Database error"))?;
    Ok(row.map(|r| map_user(&r)))
}

fn current_user(headers: &http::HeaderMap, state: &AppState) -> Result<UserRow, ApiError> {
    let token = auth_header(headers).ok_or_else(ApiError::unauthorized)?;
    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::unauthorized())?
    .claims;

    find_user_by_username(state, &claims.sub)?.ok_or_else(ApiError::unauthorized)
}

fn note_to_json(note: &NoteRow) -> Value {
    json!({
        "id": note.id.to_string(),
        "userId": note.user_id.to_string(),
        "title": note.title,
        "content": note.content,
        "tags": note.tags,
        "color": note.color,
        "pinned": note.pinned,
        "createdAt": note.created_at.to_rfc3339(),
        "updatedAt": note.updated_at.to_rfc3339(),
    })
}

fn user_login_json(user: &UserRow, token: String) -> Value {
    json!({
        "accessToken": token,
        "tokenType": "Bearer",
        "id": user.id.to_string(),
        "username": user.username,
        "email": user.email,
        "theme": user.theme,
    })
}

fn note_fields_from_bytes(bytes: &[u8]) -> Result<NoteFields, ApiError> {
    let note_request: NoteRequest = parse_json(bytes)?;
    Ok(NoteFields {
        title: note_request.title.unwrap_or_default(),
        content: note_request.content.unwrap_or_default(),
        tags: note_request.tags.unwrap_or_default(),
        color: note_request.color.unwrap_or_default(),
        pinned: note_request.pinned.unwrap_or(false),
    })
}

fn handle_login(body_bytes: &[u8], state: &AppState) -> Result<Value, ApiError> {
    let login: LoginRequest = parse_json(body_bytes)?;
    let user = find_user_by_email(state, &login.email)?
        .ok_or_else(|| ApiError::bad_request("Bad credentials"))?;
    if !verify(&login.password, &user.password).unwrap_or(false) {
        return Err(ApiError::bad_request("Bad credentials"));
    }
    let token = generate_jwt(&user.username, &state.config)?;
    Ok(user_login_json(&user, token))
}

fn handle_signup(body_bytes: &[u8], state: &AppState) -> Result<Value, ApiError> {
    let signup: SignupRequest = parse_json(body_bytes)?;
    if signup.username.trim().len() < 3
        || signup.username.len() > 100
        || signup.email.trim().is_empty()
        || signup.email.len() > 50
        || !signup.email.contains('@')
        || signup.password.len() < 6
        || signup.password.len() > 40
    {
        return Err(ApiError::bad_request("Invalid signup request"));
    }

    if find_user_by_username(state, &signup.username)?.is_some() {
        return Err(ApiError::bad_request("Error: Username is already taken!"));
    }
    if find_user_by_email(state, &signup.email)?.is_some() {
        return Err(ApiError::bad_request("Error: Email is already in use!"));
    }

    let password = hash(signup.password, DEFAULT_COST)
        .map_err(|_| ApiError::internal("Failed to hash password"))?;

    let mut conn = db_conn(state)?;
    conn.client()
        .execute(
            "INSERT INTO users (username, email, password, theme) VALUES ($1, $2, $3, 'light')",
            &[&signup.username, &signup.email, &password],
        )
        .map_err(|_| ApiError::internal("Database error"))?;

    Ok(json!({ "message": "User registered successfully!" }))
}

fn handle_update_theme(headers: &http::HeaderMap, body_bytes: &[u8], state: &AppState) -> Result<Value, ApiError> {
    let user = current_user(headers, state)?;
    let theme_request: ThemeRequest = parse_json(body_bytes)?;
    if theme_request.theme != "light" && theme_request.theme != "dark" {
        return Err(ApiError {
            status: 400,
            message: "Invalid theme preference. Must be 'light' or 'dark'.".to_string(),
            json_message: false,
        });
    }

    let mut conn = db_conn(state)?;
    conn.client()
        .execute(
            "UPDATE users SET theme = $1 WHERE id = $2",
            &[&theme_request.theme, &user.id],
        )
        .map_err(|_| ApiError::internal("Database error"))?;

    Ok(json!({
        "message": "Theme updated successfully",
        "theme": theme_request.theme
    }))
}

fn handle_get_notes(headers: &http::HeaderMap, state: &AppState) -> Result<Value, ApiError> {
    let user = current_user(headers, state)?;
    let mut conn = db_conn(state)?;
    let rows = conn
        .client()
        .query(
            "SELECT id, user_id, title, content, tags, color, pinned, created_at, updated_at \
             FROM notes WHERE user_id = $1 \
             ORDER BY pinned DESC, updated_at DESC",
            &[&user.id],
        )
        .map_err(|_| ApiError::internal("Database error"))?;

    let notes = rows
        .iter()
        .map(|row| note_to_json(&map_note(row)))
        .collect::<Vec<_>>();
    Ok(Value::Array(notes))
}

fn handle_create_note(headers: &http::HeaderMap, body_bytes: &[u8], state: &AppState) -> Result<Value, ApiError> {
    let user = current_user(headers, state)?;
    let fields = note_fields_from_bytes(body_bytes)?;

    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_one(
            "INSERT INTO notes (user_id, title, content, tags, color, pinned) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at",
            &[
                &user.id,
                &fields.title,
                &fields.content,
                &fields.tags,
                &fields.color,
                &fields.pinned,
            ],
        )
        .map_err(|_| ApiError::internal("Database error"))?;

    Ok(note_to_json(&map_note(&row)))
}

fn parse_note_id(id: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|_| ApiError::not_found())
}

fn find_note_by_id(state: &AppState, id: Uuid) -> Result<Option<NoteRow>, ApiError> {
    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_opt(
            "SELECT id, user_id, title, content, tags, color, pinned, created_at, updated_at \
             FROM notes WHERE id = $1",
            &[&id],
        )
        .map_err(|_| ApiError::internal("Database error"))?;
    Ok(row.map(|r| map_note(&r)))
}

fn handle_update_note(headers: &http::HeaderMap, body_bytes: &[u8], state: &AppState, id: &str) -> Result<Value, ApiError> {
    let user = current_user(headers, state)?;
    let note_id = parse_note_id(id)?;
    let existing = find_note_by_id(state, note_id)?.ok_or_else(ApiError::not_found)?;

    if existing.user_id != user.id {
        return Err(ApiError::forbidden(
            "Error: You don't have permission to update this note.",
        ));
    }

    let fields = note_fields_from_bytes(body_bytes)?;

    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_one(
            "UPDATE notes SET title = $1, content = $2, tags = $3, color = $4, pinned = $5, \
             updated_at = NOW() \
             WHERE id = $6 \
             RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at",
            &[
                &fields.title,
                &fields.content,
                &fields.tags,
                &fields.color,
                &fields.pinned,
                &note_id,
            ],
        )
        .map_err(|_| ApiError::internal("Database error"))?;

    Ok(note_to_json(&map_note(&row)))
}

fn handle_delete_note(headers: &http::HeaderMap, state: &AppState, id: &str) -> Result<(), ApiError> {
    let user = current_user(headers, state)?;
    let note_id = parse_note_id(id)?;
    let existing = find_note_by_id(state, note_id)?.ok_or_else(ApiError::not_found)?;

    if existing.user_id != user.id {
        return Err(ApiError::forbidden(
            "Error: You don't have permission to delete this note.",
        ));
    }

    let mut conn = db_conn(state)?;
    conn.client()
        .execute("DELETE FROM notes WHERE id = $1", &[&note_id])
        .map_err(|_| ApiError::internal("Database error"))?;
    Ok(())
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (name == key).then_some(value)
    })
}

fn request_path(uri: &http::Uri) -> String {
    if let Some(path) = uri
        .query()
        .and_then(|query| query_param(query, "path"))
    {
        let path = path.replace("%2F", "/").replace("%2f", "/");
        if path.is_empty() {
            return "/api".to_string();
        }
        return format!("/api/{}", path.trim_start_matches('/'));
    }

    uri.path().to_string()
}

fn handle_request_sync(
    parts: http::request::Parts,
    body_bytes: Vec<u8>,
) -> Result<Response<ResponseBody>, vercel_runtime::Error> {
    let state = match app_state() {
        Ok(state) => state,
        Err(err) => {
            return Ok(Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(
                    json!({ "message": err.message }).to_string(),
                ))?);
        }
    };

    let method = parts.method.as_str().to_string();
    let path = request_path(&parts.uri);
    let cors_origin = get_cors_origin(&parts.headers, state);

    if method == "OPTIONS" {
        return empty_response(&cors_origin, 204);
    }

    let result = match (method.as_str(), path.as_str()) {
        ("POST", "/api/auth/login") => handle_login(&body_bytes, state).map(|body| (200, body)),
        ("POST", "/api/auth/signup") => handle_signup(&body_bytes, state).map(|body| (200, body)),
        ("PUT", "/api/users/theme") => handle_update_theme(&parts.headers, &body_bytes, state)
            .map(|body| (200, body)),
        ("GET", "/api/notes") => handle_get_notes(&parts.headers, state).map(|body| (200, body)),
        ("POST", "/api/notes") => handle_create_note(&parts.headers, &body_bytes, state).map(|body| (201, body)),
        _ if method == "PUT" && path.starts_with("/api/notes/") => {
            let id = path.trim_start_matches("/api/notes/");
            handle_update_note(&parts.headers, &body_bytes, state, id)
                .map(|body| (200, body))
        }
        _ if method == "DELETE" && path.starts_with("/api/notes/") => {
            let id = path.trim_start_matches("/api/notes/");
            match handle_delete_note(&parts.headers, state, id) {
                Ok(()) => return empty_response(&cors_origin, 200),
                Err(err) => return error_response(&cors_origin, err),
            }
        }
        _ => Err(ApiError::not_found()),
    };

    match result {
        Ok((status, body)) => json_response(&cors_origin, status, body),
        Err(err) => error_response(&cors_origin, err),
    }
}

pub async fn handle_request(req: Request) -> Result<Response<ResponseBody>, vercel_runtime::Error> {
    let (parts, body) = req.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes().to_vec(),
        Err(_) => Vec::new(),
    };

    tokio::task::spawn_blocking(move || handle_request_sync(parts, body_bytes))
        .await
        .map_err(|e| vercel_runtime::Error::from(e.to_string()))?
}

fn app_state() -> Result<&'static AppState, ApiError> {
    // Fast path: already initialised successfully.
    if let Some(state) = APP_STATE.get() {
        return Ok(state.as_ref());
    }

    // Slow path: try to initialise. Errors are NOT cached so that a
    // transient DB hiccup doesn't permanently poison the warm instance.
    let state = load_app_state().map_err(|msg| ApiError::internal(msg))?;
    let _ = APP_STATE.set(state);
    Ok(APP_STATE.get().expect("APP_STATE was just set").as_ref())
}

fn load_app_state() -> Result<Arc<AppState>, String> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("NEON_DATABASE_URL"))
        .map_err(|_| "DATABASE_URL is not configured".to_string())?;
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        "secretKeyNotesAppJWTSecretTokenGeneratorCustomString1234567890!".into()
    });
    let jwt_expiration_ms = env::var("JWT_EXPIRATION_MS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(86_400_000);
    let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGIN")
        .map(|val| {
            val.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(|_| {
            vec![
                "http://localhost:5173".to_string(),
                "*.vercel.app".to_string(),
                "*.netlify.app".to_string(),
            ]
        });

    let db = create_pool(&database_url)?;
    if let Err(e) = ensure_schema(&db) {
        eprintln!("Warning: ensure_schema returned: {}", e);
    }

    Ok(Arc::new(AppState {
        db,
        config: AppConfig {
            jwt_secret,
            jwt_expiration_ms,
            cors_allowed_origins,
        },
    }))
}

fn wants_tls(database_url: &str) -> bool {
    let lower = database_url.to_ascii_lowercase();
    if lower.contains("sslmode=disable") {
        return false;
    }
    if lower.contains("neon.tech")
        || lower.contains("sslmode=require")
        || lower.contains("sslmode=verify")
    {
        return true;
    }
    !(lower.contains("localhost") || lower.contains("127.0.0.1"))
}

fn create_pool(database_url: &str) -> Result<DbPool, String> {
    let config: postgres::Config = database_url
        .parse()
        .map_err(|e| format!("invalid DATABASE_URL / connection string: {}", e))?;

    if wants_tls(database_url) {
        let connector = TlsConnector::builder()
            .build()
            .map_err(|_| "failed to create TLS connector".to_string())?;
        let tls = MakeTlsConnector::new(connector);
        let manager = PostgresConnectionManager::new(config, tls);
        Pool::builder()
            .max_size(2)
            .connection_timeout(Duration::from_secs(5))
            .build(manager)
            .map(DbPool::Tls)
            .map_err(|_| "failed to create Postgres pool (TLS)".to_string())
    } else {
        let manager = PostgresConnectionManager::new(config, NoTls);
        Pool::builder()
            .max_size(2)
            .connection_timeout(Duration::from_secs(5))
            .build(manager)
            .map(DbPool::Plain)
            .map_err(|_| "failed to create Postgres pool".to_string())
    }
}

fn ensure_schema(pool: &DbPool) -> Result<(), String> {
    let mut conn = pool
        .get()
        .map_err(|_| "failed to get DB connection for schema setup".to_string())?;
    conn.client()
        .batch_execute(
            r#"
            CREATE EXTENSION IF NOT EXISTS "pgcrypto";

            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(100) NOT NULL UNIQUE,
                email VARCHAR(50) NOT NULL UNIQUE,
                password VARCHAR(120) NOT NULL,
                theme VARCHAR(10) NOT NULL DEFAULT 'light'
            );

            CREATE TABLE IF NOT EXISTS notes (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                tags TEXT[] NOT NULL DEFAULT '{}',
                color TEXT NOT NULL DEFAULT '',
                pinned BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes(user_id);
            CREATE INDEX IF NOT EXISTS idx_notes_user_pinned_updated
                ON notes(user_id, pinned DESC, updated_at DESC);
            "#,
        )
        .map_err(|_| "failed to initialize database schema".to_string())
}

#[allow(dead_code)]
fn _status_text_for_contract(status: u16) -> &'static str {
    status_text(status)
}
