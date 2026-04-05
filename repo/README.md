# CampusLearn Operations Suite

Enterprise-grade campus learning management and operations platform.

**Stack**: Rust (Rocket) backend, Dioxus (WebAssembly) frontend, MySQL 8.0 database.

## Architecture

```
  Browser
     │ HTTPS :443 / HTTP :80 (→ redirect)
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────────┐     ┌───────────┐
│    Proxy      │────▶│   Frontend   │────▶│   Backend API    │────▶│  MySQL    │
│  nginx TLS    │     │ Dioxus/WASM  │     │  Rust + Rocket   │     │   8.0     │
│  :80/:443     │     │ nginx :3000  │     │    :8000         │     │  :3307    │
└──────────────┘     └──────────────┘     └──────────────────┘     └───────────┘
  TLS termination      SPA + proxy              /api/v1              Auto-migrated
  HTTP→HTTPS           (internal only)     (internal + dev port)
```

- **Backend**: Layered architecture (routes → services → repositories → models), JWT auth, bcrypt passwords, HMAC signing, rate limiting
- **Frontend**: Dioxus 0.6 WASM with role-aware navigation, toast notifications, modals, dark enterprise theme
- **Database**: MySQL 8.0 with 35+ tables, auto-migrated via sqlx on startup
- **Security**: AES-256-GCM field encryption, SSN/bank detail masking, re-auth for admin actions, nonce anti-replay, CSRF defense (see below)

## Security: CSRF Defense Model

This application uses **Bearer token authentication** (JWT in `Authorization` header), not cookie-based sessions. This inherently mitigates classic CSRF attacks because browsers do not automatically attach custom headers to cross-origin requests. The defense stack is:

1. **Bearer token auth** — `Authorization: Bearer <token>` header is never auto-sent by browsers on cross-origin requests.
2. **CORS allowlist** — Single-origin strict policy (`ALLOWED_ORIGIN` env var); cross-origin preflights for state-changing methods are blocked.
3. **Origin header monitoring** — `CsrfOriginCheck` fairing logs warnings when state-changing requests (POST/PUT/PATCH/DELETE) arrive with unexpected `Origin` headers.
4. **Proxy security headers** — HSTS, X-Content-Type-Options, X-Frame-Options, Referrer-Policy set at the TLS proxy layer.

If the application migrates to cookie-based sessions in the future, an anti-CSRF token pattern (e.g., double-submit cookie or synchronizer token) must be added.

## How to Run

```bash
docker compose up
```

That's it. No `.env` files, no manual DB imports, no manual steps. All services start automatically with health-check-based ordering.

### Local Frontend Development

To run the frontend independently for development:

```bash
cd frontend

# Prerequisites (one-time)
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.5 --locked

# Start dev server (ensure backend is running at http://localhost:8000)
trunk serve --port 3000
```

The Dioxus API client auto-detects the origin and routes API calls to the backend. In Docker, nginx proxies `/api/` requests; in local dev, `trunk serve` serves the WASM app and API calls go directly to `http://localhost:8000`.

## Service Addresses

| Service  | URL                         | Description                              |
|----------|-----------------------------|------------------------------------------|
| App      | https://localhost            | Web UI via HTTPS proxy (self-signed cert)|
| App (redirect) | http://localhost       | Redirects to HTTPS automatically         |
| Backend  | http://localhost:8000        | **Dev-only** — via override file, not for CI/prod |
| Health   | https://localhost/health     | Health check through proxy               |
| API Info | http://localhost:8000/api/v1/info | API version info (direct)           |
| MySQL    | localhost:3307               | Database (external port)                 |

> **Self-signed certificate**: The dev proxy uses a self-signed TLS cert. Browsers will show a security warning — click "Advanced → Proceed" to continue. Use `curl -k` / `--insecure` in scripts.
>
> **Production**: Bind-mount real certificates over `/etc/nginx/certs/` in the proxy container before starting.

## Default Accounts

| Username | Password         | Role            | Access Level                    |
|----------|------------------|-----------------|---------------------------------|
| admin    | Admin@12345678   | admin           | Full access to all features     |
| author   | Author@1234567   | staff_author    | Create/edit courses, submit     |
| reviewer | Review@1234567   | dept_reviewer   | Review and approve courses      |
| faculty  | Faculty@123456   | faculty         | View published courses, book    |
| student  | Student@12345    | student         | View published courses          |

All accounts are seeded automatically on first startup.

## Verification Method

### Quick API Check

All API verification should use the HTTPS proxy (default path). Direct backend access on port 8000 is available **only in local development** via `docker-compose.override.yml` and must never be used in CI/CD, staging, or production.

