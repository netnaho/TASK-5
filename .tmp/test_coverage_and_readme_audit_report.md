# Test Coverage Audit

## Scope and Method
- Audit mode: static inspection only (no execution of tests, scripts, containers, server, or build tools).
- Project type declaration found in README: **fullstack** ([repo/README.md:3]).
- Inferred type from code structure also matches fullstack ([repo/frontend/src/main.rs], [repo/backend/src/main.rs]).

## Backend Endpoint Inventory
Resolved from route mounts in [repo/backend/src/main.rs:112-123] and route macros in `repo/backend/src/routes/*.rs`.

- `GET /health`
- `GET /api/v1/info`

- `POST /api/v1/auth/login`
- `POST /api/v1/auth/change-password`
- `POST /api/v1/auth/reauth`
- `GET /api/v1/auth/me`
- `POST /api/v1/auth/hmac-keys`

- `POST /api/v1/courses`
- `GET /api/v1/courses`
- `GET /api/v1/courses/:uuid`
- `PUT /api/v1/courses/:uuid`
- `DELETE /api/v1/courses/:uuid`
- `POST /api/v1/courses/:course_uuid/sections`
- `GET /api/v1/courses/:course_uuid/sections`
- `PUT /api/v1/courses/sections/:uuid`
- `DELETE /api/v1/courses/sections/:uuid`
- `POST /api/v1/courses/sections/:section_uuid/lessons`
- `PUT /api/v1/courses/lessons/:uuid`
- `DELETE /api/v1/courses/lessons/:uuid`
- `POST /api/v1/courses/media/upload`
- `POST /api/v1/courses/media`
- `POST /api/v1/courses/media/:uuid/validate`
- `GET /api/v1/courses/:course_uuid/versions`

- `POST /api/v1/approvals/:course_uuid/submit`
- `POST /api/v1/approvals/:approval_uuid/review`
- `GET /api/v1/approvals/:uuid`
- `GET /api/v1/approvals/queue`
- `POST /api/v1/approvals/process-scheduled`
- `POST /api/v1/approvals/:course_uuid/unpublish`

- `GET /api/v1/audit`

- `POST /api/v1/tags`
- `GET /api/v1/tags`

- `GET /api/v1/bookings/resources`
- `GET /api/v1/bookings/resources/:uuid/availability`
- `POST /api/v1/bookings`
- `POST /api/v1/bookings/:uuid/reschedule`
- `POST /api/v1/bookings/:uuid/cancel`
- `GET /api/v1/bookings/my`
- `GET /api/v1/bookings/breaches`
- `POST /api/v1/bookings/:uuid/approve`
- `POST /api/v1/bookings/:uuid/reject`
- `GET /api/v1/bookings/pending-approvals`
- `GET /api/v1/bookings/:uuid/booker-breaches`
- `GET /api/v1/bookings/restrictions`

- `GET /api/v1/risk/rules`
- `GET /api/v1/risk/events`
- `PUT /api/v1/risk/events/:uuid`
- `POST /api/v1/risk/evaluate`
- `POST /api/v1/risk/subscriptions`
- `GET /api/v1/risk/subscriptions`
- `POST /api/v1/risk/postings`
- `POST /api/v1/risk/blacklist`

- `POST /api/v1/privacy/requests`
- `GET /api/v1/privacy/requests`
- `GET /api/v1/privacy/requests/my`
- `POST /api/v1/privacy/requests/:uuid/review`
- `POST /api/v1/privacy/sensitive`
- `GET /api/v1/privacy/sensitive`

- `GET /api/v1/terms`
- `GET /api/v1/terms/active`
- `POST /api/v1/terms/:term_uuid/accept`
- `GET /api/v1/terms/my-acceptances`

- `GET /api/v1/notifications`
- `GET /api/v1/notifications/unread-count`
- `PUT /api/v1/notifications/:uuid/read`
- `PUT /api/v1/notifications/read-all`

**Total endpoints: 66**

