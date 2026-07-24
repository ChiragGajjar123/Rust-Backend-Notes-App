# AWS EC2 Native Deployment Guide (Pure Rust + Systemd + Local PostgreSQL)

This guide details how to deploy your Notes App Rust backend natively on an **AWS EC2 (Free Tier)** instance running as a lightweight **Systemd background service**, connected to a local **PostgreSQL** database on EC2.

---

## ⚡ Why Native Systemd + Local PostgreSQL Deployment?

- **90% Less RAM Usage**: Uses only **~15 MB RAM** (compared to 300+ MB with Docker).
- **Zero Overhead**: Direct native binary execution on Linux (`ubuntu`).
- **Instant Speed**: 0 ms database latency (PostgreSQL runs locally on the same EC2 instance).
- **100% Free**: No extra costs or external serverless limits.

---

## 🖥️ Setup Native PostgreSQL on EC2

Run this single command on your EC2 terminal:

```bash
sudo apt update && sudo apt install -y postgresql postgresql-contrib && \
sudo -u postgres psql -c "CREATE USER notesuser WITH PASSWORD 'NotesAppSecurePass123!';" && \
sudo -u postgres psql -c "CREATE DATABASE notesdb OWNER notesuser;" && \
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE notesdb TO notesuser;" && \
sudo -u postgres psql -d notesdb -c "GRANT ALL ON SCHEMA public TO notesuser;"
```

---

## 🚀 Step-by-Step Service Configuration

```bash
sudo bash -c 'cat <<EOF > /etc/systemd/system/notes-backend.service
[Unit]
Description=Notes App Rust Backend Service
After=network.target postgresql.service

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/app
ExecStart=/usr/local/bin/notes_backend
Restart=always
RestartSec=5
Environment="PORT=8080"
Environment="SERVER_HOST=0.0.0.0"
Environment="DATABASE_URL=postgresql://notesuser:NotesAppSecurePass123!@localhost:5432/notesdb"
Environment="JWT_SECRET=secretKeyNotesAppJWTSecretTokenGeneratorCustomString1234567890!"
Environment="JWT_EXPIRATION_MS=86400000"
Environment="CORS_ALLOWED_ORIGINS=http://localhost:5173,https://angular-frontend-for-rust-backend-n.vercel.app"
Environment="MAX_CONNECTIONS=20"
Environment="BCRYPT_COST=10"
Environment="RUST_LOG=info,tower_http=info"

[Install]
WantedBy=multi-user.target
EOF' && \
sudo systemctl daemon-reload && \
sudo systemctl enable --now notes-backend && \
sudo systemctl status notes-backend
```