```bash
# Health (HTTPS — primary verification path; -k for self-signed dev cert)
curl -k https://localhost/health

# Login as admin (HTTPS)
curl -sk -X POST https://localhost/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"Admin@12345678"}' | jq .data.token
```

<details>
<summary>Direct backend access (local dev only)</summary>

The `docker-compose.override.yml` file (auto-merged by `docker compose up`) exposes port 8000 on the host for local development convenience. To run without direct backend exposure:

```bash
docker compose -f docker-compose.yml up   # no override, HTTPS-only
```

```bash
# Direct backend (dev only, no TLS)
curl http://localhost:8000/health
```

</details>

### UI Verification Flows

**1. Login & Role-Based Navigation**
1. Open https://localhost → accept self-signed cert warning → redirects to login page
2. Login as `admin` / `Admin@12345678` → full sidebar: Dashboard, Courses, Approvals, Bookings, Risk & Compliance, Audit Trail, Privacy & Data
3. Logout, login as `student` / `Student@12345` → minimal sidebar: Dashboard, Courses, Bookings, Privacy & Data (no admin sections)

**2. Course Lifecycle (create → approve → publish)**
1. Login as `author` → Courses → click "+ New Course"
2. Enter code `CS101`, title `Intro to CS`, description → click Create
3. Click the course → see detail page with "draft" badge
4. Click "Edit" → add a section "Week 1" → add a lesson "Variables" with content
5. Back to detail → click "Submit for Approval" → enter release notes + effective date (e.g. `06/15/2025 09:00 AM`)
6. Logout, login as `reviewer` → Approvals → see pending item → click Review → Approve
7. Logout, login as `admin` → Approvals → see pending step 2 → Approve → course becomes `approved_scheduled` or `published`
8. Login as `faculty` → Courses → see published course

**3. Two-Step Approval with Self-Approval Prevention**
- Author submits for approval
- Author cannot review (gets 403 — not a reviewer role)
- Reviewer approves step 1
- Admin approves step 2

**4. Scheduled Publish**
- Submit with future effective date → course status becomes `approved_scheduled`
- `POST /api/v1/approvals/process-scheduled` requires HMAC headers (not admin JWT). Use the dev scheduler key:

```bash
# Process scheduled transitions (HMAC-authenticated, not JWT)
# Required headers: X-HMAC-Key-Id, X-HMAC-Nonce, X-HMAC-Timestamp, X-HMAC-Signature
NONCE=$(uuidgen)
TIMESTAMP=$(date +%s)
BODY=""
STRING_TO_SIGN="POST\n/api/v1/approvals/process-scheduled\n${TIMESTAMP}\n${NONCE}\n${BODY}"
SIGNATURE=$(printf "$STRING_TO_SIGN" | openssl dgst -sha256 -hmac "campus-learn-hmac-dev-secret-2024" -binary | base64)

curl -sk -X POST https://localhost/api/v1/approvals/process-scheduled \
  -H "X-HMAC-Key-Id: dev-scheduler-key" \
  -H "X-HMAC-Nonce: $NONCE" \
  -H "X-HMAC-Timestamp: $TIMESTAMP" \
  -H "X-HMAC-Signature: $SIGNATURE"
```

**5. Resource Booking**
1. Login as `faculty` → Bookings → see Resources tab with rooms/labs/parking
2. Click "+ New Booking" → select Conference Room A → enter times within 07:00-22:00, max 4 hours
3. Book successfully → appears in My Bookings
4. Try same slot again → conflict error
5. Reschedule booking (max 2 times)
6. Cancel booking → if within 2 hours of start, breach is generated

**6. Late Cancellation Breach & Auto-Restriction**
- Create booking for 1 hour from now
- Cancel it → breach created (visible in Breaches tab)
- After 3 breaches in 60 days → automatic booking restriction applied

**7. Risk & Compliance (admin only)**
1. Login as `admin` → Risk & Compliance
2. See risk rules (seeded: posting frequency, blacklisted employer, abnormal compensation, duplicate posting)
3. Click "Run Evaluation" → evaluates all rules
4. Risk events appear with severity scores → acknowledge/escalate

**8. Privacy & Data Requests**
1. Login as `faculty` → Privacy & Data → "New Data Request" → select Export → submit
2. Logout, login as `admin` → Privacy & Data → see pending request → Approve
3. Request processed and marked completed

**9. Sensitive Data Masking**
- Store sensitive data via API: `POST /api/v1/privacy/sensitive` with `{"field_name": "ssn", "value": "123-45-6789"}`
- Retrieve masked: `GET /api/v1/privacy/sensitive` → returns `***-**-####` (AES-256-GCM encrypted at rest)