## API Test Mapping Table
Legend:
- Test type = `true no-mock HTTP` / `HTTP with mocking` / `unit-only or indirect`
- Coverage is based on exact method + normalized path presence in `repo/API_tests` request calls.

| Endpoint | Covered | Test type | Test files | Evidence |
|---|---|---|---|---|
| GET /health | yes | true no-mock HTTP | `test_health.py`, `test_envelope.py`, `test_rate_limit.py` | `api_get("/health")`, `api_request("GET", "/health")` |
| GET /api/v1/info | yes | true no-mock HTTP | `test_health.py`, `test_envelope.py`, `test_security_regression.py` | `api_get("/api/v1/info")` |
| POST /api/v1/auth/login | yes | true no-mock HTTP | `test_auth.py` (+many) | `api_post("/api/v1/auth/login", ...)` |
| POST /api/v1/auth/change-password | yes | true no-mock HTTP | `test_auth.py`, `test_security_regression.py` | `api_post("/api/v1/auth/change-password", ...)` |
| POST /api/v1/auth/reauth | yes | true no-mock HTTP | `test_auth.py`, `test_reauth_enforcement.py` | `api_post("/api/v1/auth/reauth", ...)` |
| GET /api/v1/auth/me | yes | true no-mock HTTP | `test_auth.py`, `test_envelope.py` | `api_get("/api/v1/auth/me", ...)` |
| POST /api/v1/auth/hmac-keys | yes | true no-mock HTTP | `test_hmac_flow.py` | `api_request("POST", "/api/v1/auth/hmac-keys", ...)` |
| POST /api/v1/courses | yes | true no-mock HTTP | `test_courses.py`, `test_approvals.py` | `api_request("POST", "/api/v1/courses", ...)` |
| GET /api/v1/courses | yes | true no-mock HTTP | `test_courses.py`, `test_scope_isolation.py` | `api_request("GET", "/api/v1/courses", ...)` |
| GET /api/v1/courses/:uuid | yes | true no-mock HTTP | `test_courses.py`, `test_approvals.py` | `api_request("GET", f"/api/v1/courses/{...}", ...)` |
| PUT /api/v1/courses/:uuid | yes | true no-mock HTTP | `test_courses.py` | `api_request("PUT", f"/api/v1/courses/{...}", ...)` |
| DELETE /api/v1/courses/:uuid | yes | true no-mock HTTP | `test_courses.py` | `api_request("DELETE", f"/api/v1/courses/{...}", ...)` |
| POST /api/v1/courses/:course_uuid/sections | yes | true no-mock HTTP | `test_courses.py` | `api_request("POST", f"/api/v1/courses/{...}/sections", ...)` |
| GET /api/v1/courses/:course_uuid/sections | yes | true no-mock HTTP | `test_courses.py`, `test_scope_isolation.py` | `api_request("GET", f"/api/v1/courses/{...}/sections", ...)` |
| PUT /api/v1/courses/sections/:uuid | yes | true no-mock HTTP | `test_courses.py` | `api_request("PUT", f"/api/v1/courses/sections/{...}", ...)` |
| DELETE /api/v1/courses/sections/:uuid | yes | true no-mock HTTP | `test_courses.py` | `api_request("DELETE", f"/api/v1/courses/sections/{...}", ...)` |
| POST /api/v1/courses/sections/:section_uuid/lessons | yes | true no-mock HTTP | `test_courses.py` | `api_request("POST", f"/api/v1/courses/sections/{...}/lessons", ...)` |
| PUT /api/v1/courses/lessons/:uuid | yes | true no-mock HTTP | `test_courses.py` | `api_request("PUT", f"/api/v1/courses/lessons/{...}", ...)` |
| DELETE /api/v1/courses/lessons/:uuid | yes | true no-mock HTTP | `test_courses.py` | `api_request("DELETE", f"/api/v1/courses/lessons/{...}", ...)` |
| POST /api/v1/courses/media/upload | yes | true no-mock HTTP | `test_courses.py` | multipart upload to `/api/v1/courses/media/upload` |
| POST /api/v1/courses/media | yes | true no-mock HTTP | `test_courses.py` | `api_request("POST", "/api/v1/courses/media", ...)` |
| POST /api/v1/courses/media/:uuid/validate | yes | true no-mock HTTP | `test_courses.py` | `api_request("POST", f"/api/v1/courses/media/{...}/validate", ...)` |
| GET /api/v1/courses/:course_uuid/versions | yes | true no-mock HTTP | `test_courses.py`, `test_approvals.py` | `api_request("GET", f"/api/v1/courses/{...}/versions", ...)` |
| POST /api/v1/approvals/:course_uuid/submit | yes | true no-mock HTTP | `test_approvals.py`, `test_courses.py` | `api_request("POST", f"/api/v1/approvals/{...}/submit", ...)` |
| POST /api/v1/approvals/:approval_uuid/review | yes | true no-mock HTTP | `test_approvals.py`, `test_reauth_enforcement.py` | `api_request("POST", f"/api/v1/approvals/{...}/review", ...)` |
| GET /api/v1/approvals/:uuid | yes | true no-mock HTTP | `test_approvals.py`, `test_security_regression.py` | `api_request("GET", f"/api/v1/approvals/{...}", ...)` |
| GET /api/v1/approvals/queue | yes | true no-mock HTTP | `test_approvals.py`, `test_scope_isolation.py` | `api_request("GET", "/api/v1/approvals/queue", ...)` |
| POST /api/v1/approvals/process-scheduled | yes | true no-mock HTTP | `test_process_scheduled.py` | [repo/API_tests/test_process_scheduled.py:79], [repo/API_tests/test_process_scheduled.py:111] |
| POST /api/v1/approvals/:course_uuid/unpublish | yes | true no-mock HTTP | `test_approvals.py` | `api_request("POST", f"/api/v1/approvals/{...}/unpublish", ...)` |
| GET /api/v1/audit | yes | true no-mock HTTP | `test_audit.py`, `test_envelope.py` | `api_get("/api/v1/audit", ...)` |
| POST /api/v1/tags | yes | true no-mock HTTP | `test_tags.py` | [repo/API_tests/test_tags.py:104], [repo/API_tests/test_tags.py:127] |
| GET /api/v1/tags | yes | true no-mock HTTP | `test_tags.py` | [repo/API_tests/test_tags.py:145] |
| GET /api/v1/bookings/resources | yes | true no-mock HTTP | `test_bookings.py`, `test_booking_approval.py` | `api_request("GET", "/api/v1/bookings/resources", ...)` |
| GET /api/v1/bookings/resources/:uuid/availability | yes | true no-mock HTTP | `test_bookings.py` | `api_request("GET", f"/api/v1/bookings/resources/{...}/availability?...", ...)` |
| POST /api/v1/bookings | yes | true no-mock HTTP | `test_bookings.py`, `test_booking_approval.py` | `api_request("POST", "/api/v1/bookings", ...)` |
| POST /api/v1/bookings/:uuid/reschedule | yes | true no-mock HTTP | `test_bookings.py` | `api_request("POST", f"/api/v1/bookings/{...}/reschedule", ...)` |
| POST /api/v1/bookings/:uuid/cancel | yes | true no-mock HTTP | `test_bookings.py`, `test_booking_approval.py` | `api_request("POST", f"/api/v1/bookings/{...}/cancel", ...)` |
| GET /api/v1/bookings/my | yes | true no-mock HTTP | `test_bookings.py` | `api_request("GET", "/api/v1/bookings/my", ...)` |
| GET /api/v1/bookings/breaches | yes | true no-mock HTTP | `test_bookings.py` | `api_request("GET", "/api/v1/bookings/breaches", ...)` |
| POST /api/v1/bookings/:uuid/approve | yes | true no-mock HTTP | `test_booking_approval.py`, `test_reauth_enforcement.py` | `api_request("POST", f"/api/v1/bookings/{...}/approve", ...)` |
| POST /api/v1/bookings/:uuid/reject | yes | true no-mock HTTP | `test_booking_approval.py`, `test_reauth_enforcement.py` | `api_request("POST", f"/api/v1/bookings/{...}/reject", ...)` |
| GET /api/v1/bookings/pending-approvals | yes | true no-mock HTTP | `test_department_booking.py` | `api_request("GET", "/api/v1/bookings/pending-approvals", ...)` |
| GET /api/v1/bookings/:uuid/booker-breaches | yes | true no-mock HTTP | `test_department_booking.py` | `api_request("GET", f"/api/v1/bookings/{...}/booker-breaches", ...)` |
| GET /api/v1/bookings/restrictions | yes | true no-mock HTTP | `test_bookings.py` | `api_request("GET", "/api/v1/bookings/restrictions", ...)` |
| GET /api/v1/risk/rules | yes | true no-mock HTTP | `test_risk.py`, `test_envelope.py` | `api_request("GET", "/api/v1/risk/rules", ...)` |
| GET /api/v1/risk/events | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | `api_request("GET", "/api/v1/risk/events", ...)` |
| PUT /api/v1/risk/events/:uuid | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | `api_request("PUT", f"/api/v1/risk/events/{...}", ...)` |
| POST /api/v1/risk/evaluate | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | `api_request("POST", "/api/v1/risk/evaluate", ...)` |
| POST /api/v1/risk/subscriptions | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | `api_request("POST", "/api/v1/risk/subscriptions", ...)` |
| GET /api/v1/risk/subscriptions | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | `api_request("GET", "/api/v1/risk/subscriptions", ...)` |
| POST /api/v1/risk/postings | yes | true no-mock HTTP | `test_risk.py` | `api_request("POST", "/api/v1/risk/postings", ...)` |
| POST /api/v1/risk/blacklist | yes | true no-mock HTTP | `test_risk.py`, `test_security_regression.py` | `api_request("POST", "/api/v1/risk/blacklist", ...)` |
| POST /api/v1/privacy/requests | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | `api_request("POST", "/api/v1/privacy/requests", ...)` |
| GET /api/v1/privacy/requests | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | `api_request("GET", "/api/v1/privacy/requests", ...)` |
| GET /api/v1/privacy/requests/my | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | `api_request("GET", "/api/v1/privacy/requests/my", ...)` |
| POST /api/v1/privacy/requests/:uuid/review | yes | true no-mock HTTP | `test_privacy.py`, `test_privacy_workflows.py` | `api_request("POST", f"/api/v1/privacy/requests/{...}/review", ...)` |
| POST /api/v1/privacy/sensitive | yes | true no-mock HTTP | `test_privacy.py`, `test_security_regression.py` | `api_request("POST", "/api/v1/privacy/sensitive", ...)` |
| GET /api/v1/privacy/sensitive | yes | true no-mock HTTP | `test_privacy.py`, `test_security_regression.py` | `api_request("GET", "/api/v1/privacy/sensitive", ...)` |
| GET /api/v1/terms | yes | true no-mock HTTP | `test_scope_isolation.py`, `test_security_regression.py` | `api_request("GET", "/api/v1/terms", ...)` |
| GET /api/v1/terms/active | yes | true no-mock HTTP | `test_terms_acceptance.py`, `test_courses.py` | `api_request("GET", "/api/v1/terms/active", ...)` |
| POST /api/v1/terms/:term_uuid/accept | yes | true no-mock HTTP | `test_terms_acceptance.py`, `test_courses.py` | `api_request("POST", f"/api/v1/terms/{...}/accept", ...)` |
| GET /api/v1/terms/my-acceptances | yes | true no-mock HTTP | `test_terms_acceptance.py` | `api_request("GET", "/api/v1/terms/my-acceptances", ...)` |
| GET /api/v1/notifications | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | `api_request("GET", "/api/v1/notifications/", ...)` |
| GET /api/v1/notifications/unread-count | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | `api_request("GET", "/api/v1/notifications/unread-count", ...)` |
| PUT /api/v1/notifications/:uuid/read | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | `api_request("PUT", f"/api/v1/notifications/{...}/read", ...)` |
| PUT /api/v1/notifications/read-all | yes | true no-mock HTTP | `test_notifications.py`, `test_security_regression.py` | `api_request("PUT", "/api/v1/notifications/read-all", ...)` |

