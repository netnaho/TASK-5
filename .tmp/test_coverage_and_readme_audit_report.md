# Test Coverage Audit

## Scope and Method
- Audit mode: static inspection only (no test execution, no app runtime).
- Project type declaration at README top: **missing**.
- Inferred project type (strict): **fullstack** (evidence: [repo/README.md:5], [repo/frontend/src/main.rs], [repo/backend/src/main.rs]).

## Backend Endpoint Inventory
Resolved from backend route mounts and route macros.

- `GET /health` ([repo/backend/src/main.rs:96], [repo/backend/src/routes/health.rs:13])
- `GET /api/v1/info` ([repo/backend/src/main.rs:97], [repo/backend/src/routes/info.rs:15])

- `POST /api/v1/auth/login` ([repo/backend/src/main.rs:98], [repo/backend/src/routes/auth.rs:18])
- `POST /api/v1/auth/change-password` ([repo/backend/src/routes/auth.rs:55])
- `POST /api/v1/auth/reauth` ([repo/backend/src/routes/auth.rs:75])
- `GET /api/v1/auth/me` ([repo/backend/src/routes/auth.rs:88])
- `POST /api/v1/auth/hmac-keys` ([repo/backend/src/routes/auth.rs:125])

- `POST /api/v1/courses` ([repo/backend/src/main.rs:99], [repo/backend/src/routes/courses.rs:17])
- `GET /api/v1/courses` ([repo/backend/src/routes/courses.rs:37])
- `GET /api/v1/courses/:uuid` ([repo/backend/src/routes/courses.rs:49])
- `PUT /api/v1/courses/:uuid` ([repo/backend/src/routes/courses.rs:61])
- `DELETE /api/v1/courses/:uuid` ([repo/backend/src/routes/courses.rs:73])
- `POST /api/v1/courses/:course_uuid/sections` ([repo/backend/src/routes/courses.rs:85])
- `GET /api/v1/courses/:course_uuid/sections` ([repo/backend/src/routes/courses.rs:104])
- `PUT /api/v1/courses/sections/:uuid` ([repo/backend/src/routes/courses.rs:116])
- `DELETE /api/v1/courses/sections/:uuid` ([repo/backend/src/routes/courses.rs:128])
- `POST /api/v1/courses/sections/:section_uuid/lessons` ([repo/backend/src/routes/courses.rs:140])
- `PUT /api/v1/courses/lessons/:uuid` ([repo/backend/src/routes/courses.rs:159])
- `DELETE /api/v1/courses/lessons/:uuid` ([repo/backend/src/routes/courses.rs:171])
- `POST /api/v1/courses/media/upload` ([repo/backend/src/routes/courses.rs:191])
- `POST /api/v1/courses/media` ([repo/backend/src/routes/courses.rs:237])
- `POST /api/v1/courses/media/:uuid/validate` ([repo/backend/src/routes/courses.rs:254])
- `GET /api/v1/courses/:course_uuid/versions` ([repo/backend/src/routes/courses.rs:266])

- `POST /api/v1/approvals/:course_uuid/submit` ([repo/backend/src/main.rs:100], [repo/backend/src/routes/approvals.rs:17])
- `POST /api/v1/approvals/:approval_uuid/review` ([repo/backend/src/routes/approvals.rs:39])
- `GET /api/v1/approvals/:uuid` ([repo/backend/src/routes/approvals.rs:56])
- `GET /api/v1/approvals/queue` ([repo/backend/src/routes/approvals.rs:68])
- `POST /api/v1/approvals/process-scheduled` ([repo/backend/src/routes/approvals.rs:78])
- `POST /api/v1/approvals/:course_uuid/unpublish` ([repo/backend/src/routes/approvals.rs:88])

- `GET /api/v1/audit` ([repo/backend/src/main.rs:101], [repo/backend/src/routes/audit.rs:12])

- `POST /api/v1/tags` ([repo/backend/src/main.rs:102], [repo/backend/src/routes/tags.rs:14])
- `GET /api/v1/tags` ([repo/backend/src/routes/tags.rs:31])

