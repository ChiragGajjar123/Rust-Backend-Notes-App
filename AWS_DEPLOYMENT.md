# AWS Deployment Guide - Pure Rust Notes Backend

This document provides complete instructions for deploying the Notes Application Rust backend on **Amazon Web Services (AWS)** using standard, non-serverless long-running HTTP Web Services for maximum concurrency and performance.

---

## 🏗️ Architecture Overview

- **Language & Runtime:** Rust 2021 (Axum 0.7 + Tokio 1.38 multi-threaded async runtime)
- **Database:** AWS RDS PostgreSQL (or Neon/Supabase PostgreSQL)
- **Container Registry:** AWS ECR (Elastic Container Registry)
- **Deployment Options:**
  - **AWS App Runner** (Recommended: Fully managed container Web Service with automatic scaling and health checks).
  - **AWS EC2** (Docker container or systemd binary service on an EC2 instance).
  - **AWS ECS Fargate** (Containerized execution).

---

## 🚀 Option 1: Deploy to AWS App Runner (Recommended)

AWS App Runner provides fully managed container deployments directly from GitHub or AWS ECR with built-in auto-scaling, load balancing, and zero cold-start delay.

### Step 1: Push Container to AWS ECR
1. Log into AWS Console and open **Amazon ECR**.
2. Create a repository named `notes-backend`.
3. Authenticate Docker with your ECR registry:
   ```bash
   aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com
   ```
4. Build and tag the Docker image:
   ```bash
   docker build -t notes-backend .
   docker tag notes-backend:latest <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/notes-backend:latest
   ```
5. Push image to ECR:
   ```bash
   docker push <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/notes-backend:latest
   ```

### Step 2: Create App Runner Service
1. Navigate to **AWS App Runner** -> **Create Service**.
2. Source: **Container registry** -> **Amazon ECR**.
3. Select `notes-backend:latest`.
4. Under **Deployment settings**, choose **Automatic** (deploys new images automatically when pushed).
5. Configure Service:
   - **Service Name:** `notes-backend-service`
   - **Port:** `8080`
   - **Health Check Path:** `/health`
6. Add Environment Variables:

| Variable | Description / Example |
| :--- | :--- |
| `PORT` | `8080` |
| `SERVER_HOST` | `0.0.0.0` |
| `DATABASE_URL` | `postgresql://user:password@rds-endpoint.amazonaws.com:5432/notesapp?sslmode=require` |
| `JWT_SECRET` | `min_32_character_random_secret_key_string` |
| `JWT_EXPIRATION_MS` | `86400000` |
| `CORS_ALLOWED_ORIGINS` | `*` |
| `MAX_CONNECTIONS` | `50` |
| `BCRYPT_COST` | `10` |
| `RUST_LOG` | `info,tower_http=info` |

7. Click **Create & Deploy**.

---

## 🖥️ Option 2: Deploy to AWS EC2 (Docker or Systemd)

For full control over server resources, deploy on an Ubuntu/Debian EC2 instance.

### Step 1: Launch EC2 Instance
1. Go to AWS EC2 Console -> **Launch Instance**.
2. **AMI:** Ubuntu 22.04 LTS or Debian 12.
3. **Instance Type:** `t3.small` or `t3.medium` (for high CPU concurrency during bcrypt operations).
4. **Security Group:**
   - Allow Inbound HTTP (`80`), HTTPS (`443`), and Custom TCP (`8080`).

### Step 2: Running with Docker on EC2
1. SSH into your EC2 instance:
   ```bash
   ssh -i your-key.pem ubuntu@<EC2_PUBLIC_IP>
   ```
2. Install Docker:
   ```bash
   sudo apt update && sudo apt install -y docker.io
   sudo systemctl enable --now docker
   ```
3. Clone repository and run with Docker:
   ```bash
   git clone https://github.com/ChiragGajjar123/Rust-Backend-Notes-App.git app
   cd app
   sudo docker build -t notes-backend .
   sudo docker run -d --name notes-backend \
     -p 8080:8080 \
     -e DATABASE_URL="postgresql://user:pass@rds-endpoint:5432/dbname" \
     -e JWT_SECRET="your_secure_jwt_secret_key" \
     -e PORT="8080" \
     --restart always \
     notes-backend
   ```

### Step 3: Running as a Native Systemd Service on EC2
1. Build native release binary:
   ```bash
   cargo build --release
   sudo cp target/release/notes_backend /usr/local/bin/notes_backend
   ```
2. Create Systemd service definition `/etc/systemd/system/notes-backend.service`:
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
   Environment="DATABASE_URL=postgresql://user:pass@rds-endpoint:5432/dbname"
   Environment="JWT_SECRET=your_secure_jwt_secret_key"
   Environment="JWT_EXPIRATION_MS=86400000"
   Environment="CORS_ALLOWED_ORIGINS=*"
   Environment="MAX_CONNECTIONS=50"
   Environment="BCRYPT_COST=10"
   Environment="RUST_LOG=info,tower_http=info"

   [Install]
   WantedBy=multi-user.target
   ```
3. Enable and start service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now notes-backend
   sudo systemctl status notes-backend
   ```

---

## 🗄️ AWS RDS PostgreSQL Setup

1. In AWS Console, go to **RDS** -> **Create Database**.
2. Engine: **PostgreSQL**.
3. Template: Free tier or Production (`db.t4g.micro` / `db.t4g.small`).
4. Settings:
   - **DB instance identifier:** `notes-db`
   - **Master username:** `notesuser`
   - **Master password:** `secure_password`
5. Public access: Choose **Yes** (if connecting outside VPC) or **No** (if running inside VPC with EC2/App Runner).
6. Set Security Group to allow inbound PostgreSQL traffic (`5432`) from your EC2 or App Runner security group.

---

## ⚡ Concurrency & Performance Tuning

- **Async CPU Offloading:** CPU-heavy bcrypt operations are offloaded to `tokio::task::spawn_blocking` to ensure high throughput for async Tokio worker threads under heavy concurrent load.
- **SQLx Pool Management:** Connections are automatically recycled and managed (`min_connections: 5`, `max_connections: 50`, idle timeout 300s).
- **Automated Migrations:** Database migrations run automatically at application boot.

---

## 🧪 Health Verification

Check backend health after deployment:
```bash
curl http://<YOUR_AWS_ENDPOINT>:8080/health
```

Expected Output:
```json
{
  "service": "notes-backend",
  "status": "ok"
}
```