## API Test Classification
- **True No-Mock HTTP**: all `repo/API_tests/test_*.py` files issue HTTP requests using `urllib.request` to `API_BASE_URL`.
- **HTTP with Mocking**: none detected.
- **Non-HTTP tests**: `repo/unit_tests/backend/*`, `repo/unit_tests/frontend/*`, and `repo/unit_tests/frontend_rs/*`.

## Mock Detection
- Searched in API and unit test trees for `jest.mock`, `vi.mock`, `sinon.stub`, `patch(`, `MagicMock`, `Mock`, `monkeypatch`, `unittest.mock`.
- Result: **no mocking/stubbing indicators found** (`rg` returned no matches).
- Note: several API tests use `subprocess.run(["docker", "exec", ... "mysql", ...])` for DB state setup/reset; this is external-state setup, not route-path mocking (e.g., [repo/API_tests/test_tags.py:16], [repo/API_tests/test_process_scheduled.py:22]).

## Coverage Summary
- Total endpoints: **66**
- Endpoints with HTTP tests: **66**
- Endpoints with true no-mock HTTP tests: **66**
- HTTP coverage: **100.00%**
- True API coverage: **100.00%**
- Previously missing endpoints are now directly covered:
  - `POST /api/v1/approvals/process-scheduled` ([repo/API_tests/test_process_scheduled.py])
  - `POST /api/v1/tags` ([repo/API_tests/test_tags.py])
  - `GET /api/v1/tags` ([repo/API_tests/test_tags.py])

