# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Start all services (one-click)
docker compose up

# Run all tests (unit + API integration)
./run_tests.sh

# Unit tests only (no services needed) — 127 tests
python3 -m unittest discover -s unit_tests -p "test_*.py" -v

# Single unit test file
python3 -m unittest unit_tests.backend.test_password -v

# API integration tests (requires running backend on :8000)
API_BASE_URL=http://localhost:8000 python3 -m unittest discover -s API_tests -p "test_*.py" -v

# Backend build (inside backend/)
cargo build --release

# Frontend WASM build (inside frontend/, requires trunk and wasm32-unknown-unknown target)
trunk build --release
```

## Architecture

Three-tier monorepo: **Dioxus/WASM frontend** (nginx :3000) → **Rocket API** (:8000) → **MySQL 8.0** (:3306).

### Backend (Rust + Rocket)

Layered architecture with strict dependency flow:

**routes/** (HTTP handlers, validation) → **services/** (business logic) → **repositories/** (SQL queries) → **models/** (entities)

Supporting modules:
- `middleware/auth_guard.rs` — JWT extraction from `Authorization: Bearer <token>` header; guards: `AdminGuard`, `StaffAuthorGuard`, `DeptReviewerGuard`, `CourseAuthorGuard`, `ReviewerGuard`. Admin role bypasses all role checks.
- `middleware/rate_limiter.rs` — 120 req/min per user via DB-backed sliding window
- `middleware/reauth_guard.rs` — `ReauthRequired` guard enforces re-auth within 15 min for admin actions
- `middleware/hmac_guard.rs` — HMAC-SHA256 signature verification with nonce anti-replay (5 min expiry)
- `middleware/correlation.rs` — `X-Correlation-Id` fairing, auto-generates UUID if not present
- `auth/` — JWT (HS256), bcrypt passwords, HMAC signing utilities
- `dto/` — request/response schemas per domain (auth, course, approval)
- `services/` — business logic: auth_service, course_service, approval_service, version_service, audit_service
- `jobs/` — background tasks: scheduled transitions processor, expired data cleanup

JWT Claims: `sub` (uuid), `user_id` (i64), `username`, `role`, `department_id`, `exp`, `iat`

All routes mounted under `/api/v1` except `/health`.

### Frontend (Dioxus 0.6)

- `auth/mod.rs` — `AUTH: GlobalSignal<AuthState>` is the single source of auth truth
- `api/mod.rs` — HTTP client auto-attaches Bearer token from LocalStorage (`campus_learn_token`)
- `layouts/mod.rs` — `MainLayout` renders role-aware sidebar navigation
- nginx config in `frontend/Dockerfile` proxies `/api/` and `/health` to `backend:8000`

### Database

- SQLx migrations in `backend/migrations/` auto-run at startup via `sqlx::migrate!()`
- Migration naming: `YYYYMMDDHHMMSS_description.sql`
- Schema: 30+ tables across domains (core, auth, academic, versioning, approvals, bookings, compliance, audit, notifications, privacy)
- Seeding (`services/seed.rs`): creates departments, permissions, and 5 default accounts on first boot

## Conventions

### API Response Format

Success: `{ "success": true, "data": T, "message": null }`
Error: `{ "status": 400, "error": "Bad Request", "message": "...", "details": null }`

### Roles

`admin` > `staff_author` > `dept_reviewer` > `faculty` > `student` (+ `integration` for API keys) — MySQL ENUM.

### Course Status Flow

`draft` → `pending_approval` → (step1: dept_reviewer, step2: admin) → `approved_scheduled` or `published` → `unpublished`
Rejection returns to `rejected` (re-editable). Author cannot self-approve.

### Default Seeded Accounts

| Username | Password | Role |
|----------|----------|------|
| admin | Admin@12345678 | admin |
| author | Author@1234567 | staff_author |
| reviewer | Review@1234567 | dept_reviewer |
| faculty | Faculty@123456 | faculty |
| student | Student@12345 | student |

## Non-Obvious Details

- Backend Docker build uses `SQLX_OFFLINE=true` — runtime queries use string-based `sqlx::query_as` (not `query_as!` macro)
- Frontend served by nginx which reverse-proxies API calls to backend container
- `docker-compose.yml` uses `service_healthy` conditions for startup ordering
- Tests are Python (unittest), not Rust — unit tests validate business rules; API tests hit the live backend
- Audit logs have 7-year retention (`retention_expires_at` column, set via `DATE_ADD(NOW(), INTERVAL 7 YEAR)`)
- Version snapshots retained for 180 days (configurable via `VERSION_RETENTION_DAYS`)
- Scheduled transitions processed via `POST /api/v1/approvals/process-scheduled` (admin-only, call from cron/background)
