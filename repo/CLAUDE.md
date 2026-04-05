# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Start all services (one-click)
docker compose up

# Run all tests (unit + API integration)
./run_tests.sh

# Unit tests only (no services needed) — 244 tests
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
- `middleware/reauth_guard.rs` — `ReauthRequired` guard enforces re-auth within 15 min for admin actions; guards: `ReauthAdminGuard`, `ReauthReviewerGuard`
- `middleware/hmac_guard.rs` — HMAC-SHA256 signature verification with nonce anti-replay (5 min expiry)
- `middleware/correlation.rs` — `X-Correlation-Id` fairing, auto-generates UUID if not present
- `middleware/csrf_guard.rs` — Origin header check fairing for state-changing requests (defense-in-depth; primary CSRF defense is Bearer token auth)
- `middleware/client_ip.rs` — Client IP extraction for login rate limiting
- `auth/` — JWT (HS256), bcrypt passwords, HMAC signing utilities
- `dto/` — request/response schemas per domain (auth, course, approval, booking, term)
- `services/` — business logic: auth_service, course_service, approval_service, version_service, audit_service, term_service, booking_service, risk_service, privacy_service, crypto_service, webhook_service
- `repositories/login_rate_limit_repo.rs` — IP-based login attempt tracking for rate limiting and account lockout
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

### Booking Status Flow

- If `resource.requires_approval == false`: booking created with `status='confirmed'` immediately.
- If `resource.requires_approval == true`: booking created with `status='pending'`, requires reviewer/admin approval.
- `POST /api/v1/bookings/<uuid>/approve` (ReviewerGuard) — transitions pending → confirmed. Approval is department-scoped: reviewers can only approve bookings within their department.
- `POST /api/v1/bookings/<uuid>/reject` (ReviewerGuard) — transitions pending → cancelled. Also department-scoped.
- `GET /api/v1/bookings/pending-approvals` — list pending bookings for reviewer's department.
- `GET /api/v1/bookings/<uuid>/booker-breaches` — view breaches for a specific booker.
- Late cancellation breach only applies to confirmed bookings (not pending).

### Terms Acceptance

- `POST /api/v1/terms/<term_uuid>/accept` — idempotent acceptance of a term.
- `GET /api/v1/terms/my-acceptances` — list user's term acceptances.
- Enforcement: active term acceptance required before booking creation and course approval submission.

### Media Pipeline

- `POST /api/v1/courses/media/upload` — multipart file upload (PDF/MP4/PNG, max 500MB). Stores file locally, creates record with `status='pending_scan'`.
- `POST /api/v1/courses/media` — metadata-only registration (file stored externally). Also creates with `status='pending_scan'`.
- `POST /api/v1/courses/media/<uuid>/validate` — deterministic validation (extension↔MIME match, checksum, absolute path). Transitions to `ready` or `failed`.
- Course approval submission blocked if any media assets are not in `ready` status.

### Login Rate Limiting

- IP-based rate limit: 10 login requests per minute per IP address.
- Account lockout: after 5 consecutive failed login attempts, the account is locked for 15 minutes.
- Tracked in `login_rate_limit_repo` via the `login_attempts` table.

### HMAC Key Provisioning

- `POST /api/v1/auth/hmac-keys` (admin-only) — provisions new HMAC key pairs for service-to-service auth.
- Dev key seeded at startup: key ID `dev-scheduler-key`, secret `campus-learn-hmac-dev-secret-2024`.
- Used by `process-scheduled` endpoint instead of JWT auth.

### Privacy Workflows

- **Export**: `POST /api/v1/privacy/requests` with type `export` → admin approves → system generates real export files containing the user's data.
- **Delete**: `POST /api/v1/privacy/requests` with type `delete` → admin approves → system anonymizes user data (replaces PII with anonymized placeholders).
- **Rectify**: `POST /api/v1/privacy/requests` with type `rectify` → admin approves → system updates specific fields as requested.

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
- Scheduled transitions processed via `POST /api/v1/approvals/process-scheduled` (HMAC-authenticated, not JWT — call from cron/background with HMAC headers)