## Unit Test Summary

### Backend Unit Tests
- Files: `repo/unit_tests/backend/test_*.py` (14 files).
- Covered areas (spec/business-rule tests): auth/password/JWT/permissions, booking rules, risk rules, scheduling logic, API shape contracts.
- Important backend modules not directly unit-tested in Rust runtime:
  - `repo/backend/src/routes/*.rs`
  - `repo/backend/src/repositories/*.rs`
  - `repo/backend/src/middleware/*.rs`
  - `repo/backend/src/services/*.rs`

### Frontend Unit Tests (STRICT)
- Frontend test files found:
  - `repo/unit_tests/frontend/test_route_definitions.py`
  - `repo/unit_tests/frontend_rs/src/lib.rs`
- Framework/tool evidence:
  - Python `unittest` for route/nav harness.
  - Rust `cargo test` crate (`repo/unit_tests/frontend_rs/Cargo.toml`) importing real frontend modules using `#[path]`.
- Direct imports/runs of real frontend source:
  - `#[path = "../../../frontend/src/types/mod.rs"]` ([repo/unit_tests/frontend_rs/src/lib.rs:14])
  - `#[path = "../../../frontend/src/role_nav.rs"]` ([repo/unit_tests/frontend_rs/src/lib.rs:17])
- Components/modules covered (direct):
  - `frontend/src/types/mod.rs`
  - `frontend/src/role_nav.rs`