- `GET /api/v1/bookings/resources` ([repo/backend/src/main.rs:103], [repo/backend/src/routes/bookings.rs:15])
- `GET /api/v1/bookings/resources/:uuid/availability` ([repo/backend/src/routes/bookings.rs:25])
- `POST /api/v1/bookings` ([repo/backend/src/routes/bookings.rs:37])
- `POST /api/v1/bookings/:uuid/reschedule` ([repo/backend/src/routes/bookings.rs:54])
- `POST /api/v1/bookings/:uuid/cancel` ([repo/backend/src/routes/bookings.rs:66])
- `GET /api/v1/bookings/my` ([repo/backend/src/routes/bookings.rs:77])
- `GET /api/v1/bookings/breaches` ([repo/backend/src/routes/bookings.rs:87])
- `POST /api/v1/bookings/:uuid/approve` ([repo/backend/src/routes/bookings.rs:97])
- `POST /api/v1/bookings/:uuid/reject` ([repo/backend/src/routes/bookings.rs:109])
- `GET /api/v1/bookings/pending-approvals` ([repo/backend/src/routes/bookings.rs:121])
- `GET /api/v1/bookings/:uuid/booker-breaches` ([repo/backend/src/routes/bookings.rs:131])
- `GET /api/v1/bookings/restrictions` ([repo/backend/src/routes/bookings.rs:142])

- `GET /api/v1/risk/rules` ([repo/backend/src/main.rs:104], [repo/backend/src/routes/risk.rs:15])
- `GET /api/v1/risk/events` ([repo/backend/src/routes/risk.rs:25])
- `PUT /api/v1/risk/events/:uuid` ([repo/backend/src/routes/risk.rs:36])
- `POST /api/v1/risk/evaluate` ([repo/backend/src/routes/risk.rs:49])
- `POST /api/v1/risk/subscriptions` ([repo/backend/src/routes/risk.rs:60])
- `GET /api/v1/risk/subscriptions` ([repo/backend/src/routes/risk.rs:81])
- `POST /api/v1/risk/postings` ([repo/backend/src/routes/risk.rs:91])
- `POST /api/v1/risk/blacklist` ([repo/backend/src/routes/risk.rs:108])

- `POST /api/v1/privacy/requests` ([repo/backend/src/main.rs:105], [repo/backend/src/routes/privacy.rs:15])
- `GET /api/v1/privacy/requests` ([repo/backend/src/routes/privacy.rs:26])
- `GET /api/v1/privacy/requests/my` ([repo/backend/src/routes/privacy.rs:37])
- `POST /api/v1/privacy/requests/:uuid/review` ([repo/backend/src/routes/privacy.rs:47])
- `POST /api/v1/privacy/sensitive` ([repo/backend/src/routes/privacy.rs:65])
- `GET /api/v1/privacy/sensitive` ([repo/backend/src/routes/privacy.rs:85])

- `GET /api/v1/terms` ([repo/backend/src/main.rs:106], [repo/backend/src/routes/terms.rs:25])
- `GET /api/v1/terms/active` ([repo/backend/src/routes/terms.rs:35])
- `POST /api/v1/terms/:term_uuid/accept` ([repo/backend/src/routes/terms.rs:45])
- `GET /api/v1/terms/my-acceptances` ([repo/backend/src/routes/terms.rs:56])

- `GET /api/v1/notifications` ([repo/backend/src/main.rs:107], [repo/backend/src/routes/notifications.rs:13])
- `GET /api/v1/notifications/unread-count` ([repo/backend/src/routes/notifications.rs:36])
- `PUT /api/v1/notifications/:uuid/read` ([repo/backend/src/routes/notifications.rs:48])
- `PUT /api/v1/notifications/read-all` ([repo/backend/src/routes/notifications.rs:61])

**Total endpoints:** 66

## API Test Mapping Table
Legend:
- Test type = `true no-mock HTTP` / `HTTP with mocking` / `unit-only or indirect`
- Coverage decision based on exact method + normalized path seen in API test calls.