**10. Audit Trail (admin only)**
- Login as `admin` → Audit Trail → see all system actions with actor, entity, correlation ID, timestamp

## Running Tests

```bash
# All tests with summary
./run_tests.sh

# Unit tests only (no services required) — 244+ tests
python3 -m unittest discover -s unit_tests -p "test_*.py" -v

# API tests only (requires running services)
# Default: direct backend port for test speed. For HTTPS-verified testing, use the proxy URL.
API_BASE_URL=http://localhost:8000 python3 -m unittest discover -s API_tests -p "test_*.py" -v

# Single test file
python3 -m unittest unit_tests.backend.test_booking_rules -v
```

### Test Coverage

**Unit Tests (244+):**
- Password complexity validation (13 tests)
- RBAC permission checks and self-approval prevention (14 tests)
- Version diff generation (8 tests)
- Scheduled publish date parsing and status transitions (12 tests)
- Media type/size validation (9 tests)
- API response shape contracts (4 tests)
- JWT claim structure (7 tests)
- Booking conflict detection, 90-day limit, 4-hour cap, reschedule limits, breach generation, auto-restriction (25 tests)
- Risk rule evaluation, blacklist, scoring, nonce expiry (21 tests)
- Frontend route definitions and role navigation (9 tests)

**API Integration Tests (60+):**
- Auth: login all roles, /me, reauth, invalid credentials
- Courses: CRUD, sections, lessons, role enforcement
- Approvals: full two-step workflow, self-approval prevention, rejection
- Bookings: happy path, conflict detection, invalid hours, reschedule, cancel
- Risk: rules, events, evaluation, blacklist, subscriptions
- Privacy: export approval flow, deletion approval flow, sensitive data masking
- Audit: admin access, role enforcement
- Health/Info: endpoint availability

## API Endpoints (55 total)

### Auth (5)
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | /api/v1/auth/login | No | Login |
| GET | /api/v1/auth/me | Bearer | Current user |
| POST | /api/v1/auth/change-password | Bearer | Change password |
| POST | /api/v1/auth/reauth | Bearer | Re-authenticate |
| POST | /api/v1/auth/hmac-keys | Bearer (Admin) | Provision HMAC keys |

> **Login rate limiting**: Login attempts are limited to 10 requests/min per IP address. After 5 consecutive failed login attempts for an account, the account is locked out for 15 minutes.

### Courses (16)
CRUD for courses (5), sections (4), lessons (3), media upload (1), media register (1), media validate (1), versions (1)

### Approvals (5)
Submit, review, get, queue, process-scheduled

### Tags (2)
Create, list

### Bookings (12)
Resources, availability, create, reschedule, cancel, approve, reject, my bookings, breaches, restrictions, pending-approvals, booker-breaches

### Risk (8)
Rules, events, update event, evaluate, postings, blacklist, subscriptions (create/list)

### Privacy (6)
Create request, list all, my requests, review, store sensitive, get masked

### Audit (1)
List audit logs (admin, filterable)

### System (2)
Health check, API info

## Project Structure

```
repo/
├── backend/                    # Rust + Rocket (65 source files)
│   ├── src/
│   │   ├── auth/               # JWT, bcrypt, HMAC
│   │   ├── config/             # Environment config
│   │   ├── dto/                # Request/response schemas
│   │   ├── jobs/               # Background job definitions
│   │   ├── middleware/         # Auth guards, rate limiter, HMAC, correlation
│   │   ├── models/             # Database entities
│   │   ├── repositories/       # SQL data access layer
│   │   ├── routes/             # HTTP handlers
│   │   ├── services/           # Business logic
│   │   └── utils/              # Errors, response helpers
│   ├── migrations/             # 3 SQL migration files
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/                   # Dioxus WASM (26 source files)
│   ├── src/
│   │   ├── api/                # Typed HTTP client
│   │   ├── auth/               # Global auth state
│   │   ├── components/         # StatusBadge, Modal, Toast, DataTable, etc.
│   │   ├── layouts/            # Sidebar + main content
│   │   ├── pages/              # 10 page components
│   │   └── types/              # All API response types
│   ├── assets/main.css         # 200-line design system
│   └── Dockerfile
├── proxy/                      # nginx TLS termination proxy
│   ├── nginx.conf              # HTTP→HTTPS redirect + TLS reverse proxy
│   └── Dockerfile              # Builds nginx with self-signed dev cert
├── mysql/init/                 # DB charset init
├── unit_tests/                 # 127 Python unit tests
├── API_tests/                  # 60+ Python API integration tests
├── docker-compose.yml
├── run_tests.sh
├── SELF_TEST_REPORT.md
└── README.md
```
