use std::env;
use std::io::{self, Read};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bcrypt::{DEFAULT_COST, hash, verify};
use bytes::BufMut;
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use may_minihttp::{HttpServer, HttpService, Request, Response};
use native_tls::TlsConnector;
use postgres::NoTls;
use postgres_native_tls::MakeTlsConnector;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// App state / config
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppConfig {
    jwt_secret: String,
    jwt_expiration_ms: i64,
    cors_origin_header: &'static str,
}

/// Connection pool that works for Neon (TLS) and local Postgres (optional NoTls).
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

#[derive(Clone)]
struct NotesService {
    state: Arc<AppState>,
}

// ---------------------------------------------------------------------------
// Models / DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
}

#[derive(Debug)]
struct ApiError {
    status: usize,
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

#[derive(Clone)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password: String,
    theme: String,
}

#[derive(Clone)]
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

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

fn now_seconds() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

fn status_text(code: usize) -> &'static str {
    match code {
        200 => "Ok",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    }
}

fn get_cors_origin(req: &Request, state: &AppState) -> &'static str {
    let origin = req.headers()
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("origin"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .unwrap_or("");

    if origin == "http://localhost:5173" {
        "Access-Control-Allow-Origin: http://localhost:5173"
    } else {
        state.config.cors_origin_header
    }
}

fn add_common_headers(rsp: &mut Response, cors_origin: &'static str) {
    rsp.header(cors_origin)
        .header("Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers: Authorization, Content-Type, Accept")
        .header("Access-Control-Allow-Credentials: true")
        .header("Connection: close");
}

fn json_response(
    rsp: &mut Response,
    cors_origin: &'static str,
    status: usize,
    value: Value,
) -> io::Result<()> {
    rsp.status_code(status, status_text(status));
    add_common_headers(rsp, cors_origin);
    rsp.header("Content-Type: application/json");
    let bytes = serde_json::to_vec(&value)?;
    rsp.body_vec(bytes);
    Ok(())
}

fn text_response(rsp: &mut Response, cors_origin: &'static str, status: usize, text: &str) {
    rsp.status_code(status, status_text(status));
    add_common_headers(rsp, cors_origin);
    rsp.header("Content-Type: text/plain; charset=utf-8");
    rsp.body_vec(text.as_bytes().to_vec());
}

fn empty_response(rsp: &mut Response, cors_origin: &'static str, status: usize) {
    rsp.status_code(status, status_text(status));
    add_common_headers(rsp, cors_origin);
    rsp.body("");
}