| Endpoint | Covered | Test type | Test files | Evidence |
|---|---|---|---|---|
| GET /health | yes | true no-mock HTTP | `test_health.py`, `test_envelope.py`, `test_rate_limit.py` | [repo/API_tests/test_health.py:26], [repo/API_tests/test_envelope.py:37], [repo/API_tests/test_rate_limit.py:114] |
| GET /api/v1/info | yes | true no-mock HTTP | `test_health.py`, `test_envelope.py`, `test_security_regression.py` | [repo/API_tests/test_health.py:40], [repo/API_tests/test_envelope.py:46], [repo/API_tests/test_security_regression.py:947] |
| POST /api/v1/auth/login | yes | true no-mock HTTP | many | [repo/API_tests/test_auth.py:73] |
| POST /api/v1/auth/change-password | yes | true no-mock HTTP | `test_auth.py`, `test_security_regression.py` | [repo/API_tests/test_auth.py:154], [repo/API_tests/test_security_regression.py:710] |
| POST /api/v1/auth/reauth | yes | true no-mock HTTP | many | [repo/API_tests/test_auth.py:130] |
| GET /api/v1/auth/me | yes | true no-mock HTTP | `test_auth.py`, `test_envelope.py`, `test_rate_limit.py` | [repo/API_tests/test_auth.py:107], [repo/API_tests/test_rate_limit.py:56] |
| POST /api/v1/auth/hmac-keys | yes | true no-mock HTTP | `test_hmac_flow.py` | [repo/API_tests/test_hmac_flow.py:61] |
| POST /api/v1/courses | yes | true no-mock HTTP | `test_courses.py`, `test_approvals.py`, `test_scope_isolation.py` | [repo/API_tests/test_courses.py:52] |
| GET /api/v1/courses | yes | true no-mock HTTP | `test_courses.py`, `test_scope_isolation.py`, `test_security_regression.py` | [repo/API_tests/test_courses.py:64] |
| GET /api/v1/courses/:uuid | yes | true no-mock HTTP | `test_courses.py`, `test_approvals.py`, `test_scope_isolation.py` | [repo/API_tests/test_courses.py:71] |
| PUT /api/v1/courses/:uuid | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:79] |
| DELETE /api/v1/courses/:uuid | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:128] |
| POST /api/v1/courses/:course_uuid/sections | yes | true no-mock HTTP | `test_courses.py`, `test_security_regression.py` | [repo/API_tests/test_courses.py:96] |
| GET /api/v1/courses/:course_uuid/sections | yes | true no-mock HTTP | `test_courses.py`, `test_scope_isolation.py` | [repo/API_tests/test_courses.py:116] |
| PUT /api/v1/courses/sections/:uuid | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:234] |
| DELETE /api/v1/courses/sections/:uuid | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:241] |
| POST /api/v1/courses/sections/:section_uuid/lessons | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:106] |
| PUT /api/v1/courses/lessons/:uuid | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:255] |
| DELETE /api/v1/courses/lessons/:uuid | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:262] |
| POST /api/v1/courses/media/upload | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:379] |
| POST /api/v1/courses/media | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:419] |
| POST /api/v1/courses/media/:uuid/validate | yes | true no-mock HTTP | `test_courses.py` | [repo/API_tests/test_courses.py:410] |
| GET /api/v1/courses/:course_uuid/versions | yes | true no-mock HTTP | `test_courses.py`, `test_approvals.py`, `test_security_regression.py` | [repo/API_tests/test_courses.py:310] |
| POST /api/v1/approvals/:course_uuid/submit | yes | true no-mock HTTP | `test_approvals.py`, `test_courses.py`, `test_scope_isolation.py` | [repo/API_tests/test_approvals.py:89] |
| POST /api/v1/approvals/:approval_uuid/review | yes | true no-mock HTTP | `test_approvals.py`, `test_reauth_enforcement.py` | [repo/API_tests/test_approvals.py:108] |
| GET /api/v1/approvals/:uuid | yes | true no-mock HTTP | `test_approvals.py`, `test_security_regression.py` | [repo/API_tests/test_approvals.py:99] |
| GET /api/v1/approvals/queue | yes | true no-mock HTTP | `test_approvals.py`, `test_scope_isolation.py` | [repo/API_tests/test_approvals.py:142] |
| POST /api/v1/approvals/process-scheduled | no | unit-only or indirect | none | no API test call found in `repo/API_tests` |
| POST /api/v1/approvals/:course_uuid/unpublish | yes | true no-mock HTTP | `test_approvals.py` | [repo/API_tests/test_approvals.py:334] |
| GET /api/v1/audit | yes | true no-mock HTTP | `test_audit.py`, `test_envelope.py`, `test_reauth_enforcement.py` | [repo/API_tests/test_audit.py:52], [repo/API_tests/test_audit.py:65] |
| POST /api/v1/tags | no | unit-only or indirect | none | no API test call found in `repo/API_tests` |
| GET /api/v1/tags | no | unit-only or indirect | none | no API test call found in `repo/API_tests` |
| GET /api/v1/bookings/resources | yes | true no-mock HTTP | `test_bookings.py`, `test_booking_approval.py`, `test_department_booking.py` | [repo/API_tests/test_bookings.py:76] |
| GET /api/v1/bookings/resources/:uuid/availability | yes | true no-mock HTTP | `test_bookings.py` | [repo/API_tests/test_bookings.py:88] |
| POST /api/v1/bookings | yes | true no-mock HTTP | `test_bookings.py`, `test_booking_approval.py`, `test_terms_acceptance.py` | [repo/API_tests/test_bookings.py:98] |
| POST /api/v1/bookings/:uuid/reschedule | yes | true no-mock HTTP | `test_bookings.py` | [repo/API_tests/test_bookings.py:131] |
| POST /api/v1/bookings/:uuid/cancel | yes | true no-mock HTTP | `test_bookings.py`, `test_booking_approval.py` | [repo/API_tests/test_bookings.py:142] |
| GET /api/v1/bookings/my | yes | true no-mock HTTP | `test_bookings.py` | [repo/API_tests/test_bookings.py:123] |
| GET /api/v1/bookings/breaches | yes | true no-mock HTTP | `test_bookings.py` | [repo/API_tests/test_bookings.py:146] |
| POST /api/v1/bookings/:uuid/approve | yes | true no-mock HTTP | `test_booking_approval.py`, `test_reauth_enforcement.py` | [repo/API_tests/test_booking_approval.py:124] |
| POST /api/v1/bookings/:uuid/reject | yes | true no-mock HTTP | `test_booking_approval.py`, `test_reauth_enforcement.py` | [repo/API_tests/test_booking_approval.py:145] |
| GET /api/v1/bookings/pending-approvals | yes | true no-mock HTTP | `test_department_booking.py` | [repo/API_tests/test_department_booking.py:82] |
| GET /api/v1/bookings/:uuid/booker-breaches | yes | true no-mock HTTP | `test_department_booking.py` | [repo/API_tests/test_department_booking.py:140] |
| GET /api/v1/bookings/restrictions | yes | true no-mock HTTP | `test_bookings.py` | [repo/API_tests/test_bookings.py:151] |
| GET /api/v1/risk/rules | yes | true no-mock HTTP | `test_risk.py`, `test_envelope.py` | [repo/API_tests/test_risk.py:55] |
| GET /api/v1/risk/events | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | [repo/API_tests/test_risk.py:78] |
| PUT /api/v1/risk/events/:uuid | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | [repo/API_tests/test_risk.py:191] |
| POST /api/v1/risk/evaluate | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | [repo/API_tests/test_risk.py:71] |
| POST /api/v1/risk/subscriptions | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | [repo/API_tests/test_risk.py:117] |
| GET /api/v1/risk/subscriptions | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | [repo/API_tests/test_risk.py:125] |
| POST /api/v1/risk/postings | yes | true no-mock HTTP | `test_risk.py` | [repo/API_tests/test_risk.py:63] |
| POST /api/v1/risk/blacklist | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | [repo/API_tests/test_risk.py:92] |
| POST /api/v1/privacy/requests | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | [repo/API_tests/test_privacy.py:46] |
| GET /api/v1/privacy/requests | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | [repo/API_tests/test_privacy.py:63] |
| GET /api/v1/privacy/requests/my | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | [repo/API_tests/test_privacy.py:56] |
| POST /api/v1/privacy/requests/:uuid/review | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py`, `test_security_regression.py` | [repo/API_tests/test_privacy.py:69] |
| POST /api/v1/privacy/sensitive | yes | true no-mock HTTP | `test_privacy.py`, `test_security_regression.py` | [repo/API_tests/test_privacy.py:116] |
| GET /api/v1/privacy/sensitive | yes | true no-mock HTTP | `test_privacy.py`, `test_security_regression.py` | [repo/API_tests/test_privacy.py:125] |
| GET /api/v1/terms | yes | true no-mock HTTP | `test_scope_isolation.py`, `test_security_regression.py` | [repo/API_tests/test_scope_isolation.py:61] |
| GET /api/v1/terms/active | yes | true no-mock HTTP | `test_terms_acceptance.py`, `test_courses.py`, `test_bookings.py` | [repo/API_tests/test_terms_acceptance.py:58] |
| POST /api/v1/terms/:term_uuid/accept | yes | true no-mock HTTP | `test_terms_acceptance.py`, `test_courses.py`, `test_bookings.py` | [repo/API_tests/test_terms_acceptance.py:66] |
| GET /api/v1/terms/my-acceptances | yes | true no-mock HTTP | `test_terms_acceptance.py` | [repo/API_tests/test_terms_acceptance.py:80] |
| GET /api/v1/notifications | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | [repo/API_tests/test_notifications.py:45] |
| GET /api/v1/notifications/unread-count | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | [repo/API_tests/test_notifications.py:53] |
| PUT /api/v1/notifications/:uuid/read | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | [repo/API_tests/test_notifications.py:282] |
| PUT /api/v1/notifications/read-all | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | [repo/API_tests/test_notifications.py:62] |

## API Test Classification
- **True No-Mock HTTP:** all files under `repo/API_tests/*.py` send real HTTP requests via `urllib.request` to `API_BASE_URL` (e.g., [repo/API_tests/test_auth.py:30], [repo/API_tests/test_courses.py:18]).
- **HTTP with Mocking:** none detected.
- **Non-HTTP tests:** all files under `repo/unit_tests/backend/*.py` and `repo/unit_tests/frontend/test_route_definitions.py`.

## Mock Detection
- Searched for: `jest.mock`, `vi.mock`, `sinon.stub`, `patch(`, `MagicMock`, `Mock`, monkeypatch patterns across `repo/API_tests` and `repo/unit_tests`.
- Result: no mocking/stubbing indicators found.
- Observation: some API tests mutate DB state via `docker exec ... mysql` subprocess calls (not mocking, but external dependency): [repo/API_tests/test_auth.py:12], [repo/API_tests/test_hmac_flow.py:33], [repo/API_tests/test_risk.py:31], [repo/API_tests/test_department_booking.py:32].

## Coverage Summary
- Total endpoints: **66**
- Endpoints with HTTP tests: **63**
- Endpoints with true no-mock HTTP tests: **63**
- HTTP coverage: **95.45%**
- True API coverage: **95.45%**
- Uncovered endpoints:
  - `POST /api/v1/approvals/process-scheduled`
  - `POST /api/v1/tags`
  - `GET /api/v1/tags`

## Unit Test Summary

### Backend Unit Tests
- Files: `repo/unit_tests/backend/test_*.py` (14 files).
- Covered module categories (mostly mirrored/spec logic, not direct Rust module execution):
  - Services/business-rule mirrors: booking, risk, scheduling ([repo/unit_tests/backend/test_booking_rules.py:1], [repo/unit_tests/backend/test_risk_rules.py:1], [repo/unit_tests/backend/test_job_scheduling.py:1])
  - Auth/permissions/security mirrors: JWT, password, permissions, encryption ([repo/unit_tests/backend/test_jwt_claims.py:1], [repo/unit_tests/backend/test_password.py:1], [repo/unit_tests/backend/test_permissions.py:1], [repo/unit_tests/backend/test_config_encryption.py:1])
  - API envelope/shape mirrors: [repo/unit_tests/backend/test_api_response_shape.py:1]
- Important backend modules not directly unit-tested (production Rust modules not imported/executed):
  - Route handlers in `repo/backend/src/routes/*.rs`
  - Repository layer in `repo/backend/src/repositories/*.rs`
  - Middleware guards/rate limiter in `repo/backend/src/middleware/*.rs`
  - Service implementations in `repo/backend/src/services/*.rs` (tested indirectly via API tests, not via Rust unit tests)

### Frontend Unit Tests (STRICT)
- Frontend test files found: `repo/unit_tests/frontend/test_route_definitions.py`
- Framework/tool evidence for frontend component tests: **none** (Python `unittest`, no Jest/Vitest/RTL/Dioxus test harness) ([repo/unit_tests/frontend/test_route_definitions.py:2])
- Imports/renders of actual frontend components/modules: **none** (file uses hardcoded dictionaries only; no imports from `repo/frontend/src`) ([repo/unit_tests/frontend/test_route_definitions.py:4])
- Components/modules covered: **none (no direct frontend module execution)**
- Important frontend components/modules not tested:
  - `repo/frontend/src/main.rs`
  - `repo/frontend/src/pages/*` (dashboard/courses/approvals/bookings/risk/privacy/audit/login)
  - `repo/frontend/src/components/*`
  - `repo/frontend/src/api/mod.rs`

**Mandatory verdict:** **Frontend unit tests: MISSING**

**CRITICAL GAP (strict rule):** Project is inferred `fullstack`, but strict frontend unit tests are missing.

### Cross-Layer Observation
- Coverage is backend-heavy: strong API HTTP coverage and many backend-spec Python tests.
- Frontend execution-level unit testing and FE↔BE end-to-end testing are absent.

## API Observability Check
- Strengths: many tests show method/path, payload, and response field assertions (e.g., [repo/API_tests/test_auth.py:73], [repo/API_tests/test_courses.py:52], [repo/API_tests/test_risk.py:117]).
- Weak spots: some tests assert mostly status code with shallow body assertions only (e.g., [repo/API_tests/test_department_booking.py:98], [repo/API_tests/test_courses.py:79], [repo/API_tests/test_security_regression.py:624]).
- Verdict: **Moderate observability; partially weak in several negative-path tests.**

## Tests Check
- Success/failure coverage: present across auth, RBAC, approval workflows, booking constraints, risk/privacy security paths.
- Edge/validation/auth checks: present in many API files.
- Over-mocking: not detected.
- `run_tests.sh` check:
  - Uses local Python interpreter and local unittest discovery, not Docker-contained execution: [repo/run_tests.sh:16], [repo/run_tests.sh:55].
  - API test execution requires externally running services (`docker compose up`), not self-contained in script: [repo/run_tests.sh:47].
  - Strict audit flag: local dependency coupling exists.

## Test Coverage Score (0-100)
**79/100**

## Score Rationale
- + High HTTP endpoint coverage (63/66).
- + API tests are real HTTP without explicit mocks.
- + Good auth/permission/failure-path breadth.
- - 3 backend endpoints fully untested (including scheduler endpoint).
- - Frontend unit test requirement fails strict criteria (critical for fullstack).
- - No FE↔BE E2E tests.
- - Many backend “unit tests” are Python mirrors, not direct execution of Rust production modules.

## Key Gaps
- No API coverage for `/api/v1/tags` (GET/POST).
- No API coverage for `/api/v1/approvals/process-scheduled` HMAC path.
- Frontend tests do not execute real frontend code/framework; strict frontend unit testing is missing.
- No fullstack E2E user-journey tests.

## Confidence & Assumptions
- Confidence: **high** for endpoint inventory and HTTP call mapping.
- Assumption: `api_request/api/_http` helpers in API tests represent real HTTP calls to running backend.
- Static-only limitation: runtime route matching behavior (e.g., trailing slash equivalence) not validated by execution.

## Test Coverage Verdict
**PARTIAL PASS (with CRITICAL GAP on frontend unit testing).**

---

# README Audit

## README Location
- Found at required path: `repo/README.md`.

## Hard Gate Results

### Formatting
- PASS: Structured markdown with headings, tables, and code blocks ([repo/README.md:1]).

### Startup Instructions (Fullstack)
- **FAIL (hard gate):** Required literal instruction `docker-compose up` is missing.
- Found `docker compose up` instead ([repo/README.md:35]).

### Access Method
- PASS: URL/port access documented for app/backend/health/db ([repo/README.md:56]).

### Verification Method
- PASS: Includes API verification via `curl` and UI verification flows ([repo/README.md:84], [repo/README.md:113]).

### Environment Rules (Docker-contained only)
- **FAIL (hard gate):** README includes runtime local installs and non-containerized local dev/test commands:
  - `rustup target add wasm32-unknown-unknown` ([repo/README.md:45])
  - `cargo install trunk --version 0.21.5 --locked` ([repo/README.md:46])
  - direct local `python3 -m unittest ...` flows ([repo/README.md:245])

### Demo Credentials (Auth exists)
- PASS: Credentials provided with username + password + roles for seeded accounts ([repo/README.md:71], [repo/backend/src/services/seed.rs:22]).

## Engineering Quality Findings

### High Priority Issues
- README endpoint count is incorrect: claims 55; static route inventory is 66 ([repo/README.md:264], [repo/backend/src/main.rs:96]).
- README project structure claims `migrations` has 3 files; repository has 12 migrations ([repo/README.md:308], `repo/backend/migrations/*.sql`).
- Fullstack test guidance overstates frontend test validity; current frontend “unit test” is a Python dictionary check, not real frontend module/component testing ([repo/README.md:258], [repo/unit_tests/frontend/test_route_definitions.py:1]).

### Medium Priority Issues
- Project type label required by audit policy (`backend/fullstack/web/android/ios/desktop`) is not declared at top ([repo/README.md:1]).
- README uses both HTTPS proxy and direct backend paths; may cause environment inconsistency if interpreted as canonical startup verification path ([repo/README.md:84], [repo/README.md:100]).

### Low Priority Issues
- Some quantitative claims appear stale (source file counts) and should be auto-derived or removed ([repo/README.md:300]).

## Hard Gate Failures
- Missing exact `docker-compose up` startup command string.
- Violates Docker-contained environment rule by including runtime/local install paths.

## README Verdict
**FAIL**

