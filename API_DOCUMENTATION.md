# API Documentation - Notes Application Backend

This document details the RESTful API endpoints, request bodies, response schemas, and authentication flow for frontend integration (Angular, React, Vue, Mobile apps, etc.).

---

## 🌐 Server Base URL

- **Production (AWS EC2):** `http://13.233.31.143:8080`
- **Local Development:** `http://localhost:8080`

---

## 🔐 Authentication Scheme

Protected endpoints require a **JWT Bearer Token** in the `Authorization` HTTP header:

```http
Authorization: Bearer <access_token>
```

---

## ⚠️ Standard Error Response Format

All error responses return structured JSON with appropriate HTTP status codes:

```json
{
  "error": "Detailed error message here"
}
```

| Status Code | Meaning | Common Causes |
| :--- | :--- | :--- |
| `400 Bad Request` | Invalid payload or missing fields | Invalid email, short password, missing title |
| `401 Unauthorized` | Unauthenticated / Invalid Token | Missing, invalid, or expired JWT token |
| `404 Not Found` | Resource not found | Note ID does not exist or belongs to another user |
| `409 Conflict` | Duplicate resource | Username or email already registered |
| `500 Internal Error` | Server error | Database connection failure |

---

## 📌 Endpoint Summary

| Endpoint | Method | Auth Required | Description |
| :--- | :--- | :--- | :--- |
| [`/health`](#1-health-check) | `GET` | ❌ No | Server health status check |
| [`/auth/signup`](#2-user-signup) | `POST` | ❌ No | Create a new user account |
| [`/auth/login`](#3-user-login) | `POST` | ❌ No | Authenticate user & retrieve JWT token |
| [`/auth/me`](#4-get-current-user) | `GET` | ✅ Yes | Get profile of logged-in user |
| [`/notes`](#5-list-notes) | `GET` | ✅ Yes | Get all notes for logged-in user |
| [`/notes`](#6-create-note) | `POST` | ✅ Yes | Create a new note |
| [`/notes/:id`](#7-update-note) | `PUT` | ✅ Yes | Update an existing note by ID |
| [`/notes/:id`](#8-delete-note) | `DELETE` | ✅ Yes | Delete a note by ID |
| [`/users/theme`](#9-update-user-theme) | `PUT` | ✅ Yes | Update user UI theme (`light` / `dark`) |

---

## 📖 Endpoint Details

### 1. Health Check
Checks if backend service is running.

- **URL:** `/health`
- **Method:** `GET`
- **Auth Required:** No

#### Response (`200 OK`):
```json
{
  "service": "notes-backend",
  "status": "ok"
}
```

---

### 2. User Signup
Registers a new user account.

- **URL:** `/auth/signup`
- **Method:** `POST`
- **Auth Required:** No

#### Request Body:
```json
{
  "username": "johndoe",
  "email": "johndoe@example.com",
  "password": "secretpassword123"
}
```

#### Field Validation Rules:
- `username`: String (3 to 100 characters)
- `email`: String (valid email format containing `@`, max 255 chars)
- `password`: String (6 to 100 characters)

#### Response (`201 Created`):
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6...",
  "token_type": "Bearer",
  "user": {
    "id": "e93f87c3-3058-4521-a47f-cf12e032bc9d",
    "username": "johndoe",
    "email": "johndoe@example.com",
    "theme": "light"
  }
}
```

---

### 3. User Login
Authenticates user credentials and returns a JWT access token.

- **URL:** `/auth/login`
- **Method:** `POST`
- **Auth Required:** No

#### Request Body:
```json
{
  "email": "johndoe@example.com",
  "password": "secretpassword123"
}
```

#### Response (`200 OK`):
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6...",
  "token_type": "Bearer",
  "user": {
    "id": "e93f87c3-3058-4521-a47f-cf12e032bc9d",
    "username": "johndoe",
    "email": "johndoe@example.com",
    "theme": "light"
  }
}
```

---

### 4. Get Current User
Retrieves profile information for the authenticated user.

- **URL:** `/auth/me`
- **Method:** `GET`
- **Auth Required:** Yes (`Bearer <token>`)

#### Response (`200 OK`):
```json
{
  "id": "e93f87c3-3058-4521-a47f-cf12e032bc9d",
  "username": "johndoe",
  "email": "johndoe@example.com",
  "theme": "light"
}
```

---

### 5. List Notes
Retrieves all notes created by the authenticated user, sorted by pinned status (`pinned DESC`) and update date (`updated_at DESC`).

- **URL:** `/notes`
- **Method:** `GET`
- **Auth Required:** Yes (`Bearer <token>`)

#### Response (`200 OK`):
```json
[
  {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "user_id": "e93f87c3-3058-4521-a47f-cf12e032bc9d",
    "title": "Meeting Notes",
    "content": "Discuss Q3 backend architecture and release pipeline.",
    "tags": ["work", "meeting", "rust"],
    "color": "#ffeb3b",
    "pinned": true,
    "created_at": "2026-07-24T14:00:00Z",
    "updated_at": "2026-07-24T14:30:00Z"
  }
]
```

---

### 6. Create Note
Creates a new note for the authenticated user.

- **URL:** `/notes`
- **Method:** `POST`
- **Auth Required:** Yes (`Bearer <token>`)

#### Request Body:
```json
{
  "title": "Shopping List",
  "content": "Buy coffee beans, milk, and eggs.",
  "tags": ["personal", "groceries"],
  "color": "#81c784",
  "pinned": false
}
```

*(Note: All fields in request body are optional and default to empty string / empty array / false).*

#### Response (`201 Created`):
```json
{
  "id": "b2c3d4e5-f6a7-8901-bcde-f23456789012",
  "user_id": "e93f87c3-3058-4521-a47f-cf12e032bc9d",
  "title": "Shopping List",
  "content": "Buy coffee beans, milk, and eggs.",
  "tags": ["personal", "groceries"],
  "color": "#81c784",
  "pinned": false,
  "created_at": "2026-07-24T15:00:00Z",
  "updated_at": "2026-07-24T15:00:00Z"
}
```

---

### 7. Update Note
Updates an existing note by ID.

- **URL:** `/notes/:id`
- **Method:** `PUT`
- **Auth Required:** Yes (`Bearer <token>`)

#### Path Parameter:
- `id`: Note UUID string (e.g. `b2c3d4e5-f6a7-8901-bcde-f23456789012`)

#### Request Body:
```json
{
  "title": "Updated Shopping List",
  "content": "Buy coffee, milk, eggs, and organic bread.",
  "tags": ["personal", "groceries", "urgent"],
  "color": "#4fc3f7",
  "pinned": true
}
```

#### Response (`200 OK`):
```json
{
  "id": "b2c3d4e5-f6a7-8901-bcde-f23456789012",
  "user_id": "e93f87c3-3058-4521-a47f-cf12e032bc9d",
  "title": "Updated Shopping List",
  "content": "Buy coffee, milk, eggs, and organic bread.",
  "tags": ["personal", "groceries", "urgent"],
  "color": "#4fc3f7",
  "pinned": true,
  "created_at": "2026-07-24T15:00:00Z",
  "updated_at": "2026-07-24T15:05:00Z"
}
```

---

### 8. Delete Note
Deletes a note by ID.

- **URL:** `/notes/:id`
- **Method:** `DELETE`
- **Auth Required:** Yes (`Bearer <token>`)

#### Path Parameter:
- `id`: Note UUID string

#### Response (`200 OK`):
```json
{
  "message": "Note deleted successfully"
}
```

---

### 9. Update User Theme
Updates the authenticated user's UI theme preference.

- **URL:** `/users/theme`
- **Method:** `PUT`
- **Auth Required:** Yes (`Bearer <token>`)

#### Request Body:
```json
{
  "theme": "dark"
}
```
*(Valid theme values: `"light"` or `"dark"`).*

#### Response (`200 OK`):
```json
{
  "message": "Theme updated successfully"
}
```

---

## 💻 Frontend Code Snippet Examples

### JavaScript / TypeScript Fetch Example:
```typescript
const BASE_URL = 'http://13.233.31.143:8080';
let token = localStorage.getItem('access_token');

// Create Note
async function createNote(title: string, content: string) {
  const response = await fetch(`${BASE_URL}/notes`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({ title, content, tags: [], color: '#ffffff', pinned: false })
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.error || 'Failed to create note');
  }

  return await response.json();
}
```