fn error_response(rsp: &mut Response, cors_origin: &'static str, err: ApiError) -> io::Result<()> {
    if err.message.is_empty() {
        empty_response(rsp, cors_origin, err.status);
        return Ok(());
    }
    if err.json_message {
        json_response(rsp, cors_origin, err.status, json!({ "message": err.message }))
    } else {
        text_response(rsp, cors_origin, err.status, &err.message);
        Ok(())
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(req: Request) -> Result<T, ApiError> {
    let mut body = req.body();
    let mut bytes = Vec::new();
    body.read_to_end(&mut bytes)
        .map_err(|_| ApiError::bad_request("Invalid request body"))?;
    serde_json::from_slice(&bytes).map_err(|_| ApiError::bad_request("Invalid JSON request body"))
}

fn auth_header(req: &Request) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("authorization"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
}

// ---------------------------------------------------------------------------
// Auth (JWT + bcrypt) — same contract as Spring backend
// ---------------------------------------------------------------------------

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

fn current_user(req: &Request, state: &AppState) -> Result<UserRow, ApiError> {
    let token = auth_header(req).ok_or_else(ApiError::unauthorized)?;
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

// ---------------------------------------------------------------------------
// Handlers — same routes / status codes / payloads as Spring API
// ---------------------------------------------------------------------------

fn handle_login(req: Request, state: &AppState) -> Result<Value, ApiError> {
    let login: LoginRequest = read_json(req)?;
    let user = find_user_by_email(state, &login.email)?
        .ok_or_else(|| ApiError::bad_request("Bad credentials"))?;
    if !verify(&login.password, &user.password).unwrap_or(false) {
        return Err(ApiError::bad_request("Bad credentials"));
    }
    let token = generate_jwt(&user.username, &state.config)?;
    Ok(user_login_json(&user, token))
}

fn handle_signup(req: Request, state: &AppState) -> Result<Value, ApiError> {
    let signup: SignupRequest = read_json(req)?;
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

fn handle_update_theme(req: Request, state: &AppState) -> Result<Value, ApiError> {
    let user = current_user(&req, state)?;
    let theme_request: ThemeRequest = read_json(req)?;
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

fn handle_get_notes(req: Request, state: &AppState) -> Result<Value, ApiError> {
    let user = current_user(&req, state)?;
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

fn handle_create_note(req: Request, state: &AppState) -> Result<Value, ApiError> {
    let user = current_user(&req, state)?;
    let note_request: NoteRequest = read_json(req)?;

    let title = note_request.title.unwrap_or_default();
    let content = note_request.content.unwrap_or_default();
    let tags = note_request.tags.unwrap_or_default();
    let color = note_request.color.unwrap_or_default();
    let pinned = note_request.pinned.unwrap_or(false);

    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_one(
            "INSERT INTO notes (user_id, title, content, tags, color, pinned) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at",
            &[&user.id, &title, &content, &tags, &color, &pinned],
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

fn handle_update_note(req: Request, state: &AppState, id: &str) -> Result<Value, ApiError> {
    let user = current_user(&req, state)?;
    let note_id = parse_note_id(id)?;
    let existing = find_note_by_id(state, note_id)?.ok_or_else(ApiError::not_found)?;

    if existing.user_id != user.id {
        return Err(ApiError::forbidden(
            "Error: You don't have permission to update this note.",
        ));
    }

    let note_request: NoteRequest = read_json(req)?;
    let title = note_request.title.unwrap_or_default();
    let content = note_request.content.unwrap_or_default();
    let tags = note_request.tags.unwrap_or_default();
    let color = note_request.color.unwrap_or_default();
    let pinned = note_request.pinned.unwrap_or(false);

    let mut conn = db_conn(state)?;
    let row = conn
        .client()
        .query_one(
            "UPDATE notes SET title = $1, content = $2, tags = $3, color = $4, pinned = $5, \
             updated_at = NOW() \
             WHERE id = $6 \
             RETURNING id, user_id, title, content, tags, color, pinned, created_at, updated_at",
            &[&title, &content, &tags, &color, &pinned, &note_id],
        )
        .map_err(|_| ApiError::internal("Database error"))?;

    Ok(note_to_json(&map_note(&row)))
}

fn handle_delete_note(req: Request, state: &AppState, id: &str) -> Result<(), ApiError> {
    let user = current_user(&req, state)?;
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

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

impl HttpService for NotesService {
    fn call(&mut self, req: Request, rsp: &mut Response) -> io::Result<()> {
        let state = self.state.as_ref();

        let method = req.method().to_string();
        let path = req.path().split('?').next().unwrap_or("").to_string();
        println!("Received request: {} {}", method, path);

        let cors_origin = get_cors_origin(&req, state);

        if req.method() == "OPTIONS" {
            empty_response(rsp, cors_origin, 204);
            return Ok(());
        }

        let result = match (method.as_str(), path.as_str()) {
            ("POST", "/api/auth/login") => handle_login(req, state).map(|body| (200, body)),
            ("POST", "/api/auth/signup") => handle_signup(req, state).map(|body| (200, body)),
            ("PUT", "/api/users/theme") => handle_update_theme(req, state).map(|body| (200, body)),
            ("GET", "/api/notes") => handle_get_notes(req, state).map(|body| (200, body)),
            ("POST", "/api/notes") => handle_create_note(req, state).map(|body| (201, body)),
            _ if method == "PUT" && path.starts_with("/api/notes/") => {
                let id = path.trim_start_matches("/api/notes/");
                handle_update_note(req, state, id).map(|body| (200, body))
            }
            _ if method == "DELETE" && path.starts_with("/api/notes/") => {
                let id = path.trim_start_matches("/api/notes/");
                match handle_delete_note(req, state, id) {
                    Ok(()) => {
                        empty_response(rsp, cors_origin, 200);
                        return Ok(());
                    }
                    Err(err) => {
                        return error_response(rsp, cors_origin, err);
                    }
                }
            }
            _ => Err(ApiError::not_found()),
        };

        match result {
            Ok((status, body)) => json_response(rsp, cors_origin, status, body),
            Err(err) => error_response(rsp, cors_origin, err),
        }
    }
}

// ---------------------------------------------------------------------------
// DB bootstrap (Neon Postgres)
// ---------------------------------------------------------------------------

fn wants_tls(database_url: &str) -> bool {
    let lower = database_url.to_ascii_lowercase();
    if lower.contains("sslmode=disable") {
        return false;
    }
    // Neon always needs TLS; local defaults can opt out via sslmode=disable.
    if lower.contains("neon.tech") || lower.contains("sslmode=require") || lower.contains("sslmode=verify")
    {
        return true;
    }
    // Default: TLS for remote hosts, plain for localhost
    !(lower.contains("localhost") || lower.contains("127.0.0.1"))
}

fn create_pool(database_url: &str) -> DbPool {
    let config: postgres::Config = database_url
        .parse()
        .expect("invalid DATABASE_URL / connection string");

    if wants_tls(database_url) {
        let connector = TlsConnector::builder()
            .build()
            .expect("failed to create TLS connector");
        let tls = MakeTlsConnector::new(connector);
        let manager = PostgresConnectionManager::new(config, tls);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .expect("failed to create Postgres pool (TLS)");
        DbPool::Tls(pool)
    } else {
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .expect("failed to create Postgres pool");
        DbPool::Plain(pool)
    }
}

fn ensure_schema(pool: &DbPool) {
    let mut conn = pool.get().expect("failed to get DB connection for schema setup");
    let client = conn.client();

    client
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
        .expect("failed to initialize database schema");
}

fn main() {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("NEON_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/notes_app_db?sslmode=disable".into()
        })
    });
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        "secretKeyNotesAppJWTSecretTokenGeneratorCustomString1234567890!".into()
    });
    let jwt_expiration_ms = env::var("JWT_EXPIRATION_MS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(86_400_000);
    let cors_origin =
        env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "https://react-frontend-for-rust-backend-notes.netlify.app".to_string());
    let cors_origin_header =
        Box::leak(format!("Access-Control-Allow-Origin: {cors_origin}").into_boxed_str());

    if let Some(workers) = env::var("MAY_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
    {
        may::config().set_workers(workers);
    }
    // Set larger stack size to prevent stack overflows in TLS/database handlers on Windows
    may::config().set_stack_size(256 * 1024);

    println!("Connecting to Neon/Postgres...");
    let db = create_pool(&database_url);
    ensure_schema(&db);
    println!("Database schema ready.");

    let state = Arc::new(AppState {
        db,
        config: AppConfig {
            jwt_secret,
            jwt_expiration_ms,
            cors_origin_header,
        },
    });

    let addr = format!("0.0.0.0:{port}");
    println!("Rust notes backend listening on http://{addr}");
    println!("API: /api/auth/login|signup  /api/users/theme  /api/notes");
    let server = HttpServer(NotesService { state })
        .start(addr)
        .expect("failed to start HTTP server");
    server.wait();
}