- Important frontend modules not yet unit-tested directly:
  - `repo/frontend/src/main.rs`
  - `repo/frontend/src/pages/*`
  - `repo/frontend/src/components/*`
  - `repo/frontend/src/api/mod.rs`

**Mandatory verdict: Frontend unit tests: PRESENT**

### Cross-Layer Observation
- Backend API coverage is now comprehensive (66/66).
- Frontend strict unit coverage now exists for real source modules, but still focuses on pure logic/types; UI/component execution coverage remains comparatively thinner.

## API Observability Check
- Strong: most tests expose method/path, request body, and response data assertions.
- Moderate weakness remains in a subset of negative-path tests that primarily assert status code (examples: [repo/API_tests/test_department_booking.py], [repo/API_tests/test_security_regression.py:624]).
- Verdict: **Good observability overall, with some shallow negative-path assertions.**

## Tests Check
- Success/failure coverage: present across auth, RBAC, approvals, bookings, risk, privacy, notifications.
- Edge/validation/auth checks: present broadly.
- Over-mocking: not detected.
- `run_tests.sh` static compliance:
  - Docker-contained execution enforced ([repo/run_tests.sh:5-13], [repo/run_tests.sh:91-97], [repo/run_tests.sh:117-177]).
  - Strict failure handling present: `set -Eeuo pipefail`, `ERR`/`EXIT` traps, diagnostics, stage-wise non-zero failure aggregation ([repo/run_tests.sh:14], [repo/run_tests.sh:45-75], [repo/run_tests.sh:198-201]).

