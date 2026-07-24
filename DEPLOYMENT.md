# AWS Deployment Guide for Notes Backend

This document contains step-by-step instructions for deploying this standalone **Axum + SQLx Rust HTTP server** to Amazon Web Services (AWS).

---

## 📌 Architecture Overview

- **Framework:** Axum 0.7 + Tokio async runtime
- **Database:** PostgreSQL (AWS RDS or Neon Postgres) with automatic SQLx startup migrations
- **Type:** Standalone long-running HTTP server (non-serverless)
- **Default Port:** `8080`

---

## 🔑 Required Environment Variables

Create a `.env` file on your production server with the following values:

```env
PORT=8080
DATABASE_URL=postgresql://USER:PASSWORD@HOST:5432/neondb?sslmode=require
JWT_SECRET=your_super_secret_production_jwt_key_12345!
JWT_EXPIRATION_MS=86400000
CORS_ALLOWED_ORIGIN=https://your-frontend-domain.com
RUST_LOG=info
```

---

## 🚀 Deployment Option 1: AWS EC2 (Systemd Service) [Recommended for Fixed Cost]

Runs the compiled Rust binary directly on an Ubuntu EC2 instance as a 24/7 background system service with Nginx reverse proxy.

### Step 1: Launch EC2 Instance
1. Go to **AWS Console** → **EC2** → **Launch Instance**.
2. **Name:** `notes-backend-server`
3. **OS:** Ubuntu 24.04 LTS (Free Tier eligible: `t3.micro` or `t4g.micro`).
4. **Key Pair:** Create or select your SSH `.pem` key pair.
5. **Security Group:** Enable **SSH (22)**, **HTTP (80)**, and **HTTPS (443)**.

### Step 2: SSH into Instance
```bash
ssh -i "your-key.pem" ubuntu@<EC2_PUBLIC_IP>
```

### Step 3: Install Prerequisites
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build dependencies, OpenSSL, Git, Nginx, Certbot
sudo apt install -y build-essential pkg-config libssl-dev git nginx certbot python3-certbot-nginx

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Step 4: Clone & Build Release Binary
```bash
git clone <YOUR_GIT_REPO_URL> app
cd app
nano .env # Paste your production environment variables here

cargo build --release
```

### Step 5: Configure Systemd Service
Create service file `/etc/systemd/system/notes-backend.service`:
```bash
sudo nano /etc/systemd/system/notes-backend.service
```

Add configuration:
```ini
[Unit]
Description=Notes Axum Rust Backend Service
After=network.target

[Service]
User=ubuntu
WorkingDirectory=/home/ubuntu/app
ExecStart=/home/ubuntu/app/target/release/notes_backend
EnvironmentFile=/home/ubuntu/app/.env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable & start service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable notes-backend
sudo systemctl start notes-backend
sudo systemctl status notes-backend
```

### Step 6: Configure Nginx & SSL
Create Nginx site configuration:
```bash
sudo nano /etc/nginx/sites-available/notes-api
```

Paste configuration:
```nginx
server {
    listen 80;
    server_name api.yourdomain.com; # Or EC2 Public IP

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Link site & restart Nginx:
```bash
sudo ln -s /etc/nginx/sites-available/notes-api /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl restart nginx
```

(Optional) Attach Free SSL Certificate:
```bash
sudo certbot --nginx -d api.yourdomain.com
```

---

## 🐋 Deployment Option 2: Docker / AWS App Runner [Serverless Containers]

### Multi-Stage Dockerfile (`Dockerfile`):
```dockerfile
FROM rust:1.80-slim as builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/notes_backend /app/notes_backend
EXPOSE 8080
CMD ["/app/notes_backend"]
```

---

## 📋 Useful Server Commands

- **Check status:** `sudo systemctl status notes-backend`
- **View live logs:** `journalctl -u notes-backend -f`
- **Restart app:** `sudo systemctl restart notes-backend`
- **Redeploy new code:**
  ```bash
  cd ~/app
  git pull
  cargo build --release
  sudo systemctl restart notes-backend
  ```
