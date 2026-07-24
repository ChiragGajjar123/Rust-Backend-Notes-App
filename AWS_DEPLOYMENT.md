# AWS Free Tier Deployment Guide - Pure Rust Notes Backend

This step-by-step guide explains how to deploy your Notes App Rust backend on **AWS 100% Free Tier** (Zero Monthly Cost) using an **AWS EC2 `t3.micro` / `t2.micro`** instance and **AWS RDS PostgreSQL** (or Free Cloud PostgreSQL).

---

## 🎁 AWS Free Tier Allocation

| Resource | AWS Free Tier Allowance | Purpose |
| :--- | :--- | :--- |
| **AWS EC2** | 750 hours/month of `t2.micro` / `t3.micro` (24/7 runtime) | Hosts the Rust HTTP Web Server |
| **EBS Storage** | 30 GB SSD storage | OS & Application Disk Space |
| **AWS RDS PostgreSQL** | 750 hours/month of `db.t4g.micro` / `db.t3.micro` | Managed PostgreSQL Database |

---

## 🛠️ Step 1: Provision Free Database (AWS RDS or Neon Postgres)

### Option A: AWS RDS PostgreSQL (Free Tier)
1. Log into [AWS Management Console](https://console.aws.amazon.com/rds).
2. Go to **RDS** -> **Create Database**.
3. **Database creation method:** Standard create
4. **Engine options:** PostgreSQL
5. **Templates:** **Free Tier**
6. **Settings:**
   - **DB instance identifier:** `notes-db`
   - **Master username:** `notesuser`
   - **Master password:** `CreateASecurePassword123!`
7. **Instance configuration:** `db.t4g.micro` or `db.t3.micro`
8. **Storage:** 20 GiB General Purpose SSD (gp3)
9. **Connectivity:**
   - **Public access:** `Yes` (or restrict to your EC2 security group)
10. Click **Create database**. Copy your **Endpoint** URI once created (e.g. `notes-db.xxxx.us-east-1.rds.amazonaws.com`).

---

## 🖥️ Step 2: Provision Free AWS EC2 Server

1. Go to [AWS EC2 Console](https://console.aws.amazon.com/ec2).
2. Click **Launch Instance**.
3. **Name:** `notes-backend-server`
4. **AMI:** Ubuntu Server 22.04 LTS (Free Tier eligible)
5. **Instance type:** `t3.micro` or `t2.micro` (Free Tier eligible)
6. **Key pair:** Create or select an existing SSH key pair (download `.pem` file).
7. **Network Settings (Security Group):**
   - Check **Allow SSH traffic** (Port 22)
   - Check **Allow HTTP traffic from the internet** (Port 80)
   - Add Custom TCP Rule: Port `8080` (Source: `0.0.0.0/0`)
8. Click **Launch Instance**.

---

## 🚀 Step 3: 1-Command Automated Backend Setup on EC2

1. SSH into your EC2 instance from your terminal:
   ```bash
   ssh -i /path/to/your-key.pem ubuntu@<YOUR_EC2_PUBLIC_IP>
   ```

2. Run this single automated setup command on EC2 to install Docker, clone your repository, build, and launch the service:

   ```bash
   sudo apt update && sudo apt install -y docker.io git && \
   sudo systemctl enable --now docker && \
   git clone https://github.com/ChiragGajjar123/Rust-Backend-Notes-App.git app && \
   cd app && \
   sudo docker build -t notes-backend . && \
   sudo docker run -d \
     --name notes-backend \
     --restart always \
     -p 8080:8080 \
     -e PORT="8080" \
     -e SERVER_HOST="0.0.0.0" \
     -e DATABASE_URL="postgresql://notesuser:CreateASecurePassword123!@notes-db.xxxx.us-east-1.rds.amazonaws.com:5432/notesapp?sslmode=require" \
     -e JWT_SECRET="your_custom_super_secret_jwt_key_min_32_chars" \
     -e JWT_EXPIRATION_MS="86400000" \
     -e CORS_ALLOWED_ORIGINS="*" \
     -e MAX_CONNECTIONS="20" \
     -e BCRYPT_COST="10" \
     -e RUST_LOG="info,tower_http=info" \
     notes-backend
   ```

---

## ⚡ Step 4: Optional (Native Systemd Service without Docker)

If you prefer running without Docker to minimize RAM usage on `t3.micro`:

1. Install Rust on EC2:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   source $HOME/.cargo/env
   ```
2. Clone and build release binary:
   ```bash
   git clone https://github.com/ChiragGajjar123/Rust-Backend-Notes-App.git app
   cd app
   cargo build --release
   sudo cp target/release/notes_backend /usr/local/bin/notes_backend
   ```
3. Create Systemd Service `/etc/systemd/system/notes-backend.service`:
   ```ini
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
   Environment="DATABASE_URL=postgresql://notesuser:CreateASecurePassword123!@notes-db.xxxx.us-east-1.rds.amazonaws.com:5432/notesapp?sslmode=require"
   Environment="JWT_SECRET=your_custom_super_secret_jwt_key_min_32_chars"
   Environment="JWT_EXPIRATION_MS=86400000"
   Environment="CORS_ALLOWED_ORIGINS=*"
   Environment="MAX_CONNECTIONS=20"
   Environment="BCRYPT_COST=10"
   Environment="RUST_LOG=info,tower_http=info"

   [Install]
   WantedBy=multi-user.target
   ```
4. Enable & start service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now notes-backend
   sudo systemctl status notes-backend
   ```

---

## 🧪 Verification & Health Check

Test backend health from your browser or terminal:

```bash
curl http://<YOUR_EC2_PUBLIC_IP>:8080/health
```

Expected Response:
```json
{
  "service": "notes-backend",
  "status": "ok"
}
```