## End-to-End Expectations
- Fullstack FE↔BE browser-journey E2E automation (e.g., Playwright/Cypress) is still absent.
- Given 100% API endpoint coverage + backend/unit layers + strict frontend unit tests, this is a **remaining gap**, but no longer a blocker for strict API coverage/readme gate compliance.

## Test Coverage Score (0–100)
**95/100**

## Score Rationale
- + 100% endpoint HTTP coverage with real route-path calls.
- + No explicit mocking/stubbing in API path.
- + Previously uncovered critical endpoints now tested (tags + process-scheduled HMAC).
- + Frontend strict unit tests now execute real frontend Rust modules.
- - FE↔BE automated browser E2E still missing.
- - Some negative-path observability remains shallow.

## Key Gaps
- No automated browser-level FE↔BE E2E suite.
- Direct unit execution for many frontend UI pages/components is still limited.
- Many backend “unit” tests are behavior mirrors in Python rather than Rust module-level tests.

## Confidence & Assumptions
- Confidence: **high** for endpoint inventory and static HTTP mapping.
- Assumption: request helpers (`api_request`, `api_get`, `api_post`) are used consistently as real HTTP calls to a running backend.
- Static-only limitation: runtime route normalization behavior (e.g., trailing slash handling) is inferred from test intent and helper usage, not executed.

## Test Coverage Verdict
**PASS (strict API coverage achieved; frontend strict-unit requirement present).**

---

# README Audit

## README Location
- Found at required path: `repo/README.md`.

## Hard Gate Results

### Formatting
- PASS: Structured markdown with clear headings, tables, and runnable command blocks.

### Startup Instructions (Fullstack)
- PASS: exact literal `docker-compose up` present ([repo/README.md:43]).

### Access Method
- PASS: URL/port/service mapping present ([repo/README.md:52-60]).

### Verification Method
- PASS: API verification (`curl`) and explicit UI verification flows present ([repo/README.md:77-181]).

### Environment Rules (Docker-contained only)
- PASS: README explicitly states no host-side runtime/toolchain requirements and Docker-contained execution ([repo/README.md:48], [repo/README.md:182-198]).
- PASS: no prohibited install/setup commands (`npm install`, `pip install`, `apt-get`, manual DB setup) were found in README.

### Demo Credentials (Auth exists)
- PASS: usernames, passwords, and role coverage listed ([repo/README.md:67-73]); seed evidence exists ([repo/backend/src/services/seed.rs:22-28]).

## Engineering Quality Findings

### High Priority Issues
- None found that violate hard gates.

### Medium Priority Issues
- README includes both HTTPS proxy path and dev-only direct backend path; this is documented but still introduces dual-path operator choice ([repo/README.md:56], [repo/README.md:81], [repo/README.md:96-105]).

### Low Priority Issues
- Quantitative claims (endpoint counts, file counts) are currently aligned, but these are maintenance-sensitive and can drift without automation.

## Hard Gate Failures
- None.

## README Verdict
**PASS**

---

## Final Combined Verdict
- **Test Coverage Audit:** PASS
- **README Audit:** PASS
