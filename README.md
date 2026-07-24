# Notes Rust Backend

High-performance Rust web API built with Axum, Tokio, and SQLx, backed by PostgreSQL (or Neon) with JWT authentication and bcrypt password hashing.

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/auth/signup` | No | Register user |
| `POST` | `/api/auth/login` | No | Login and return JWT |
| `PUT` | `/api/users/theme` | Bearer | Update light/dark theme |
| `GET` | `/api/notes` | Bearer | List notes, pinned first |
| `POST` | `/api/notes` | Bearer | Create note |
| `PUT` | `/api/notes/{id}` | Bearer | Update note |
| `DELETE` | `/api/notes/{id}` | Bearer | Delete note |

## Environment Variables

Configure these in `.env`:

```env
PORT=8080
DATABASE_URL=postgresql://USER:PASSWORD@HOST/neondb?sslmode=require
JWT_SECRET=change-me-to-a-long-random-secret
JWT_EXPIRATION_MS=86400000
CORS_ALLOWED_ORIGIN=http://localhost:5173
```

## Running Locally

```powershell
cargo run
```

## Database Migrations

Migrations are stored in `migrations/` and executed automatically when the backend starts up.

