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
- **Architecture Type:** Standalone long-running HTTP Web Server (Deployed natively on AWS EC2 via Systemd)

---

## 📂 Repository Structure

```
.
├── Cargo.toml          # Package manifest and dependencies
├── Cargo.lock          # Dependency lockfile
├── .env.example        # Example environment variables template
├── README.md           # General project overview
├── DEPLOYMENT.md       # AWS EC2 Systemd deployment summary
├── AWS_DEPLOYMENT.md   # Step-by-step native AWS EC2 deployment guide
├── AGENTS.md           # Guidelines for AI coding agents
├── migrations/         # SQLx PostgreSQL migration files
└── src/
    ├── main.rs         # Entry point: tracing, config, DB pool, router, server bind
    ├── config/         # Modular config parser
    ├── db/             # Database connection & migration module
    ├── errors.rs       # Centralized AppError handling
    ├── handlers/       # HTTP request handlers (auth, notes, users, health)
    ├── middleware/     # Custom Axum extractors (AuthUser authentication state)
    ├── models/         # Entity models & DTOs
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

---

## 🔑 Environment Configuration

Required keys in `.env` for local and production execution:

| Variable | Description | Default / Example |
| :--- | :--- | :--- |
| `PORT` | Listening HTTP server port | `8080` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/dbname?sslmode=require` |
| `JWT_SECRET` | Secret key for signing JWT tokens | `min_32_character_random_string` |
| `JWT_EXPIRATION_MS` | JWT token TTL in milliseconds | `86400000` (24 hours) |
| `CORS_ALLOWED_ORIGINS`| Allowed CORS origins for frontend | `http://localhost:5173,https://angular-frontend-for-rust-backend-n.vercel.app` |
| `RUST_LOG` | Tracing filter level | `info,tower_http=info` |

---

## 📐 Coding Conventions & Guidelines

1. **Async & Axum Patterns:**
   - Use Axum extractors (`State`, `Path`, `Json`, `AuthUser`) for route handlers.
   - Handlers should return `Result<impl IntoResponse, AppError>`.

2. **Database Queries:**
   - Use SQLx parameterized queries (`sqlx::query_as::<_, T>`, `sqlx::query`) with `.bind(...)` to prevent SQL injection and enable clean offline compilation.
   - Never write dynamic raw string concatenations for database queries.

3. **Error Handling:**
   - Use central `AppError` type with structured JSON response `{ "error": "..." }`.
   - Log detailed internal errors using `tracing::error!`.

4. **Security & Authentication:**
   - Passwords must always be hashed with `bcrypt` offloaded via `tokio::task::spawn_blocking`.
   - Protected routes must extract user identity using `AuthUser` extractor.

---

## ⚠️ CRITICAL SAFETY RULES FOR AI AGENTS

- **NEVER RUN `git commit` OR `git push` WITHOUT EXPLICIT USER APPROVAL.**
- Do NOT delete or clear existing migrations in `migrations/` without confirmation.
- Do NOT hardcode secrets or database passwords in source code. Always consume from `Config` / `.env`.
