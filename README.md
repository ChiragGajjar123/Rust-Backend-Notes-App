# Notes Rust Backend for Vercel

Rust serverless implementation of the Notes API, backed by Neon/Postgres with JWT and bcrypt auth.

The HTTP contract matches the existing frontend API:

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/auth/signup` | No | Register user |
| `POST` | `/api/auth/login` | No | Login and return JWT |
| `PUT` | `/api/users/theme` | Bearer | Update light/dark theme |
| `GET` | `/api/notes` | Bearer | List notes, pinned first |
| `POST` | `/api/notes` | Bearer | Create note |
| `PUT` | `/api/notes/{id}` | Bearer | Update note |
| `DELETE` | `/api/notes/{id}` | Bearer | Delete note |

## Vercel Structure

- `api/index.rs` is the Vercel Rust function entrypoint.
- `src/lib.rs` contains the shared API/router/database/auth code.
- `vercel.json` rewrites `/api/*` requests into the single Rust function.
- `Cargo.toml` is the only Cargo manifest.

## Environment Variables

Set these in the Vercel dashboard:

```env
DATABASE_URL=postgresql://USER:PASSWORD@HOST/neondb?sslmode=require
JWT_SECRET=change-me-to-a-long-random-secret
JWT_EXPIRATION_MS=86400000
CORS_ALLOWED_ORIGIN=https://your-frontend-domain.example
```

`DATABASE_URL` may also be provided as `NEON_DATABASE_URL`. Tables and indexes are created automatically on cold start if they do not exist.

## Deploy

```powershell
vercel deploy --prod
```

For local Vercel development:

```powershell
vercel dev
```

Point the frontend at:

```env
VITE_API_URL=https://your-vercel-app.vercel.app/api
```

## Database

The schema is kept in `schema.sql` for reference and manual setup. The function also runs the same schema creation during initialization.
