# AWS Deployment Guide (Neon PostgreSQL + AWS EC2 Free Tier)

This guide details how to deploy your Notes App Rust backend to **AWS EC2 (Free Tier)** using your live **Neon PostgreSQL** database.

---

## 🚀 Why Neon PostgreSQL + AWS EC2 is the Ideal Setup

- **Neon PostgreSQL**: Free forever, managed, serverless PostgreSQL with SSL enabled out of the box.
- **AWS EC2 (`t3.micro` / `t2.micro`)**: 100% Free for 12 months (750 hours/month 24/7 server).
- **Automated Schema Migrations**: The backend automatically runs all database migrations on boot using `sqlx::migrate!`, creating tables and indexes on Neon automatically!

---

## 🛠️ Step 1: Prepare Environment Variables

Use your existing `DATABASE_URL` from `.env` (e.g., `postgresql://user:pass@ep-xxxx.neon.tech/neondb?sslmode=require`).

Required Environment Variables for AWS EC2:
```env
PORT=8080
SERVER_HOST=0.0.0.0
DATABASE_URL=postgresql://user:pass@ep-xxxx.neon.tech/neondb?sslmode=require
JWT_SECRET=your_custom_super_secret_jwt_key_min_32_chars
JWT_EXPIRATION_MS=86400000
CORS_ALLOWED_ORIGINS=*
MAX_CONNECTIONS=20
BCRYPT_COST=10
RUST_LOG=info,tower_http=info
```

---

## 🖥️ Step 2: Provision AWS EC2 Instance (Free Tier)

1. Open [AWS EC2 Console](https://console.aws.amazon.com/ec2).
2. Click **Launch Instance**.
3. **Name**: `notes-backend-server`
4. **AMI**: Ubuntu Server 22.04 LTS (Free Tier eligible).
5. **Instance type**: `t3.micro` or `t2.micro`.
6. **Key pair**: Select or create an SSH key pair (`.pem` file).
7. **Network Settings (Security Group)**:
   - Check **Allow SSH** (Port 22).
   - Check **Allow HTTP** (Port 80).
   - Add Custom TCP Rule: Port `8080` (Source: `0.0.0.0/0`).
8. Click **Launch Instance**.

---

## ⚡ Step 3: Single Command Automated Deployment on EC2

1. SSH into your EC2 server:
   ```bash
   ssh -i /path/to/your-key.pem ubuntu@<YOUR_EC2_PUBLIC_IP>
   ```

2. Run this single automated setup command on EC2 to install Docker, clone your repository, build, and connect directly to your **Neon PostgreSQL** database:

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
     -e DATABASE_URL="YOUR_NEON_DATABASE_URL_HERE" \
     -e JWT_SECRET="your_custom_super_secret_jwt_key_min_32_chars" \
     -e JWT_EXPIRATION_MS="86400000" \
     -e CORS_ALLOWED_ORIGINS="*" \
     -e MAX_CONNECTIONS="20" \
     -e BCRYPT_COST="10" \
     -e RUST_LOG="info,tower_http=info" \
     notes-backend
   ```

   *(Replace `YOUR_NEON_DATABASE_URL_HERE` with your actual Neon connection string from your local `.env` file).*

---

## 🧪 Step 4: Verification & Health Check

Test backend health from your terminal or browser:

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
