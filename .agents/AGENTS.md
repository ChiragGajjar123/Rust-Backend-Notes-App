# AGENTS.md - Workspace Customization Rules

## Critical Constraints for AI Assistants
- **Git Push / Commit Rule:** NEVER run `git commit` or `git push` without explicit user permission.
- **Backend Architecture:** Standalone Axum 0.7 + Tokio + SQLx PostgreSQL HTTP web application.
- **Deployment Target:** AWS EC2 (Systemd Service) or AWS App Runner / Docker container.
- **Environment Configuration:** All runtime configs must be parsed from environment variables via `Config::from_env()`.
