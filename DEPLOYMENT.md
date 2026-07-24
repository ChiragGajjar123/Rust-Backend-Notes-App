# Deployment Guide - Pure Rust Notes App Backend

This repository contains a high-performance RESTful API written in **pure Rust** (Axum 0.7 + Tokio 1.38 + SQLx 0.8), designed for **native deployment on AWS EC2** as a lightweight **Systemd background service**.

---

## 📌 Quick Reference

| File | Description |
| :--- | :--- |
| [AWS_DEPLOYMENT.md](file:///d:/Rust/Notes%20App/Rust%20backend/AWS_DEPLOYMENT.md) | Step-by-step Native AWS EC2 Systemd deployment guide |
| [.env.example](file:///d:/Rust/Notes%20App/Rust%20backend/.env.example) | Environment variables template |

---

## ⚡ Native EC2 Systemd Deployment (No Docker)

```bash
# Build & deploy as systemd service on Ubuntu EC2:
cargo build --release
sudo cp target/release/notes_backend /usr/local/bin/notes_backend
sudo systemctl enable --now notes-backend
```

See [AWS_DEPLOYMENT.md](file:///d:/Rust/Notes%20App/Rust%20backend/AWS_DEPLOYMENT.md) for full instructions and automated script.
