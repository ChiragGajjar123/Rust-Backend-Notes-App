# Deployment Overview - Notes App Rust Backend

This directory contains the source code and configuration for deploying the Notes Application pure Rust backend to **Amazon Web Services (AWS)**.

---

## 📌 Quick Reference

| Resource | File Link | Description |
| :--- | :--- | :--- |
| **AWS Deployment Guide** | [AWS_DEPLOYMENT.md](file:///d:/Rust/Notes%20App/Rust%20backend/AWS_DEPLOYMENT.md) | Full guide for AWS App Runner, AWS EC2, ECR, and RDS |
| **App Runner Manifest** | [apprunner.yaml](file:///d:/Rust/Notes%20App/Rust%20backend/apprunner.yaml) | Configuration manifest for 1-click AWS App Runner deployments |
| **Multi-Stage Dockerfile** | [Dockerfile](file:///d:/Rust/Notes%20App/Rust%20backend/Dockerfile) | Production Docker image build file |
| **Environment Template** | [.env.example](file:///d:/Rust/Notes%20App/Rust%20backend/.env.example) | Environment variables template |

---

## 🚀 Quick Deployment Summary

### 1. AWS App Runner (Container Service)
1. Push Docker image to AWS ECR using `Dockerfile`.
2. Provision App Runner service pointing to your ECR image using `apprunner.yaml`.
3. Set environment variables (`DATABASE_URL`, `JWT_SECRET`, `PORT=8080`).

### 2. AWS EC2 (Docker or Systemd Service)
1. Launch Ubuntu/Debian EC2 instance (`t3.small` or `t3.medium`).
2. Run via Docker (`docker run -d -p 8080:8080 ...`) OR compile release binary (`cargo build --release`) and manage with `systemd`.

---

## 🔍 Verification Endpoint

- `GET /health` -> `{"status": "ok", "service": "notes-backend"}`
