# AGENTS.md - Repository Guidelines for AI Assistant Agents

This document provides context, instructions, and coding standards for AI models (e.g. Antigravity, Claude, Copilot, Cursor, ChatGPT) working on this codebase.

---

## 📌 Project Overview

**Notes Application Backend** is a high-performance, asynchronous RESTful API written in Rust.

- **Language:** Rust 2021 Edition
- **Web Framework:** Axum 0.7
- **Async Runtime:** Tokio 1.38
- **Database:** PostgreSQL (via SQLx 0.8) with runtime migrations
- **Authentication:** JWT (`jsonwebtoken`) + Bcrypt password hashing (`bcrypt`)
- **JSON Serialization:** Serde
- **Architecture Type:** Standalone long-running HTTP Web Server (Deployed on AWS EC2 / Docker)

---

## 📂 Repository Structure

```
.
├── Cargo.toml          # Package manifest and dependencies
├── Cargo.lock          # Dependency lockfile
├── .env.example        # Example environment variables template
├── README.md           # General project overview
├── DEPLOYMENT.md       # Step-by-step AWS EC2 & Docker deployment guide
├── AGENTS.md           # Guidelines for AI coding agents
├── migrations/         # SQLx PostgreSQL migration files
└── src/
    ├── main.rs         # Entry point: tracing, config, DB pool, router, server bind
    ├── config.rs       # Environment variable config parser
    ├── db.rs           # Database connection pool setup & migration runner
    ├── models.rs       # Data structures, database entities & API request/response DTOs
    ├── routes.rs       # Router definition and route mapping
    ├── config/         # Modular config components
    ├── db/             # Database connection & query modules
    ├── handlers/       # HTTP request handlers (auth, notes, user theme)
    ├── middleware/     # Custom Axum extractors (AuthUser authentication state)
    ├── models/         # Entity models
    ├── routes/         # Modular API router groups
    └── utils/          # Password hashing, JWT token utilities
```

---

## 🛠️ Common Commands

### Local Development
```powershell
cargo run
```

### Type Checking & Linting
```powershell
cargo check
cargo clippy
```

### Build for Production
```powershell
cargo build --release
```

### SQLx Offline Mode (Before Container Builds)
```powershell
cargo install sqlx-cli
cargo sqlx prepare
```

---

## 🔑 Environment Configuration

Required keys in `.env` for local and production execution:

| Variable | Description | Default / Example |
| :--- | :--- | :--- |
| `PORT` | Listening HTTP server port | `8080` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/dbname?sslmode=require` |
| `JWT_SECRET` | Secret key for signing JWT tokens | `min_32_character_random_string` |
| `JWT_EXPIRATION_MS` | JWT token TTL in milliseconds | `86400000` (24 hours) |
| `CORS_ALLOWED_ORIGIN`| Allowed CORS origin for frontend | `http://localhost:5173` |
| `RUST_LOG` | Tracing filter level | `info,tower_http=debug` |

---

## 📐 Coding Conventions & Guidelines

1. **Async & Axum Patterns:**
   - Use Axum extractors (`State`, `Path`, `Json`, `AuthUser`) for route handlers.
   - Handlers should return `Result<impl IntoResponse, StatusCode>` or structured JSON error responses.

2. **Database Queries:**
   - Use SQLx parameterized queries (`sqlx::query!`, `sqlx::query_as!`) to prevent SQL injection and enable compile-time type verification.
   - Never write dynamic raw string concatenations for database queries.

3. **Error Handling:**
   - Prefer returning explicit HTTP status codes (`StatusCode::BAD_REQUEST`, `StatusCode::UNAUTHORIZED`, `StatusCode::INTERNAL_SERVER_ERROR`).
   - Log detailed internal errors using `tracing::error!` before converting them to client-facing HTTP responses.

4. **Security & Authentication:**
   - Passwords must always be hashed with `bcrypt` before storage.
   - Protected routes must extract user identity using the `AuthUser` extractor from `src/middleware/auth.rs`.

---

## ⚠️ CRITICAL SAFETY RULES FOR AI AGENTS

- **NEVER RUN `git commit` OR `git push` WITHOUT EXPLICIT USER APPROVAL.**
- Do NOT delete or clear existing migrations in `migrations/` without confirmation.
- Do NOT hardcode secrets or database passwords in source code. Always consume from `Config` / `.env`.
