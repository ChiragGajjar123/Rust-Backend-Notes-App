# AWS EC2 Native Deployment Guide (Pure Rust + Systemd + Neon Postgres)

This guide details how to deploy your Notes App Rust backend natively on an **AWS EC2 (Free Tier)** instance running as a lightweight **Systemd background service**, connected to **Neon PostgreSQL**.

---

## ⚡ Why Native Systemd Deployment (No Docker)?

- **90% Less RAM Usage**: Uses only **~15 MB RAM** (compared to 300+ MB with Docker).
- **Zero Overhead**: Direct native binary execution on Linux (`ubuntu`).
- **Instant Auto-Restart**: Systemd automatically manages the process and restarts it if the EC2 server reboots.
- **Fast Build Times**: Uses lightweight `cargo build --release` directly on EC2.

---

## 🎁 AWS Free Tier & Database Allocation

- **AWS EC2 (`t3.micro` / `t2.micro`)**: 750 hours/month (100% FREE for 12 months).
- **Neon PostgreSQL**: Free forever serverless database (`sslmode=require`).

---

## 🖥️ Step 1: Provision AWS EC2 Instance (Free Tier)

1. Open [AWS EC2 Console](https://console.aws.amazon.com/ec2).
2. Click **Launch Instance**.
3. **Name**: `notes-backend-server`
4. **AMI**: Ubuntu Server 22.04 LTS (Free Tier eligible).
5. **Instance type**: `t3.micro` or `t2.micro`.
6. **Key pair**: Create or select an SSH key pair (`.pem` file).
7. **Network Settings (Security Group)**:
   - Check **Allow SSH** (Port 22 from Anywhere).
   - Check **Allow HTTP** (Port 80).
   - Check **Allow HTTPS** (Port 443).
8. Click **Launch Instance**.

---

## 🚀 Step 2: Single Command Automated Native Deployment (No Docker!)

1. Connect to your EC2 instance via **EC2 Instance Connect** in your browser (or SSH).
2. Copy and paste this single command into the EC2 terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
source $HOME/.cargo/env && \
cd ~ && rm -rf app && git clone https://github.com/ChiragGajjar123/Rust-Backend-Notes-App.git app && \
cd app && \
cargo build --release && \
sudo cp target/release/notes_backend /usr/local/bin/notes_backend && \
sudo bash -c 'cat <<EOF > /etc/systemd/system/notes-backend.service
[Unit]
Description=Notes App Rust Backend Service
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu
ExecStart=/usr/local/bin/notes_backend
Restart=always
RestartSec=5
Environment="PORT=8080"
Environment="SERVER_HOST=0.0.0.0"
Environment="DATABASE_URL=postgresql://neondb_owner:npg_Dle7p9JNFGOW@ep-autumn-smoke-azqqtdsm-pooler.c-3.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require"
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

---

## 🛠️ Management Commands on EC2

- **Check status**: `sudo systemctl status notes-backend`
- **View live logs**: `sudo journalctl -u notes-backend -f`
- **Restart service**: `sudo systemctl restart notes-backend`
- **Stop service**: `sudo systemctl stop notes-backend`

---

## 🧪 Verification

Test backend health from terminal or browser:

```bash
curl http://<YOUR_EC2_PUBLIC_IP>:8080/health
```

Expected Output:
```json
{
  "service": "notes-backend",
  "status": "ok"
}
```
