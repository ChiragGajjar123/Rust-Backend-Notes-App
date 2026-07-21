# Notes Rust Backend (Neon Postgres + JWT Auth)

`may_minihttp` implementation of the existing Notes API, backed by **Neon Postgres** with **JWT + bcrypt auth**.

The HTTP contract matches the Spring backend so the React frontend works without changes.

## API (unchanged)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/auth/signup` | No | Register user |
| `POST` | `/api/auth/login` | No | Login → JWT |
| `PUT` | `/api/users/theme` | Bearer | Update light/dark theme |
| `GET` | `/api/notes` | Bearer | List notes (pinned first) |
| `POST` | `/api/notes` | Bearer | Create note |
| `PUT` | `/api/notes/{id}` | Bearer | Update note |
| `DELETE` | `/api/notes/{id}` | Bearer | Delete note |

## Setup Neon

1. Create a project at [https://console.neon.tech](https://console.neon.tech)
2. Copy the connection string (URI) from **Connection Details**
3. Configure env:

```powershell
cd "Rust backend"
copy .env.example .env
# Edit .env and set DATABASE_URL to your Neon URI
```

Example:

```env
DATABASE_URL=postgresql://USER:PASSWORD@ep-xxxx.us-east-2.aws.neon.tech/neondb?sslmode=require
JWT_SECRET=change-me-to-a-long-random-secret
CORS_ALLOWED_ORIGIN=http://localhost:5173
PORT=8080
```

Tables are created automatically on startup (`users`, `notes`). See `schema.sql` for the full DDL.

## Run

```powershell
cd "Rust backend"
cargo run --release
```

Point the frontend at the same base URL as before:

```env
# frontend/.env
VITE_API_URL=http://localhost:8080/api
```

## Auth

- Passwords hashed with **bcrypt** (same idea as Spring `BCryptPasswordEncoder`)
- Tokens are **JWT HMAC-SHA256** with claim `sub` = username
- Protected routes require `Authorization: Bearer <token>`
- Login response shape (frontend-compatible):

```json
{
  "accessToken": "...",
  "tokenType": "Bearer",
  "id": "uuid",
  "username": "...",
  "email": "...",
  "theme": "light"
}
```

## Local Postgres (optional)

```env
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/notes_app_db?sslmode=disable
```

## Notes

- IDs are UUIDs (JSON strings), same string-id usage as the Mongo/Spring app
- `MAY_WORKERS` controls the may scheduler worker count
- Prefer `cargo run --release` for real load testing
