# CampusLearn Operations Suite — Static Delivery Acceptance & Architecture Audit

Date: 2026-04-06  
Audit mode: **Static-only** (no runtime execution)

## 1. Verdict

- **Overall conclusion: Fail**

Reason: multiple **Blocker/High** gaps against prompt-critical requirements (core UI flow completeness, authorization/data-scope boundaries, and documentation-to-implementation consistency).

---

## 2. Scope and Static Verification Boundary

### Reviewed

- Project docs/manifests/config: `README.md`, `docker-compose.yml`, `backend/Cargo.toml`, `frontend/Cargo.toml`, `package.json`, `run_tests.sh`
- Backend entry/architecture/security/business logic: `backend/src/main.rs`, routes/services/repositories/middleware/auth modules, migrations
- Frontend route/page/API structure: `frontend/src/main.rs`, `frontend/src/layouts/mod.rs`, `frontend/src/pages/*.rs`, `frontend/src/api/mod.rs`, `frontend/assets/main.css`
- Test suites (static): `API_tests/*.py`, `unit_tests/backend/*.py`, `unit_tests/frontend/*.py`

### Not reviewed / not executed

- No project startup, no Docker, no tests executed, no browser interactions, no network calls.
- No dynamic verification of concurrency races, TLS cert behavior, scheduler timing, or webhook delivery.

### Claims requiring manual verification

- True end-to-end behavior under parallel booking requests
- TLS termination and strict HTTPS deployment posture in real target environment
- Runtime scheduler cadence and queue retry behavior
- UI runtime behavior/accessibility/visual polish on real browsers/devices

---

## 3. Repository / Requirement Mapping Summary

### Prompt core goal (condensed)

Unified offline portal for:

1. course authoring/versioning + two-step publish/unpublish approvals,
2. shared resource booking with strict policy constraints/breaches/restrictions,
3. risk/anomaly detection + notifications,  
   with strict security/RBAC/scope controls and privacy workflows.

### Main implementation areas mapped

- Backend: Rocket REST API + MySQL schema/migrations + auth/guards + services (`backend/src/**`, `backend/migrations/**`)
- Frontend: Dioxus role-aware pages (`frontend/src/pages/**`, `frontend/src/layouts/mod.rs`)
- Security/compliance controls: JWT, reauth, HMAC nonce checks, audit/security events, encryption vault
- Tests: Python API integration and Python unit-spec mirrors

---

## 4. Section-by-section Review

## 4.1 Hard Gates

### 4.1.1 Documentation and static verifiability

- **Conclusion: Partial Pass**
- **Rationale:** README and structure are substantial, but there is a critical HMAC doc/impl mismatch that can block reproducible static verification of a core scheduled endpoint.
- **Evidence:**
  - HMAC example uses newline payload + base64 signature: `README.md:155-156`
  - Runtime implementation expects `key_id:nonce:timestamp:method:path` and hex signature: `backend/src/auth/hmac.rs:11,25`
  - Guard verifies against that implementation: `backend/src/middleware/hmac_guard.rs:90`
- **Manual verification:** required to confirm endpoint interop using corrected signing instructions.

### 4.1.2 Material deviation from prompt

- **Conclusion: Fail**
- **Rationale:** key prompt flows are partially replaced with non-UI/manual API guidance, and some security/scope constraints are weakened.
- **Evidence:**
  - Media upload UX is instruction text, not actual local upload flow in UI: `frontend/src/pages/course_detail.rs:170-172`
  - Prompt requires department+term data restrictions, but scoped viewing allows broader published visibility without dept check: `backend/src/services/course_service.rs:45,51,53`

---

## 4.2 Delivery Completeness

### 4.2.1 Coverage of explicit core requirements

- **Conclusion: Partial Pass**
- **Rationale:** many requirements are implemented (approvals, booking constraints, risk engine, privacy), but notable misses exist.
- **Evidence (implemented examples):**
  - Two-step approval workflow + self-approval prevention: `backend/src/services/approval_service.rs:56-66,150-163`
  - Effective date parsing in required format: `backend/src/services/approval_service.rs:357-362`
  - Booking constraints (90-day, max duration, active cap, hours): `backend/src/services/booking_service.rs:92-130`
  - Concurrency-safe booking via transaction + row lock: `backend/src/repositories/booking_repo.rs:21-63`
  - Risk scheduling fields default 15 min: `backend/migrations/20240103000000_phase3_schema.sql:50-51`
- **Evidence (gaps):**
  - UI does not implement local media upload interaction: `frontend/src/pages/course_detail.rs:170-172`
  - Immediate conflict feedback in UI not wired (availability state declared but unused): `frontend/src/pages/bookings.rs:14`; no UI call to availability API: `frontend/src/**/*.rs` search (no `api::check_availability`)
  - “Clearly show what changed since last approved release” not surfaced; diff retrieval function exists but is unused: `backend/src/repositories/course_repo.rs:259` (only match for `get_diff(`)

### 4.2.2 End-to-end 0→1 deliverable (vs partial/demo)

- **Conclusion: Partial Pass**
- **Rationale:** repository is product-shaped and broad, but several “unit tests” are spec mirrors and API tests depend on docker-side DB manipulation, reducing confidence in true end-to-end validation discipline.
- **Evidence:**
  - Unit tests explicitly mirror logic instead of executing Rust production code: `unit_tests/backend/test_booking_rules.py:1-2`, `unit_tests/backend/test_version_diff.py:6`
  - API tests invoke `docker exec` resets in test code: e.g. `API_tests/test_auth.py:14`, `API_tests/test_booking_approval.py:14`

---

## 4.3 Engineering and Architecture Quality

### 4.3.1 Structure/module decomposition

- **Conclusion: Pass**
- **Rationale:** clear modular decomposition by routes/services/repositories/middleware, frontend pages/components/types, and migration layering.
- **Evidence:** `backend/src/main.rs:3-10`, mounted route modules `backend/src/main.rs:111-123`, folder structure in README `README.md:276-313`.

### 4.3.2 Maintainability/extensibility

- **Conclusion: Partial Pass**
- **Rationale:** architecture is generally extensible, but some policy/scope checks are inconsistently centralized (review paths differ), causing maintainability/security risk.
- **Evidence:**
  - Approval read path has dept/term scope logic: `backend/src/services/approval_service.rs:269-288`
  - Approval review path lacks equivalent dept/term object-scope guard: `backend/src/services/approval_service.rs:145-173`

---

## 4.4 Engineering Details and Professionalism

### 4.4.1 Error handling/logging/validation/API shape

- **Conclusion: Partial Pass**
- **Rationale:** consistent error envelope and validation patterns exist; however, security-sensitive defaults and secret handling are weak for a delivery acceptance target.
- **Evidence:**
  - Validation usage examples: `backend/src/routes/auth.rs:23-31`, `backend/src/routes/approvals.rs:24-30`
  - Structured request logging fairing: `backend/src/main.rs:100-109`
  - HMAC secret persisted in plaintext field despite `secret_hash` naming: insert path `backend/src/routes/auth.rs:133`; read path `backend/src/middleware/hmac_guard.rs:76`

### 4.4.2 Product-grade vs demo-level

- **Conclusion: Partial Pass**
- **Rationale:** broad feature set and real schema suggest product intent, but key user workflows still rely on manual API instructions in UI and several test files are spec-style mirrors.
- **Evidence:** media tab guidance-only UI `frontend/src/pages/course_detail.rs:170-172`; mirror-unit tests `unit_tests/backend/test_permissions.py:1`, `unit_tests/backend/test_booking_rules.py:1-2`.

---

## 4.5 Prompt Understanding and Requirement Fit

### 4.5.1 Business goal/constraints fit

- **Conclusion: Partial Pass**
- **Rationale:** major domains are present, but fit is weakened by scope-control mismatches and missing direct UI workflows.
- **Evidence:**
  - Domain coverage routes: `backend/src/main.rs:111-123`
  - Department/term scope weakness in course visibility logic: `backend/src/services/course_service.rs:45,51,53`
  - Booking conflict feedback API exists but no UI integration: `frontend/src/api/mod.rs:143-145`; no call from UI (`frontend/src/**/*.rs`, no `api::check_availability`)

---

## 4.6 Aesthetics (frontend-only/full-stack)

### 4.6.1 Visual/interaction design quality

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** CSS system and structured pages exist, but visual quality, render correctness, and interaction feel require runtime browser verification.
- **Evidence:** design system `frontend/assets/main.css:1-300`; routed pages `frontend/src/main.rs:54-82`.
- **Manual verification:** required.

---

## 5. Issues / Suggestions (Severity-Rated)

### Blocker

1. **Missing in-UI local media upload workflow (core prompt flow not delivered)**

- **Conclusion:** Fail
- **Evidence:** `frontend/src/pages/course_detail.rs:170-172`
- **Impact:** Staff Author cannot complete required “locally uploaded media” flow in the Dioxus portal; current UI instructs users to call APIs manually.
- **Minimum actionable fix:** Implement file picker + upload action to `POST /api/v1/courses/media/upload` and validation action in UI with status feedback.

### High

2. **Approval review endpoint lacks department/term object-scope authorization**

- **Conclusion:** Fail
- **Evidence:** review checks only role/self-approval (`backend/src/services/approval_service.rs:150-173`), while read path has dept/term checks (`backend/src/services/approval_service.rs:269-288`)
- **Impact:** A reviewer with a valid role may potentially review out-of-scope approvals by UUID.
- **Minimum actionable fix:** Reuse same dept/term scope check used in `get_approval` before allowing `review_approval` decisions.

3. **Booking “booker breaches” endpoint leaks cross-scope user breach records by booking UUID**

- **Conclusion:** Fail
- **Evidence:** route accepts reviewer only (`backend/src/routes/bookings.rs:132-137`), service fetches booking then returns all breaches of `booking.booked_by` without department scope check (`backend/src/services/booking_service.rs:482-485`)
- **Impact:** Reviewer may access breach history outside their department scope.
- **Minimum actionable fix:** Add reviewer department/resource scope validation (similar to approve/reject checks) before returning breaches.

4. **Prompt-required data scope restriction (department + term) is only partially enforced**

- **Conclusion:** Fail
- **Evidence:**
  - Staff author visibility allows non-draft in-term regardless of department: `backend/src/services/course_service.rs:45`
  - Reviewer visibility includes any published in-term (not only dept): `backend/src/services/course_service.rs:51`
  - Default branch for non-admin roles only checks published+term: `backend/src/services/course_service.rs:53`
- **Impact:** Data isolation semantics deviate from prompt; potential cross-department exposure.
- **Minimum actionable fix:** enforce explicit department filter for all non-admin reads where prompt requires it; document any intentional exceptions.

5. **Critical doc-to-code mismatch for HMAC scheduled endpoint**

- **Conclusion:** Fail
- **Evidence:** README uses newline/base64 signing `README.md:155-156`; implementation uses colon-delimited message and hex digest `backend/src/auth/hmac.rs:11,25`, validated in guard `backend/src/middleware/hmac_guard.rs:90`
- **Impact:** Operators following docs may fail to invoke scheduled processing; verifiability gate is weakened.
- **Minimum actionable fix:** update README with exact message canonicalization and hex signature format used by backend.

6. **Security-sensitive defaults and secret exposure in delivery config**

- **Conclusion:** Partial Fail
- **Evidence:** exposed backend HTTP port `docker-compose.yml:58`; hardcoded JWT/dev encryption key `docker-compose.yml:34,48`; dev origin on HTTP `docker-compose.yml:40`; hardcoded seeded credentials `backend/src/services/seed.rs:23-27`
- **Impact:** Elevated risk of insecure deployment carryover and credential leakage.
- **Minimum actionable fix:** move secrets to secure env injection, disable host-exposed backend in default compose profile, and gate seed credentials behind explicit dev flag.

7. **Immediate booking conflict feedback is not implemented in UI flow**

- **Conclusion:** Fail (requirement gap)
- **Evidence:** availability state declared but unused `frontend/src/pages/bookings.rs:14`; no frontend call to availability API (`frontend/src/**/*.rs`, no `api::check_availability`)
- **Impact:** user only discovers conflicts after submit; deviates from prompt “immediate conflict feedback”.
- **Minimum actionable fix:** call availability endpoint on resource/date changes and render slot/conflict feedback pre-submit.

### Medium

8. **Risk thresholds not configurable via API/UI despite prompt requiring configurable thresholds**

- **Conclusion:** Partial Fail
- **Evidence:** risk route set excludes any rule update endpoint `backend/src/routes/risk.rs:127`; only list/evaluate/event update/subscription/posting/blacklist exposed.
- **Impact:** operational tuning requires DB-level/manual changes.
- **Minimum actionable fix:** add authenticated admin endpoint + UI for rule threshold/schedule condition updates with audit logging.

9. **Version diff generated but not consumable through API/UI**

- **Conclusion:** Partial Fail
- **Evidence:** diff persisted (`backend/src/services/version_service.rs:24-31`), retrieval exists only in repository (`backend/src/repositories/course_repo.rs:259`, single usage match), UI versions table shows summary only (`frontend/src/pages/course_detail.rs:187`)
- **Impact:** weak fulfillment of “clearly show what changed since last approved release.”
- **Minimum actionable fix:** add diff endpoint and visualize structured diffs in version tab.

10. **Test architecture has heavy environment coupling and many mirror-spec unit tests**

- **Conclusion:** Partial Fail
- **Evidence:** `docker exec` state resets in many API test modules (e.g., `API_tests/test_auth.py:14`, `API_tests/test_security_regression.py:37`); backend “unit tests” are Python mirrors (`unit_tests/backend/test_booking_rules.py:1-2`).
- **Impact:** lower trust in direct regression protection of production Rust code and reduced portability.
- **Minimum actionable fix:** add Rust-native unit/integration tests for critical guards and service rules; reduce Docker-shell coupling in Python tests.

11. **Unrelated external dependency appears in root manifest**

- **Conclusion:** Suspected Risk / Partial Fail
- **Evidence:** `package.json:3` (`firebase-admin`)
- **Impact:** potential prompt misalignment with offline/no-external-dependency constraint; at minimum introduces ambiguity and review noise.
- **Minimum actionable fix:** remove if unused, or document exact offline-safe purpose.

---

## 6. Security Review Summary

- **Authentication entry points:** **Pass**  
  Evidence: JWT login/me/reauth flows and guards (`backend/src/routes/auth.rs:16-95`, `backend/src/middleware/auth_guard.rs:14-52`, `backend/src/middleware/reauth_guard.rs:17-50`)

- **Route-level authorization:** **Partial Pass**  
  Evidence: role guards broadly applied (`backend/src/routes/*.rs`), but some sensitive reads use admin-only without reauth (risk list/events: `backend/src/routes/risk.rs:16-28`).

- **Object-level authorization:** **Fail**  
  Evidence: approval review lacks dept/term object check (`backend/src/services/approval_service.rs:150-173`), booker breaches lacks dept scope (`backend/src/services/booking_service.rs:482-485`).

- **Function-level authorization:** **Partial Pass**  
  Evidence: strong checks for booking approve/reject dept scope (`backend/src/services/booking_service.rs:262-272,340-350`), self-approval prevention (`backend/src/services/approval_service.rs:154-163`). Gaps noted above.

- **Tenant/user data isolation:** **Partial Pass**  
  Evidence: per-user notification and privacy retrieval endpoints exist (`backend/src/routes/notifications.rs:10-67`, `backend/src/routes/privacy.rs:83-92`). Course visibility logic weakens strict department isolation requirement (`backend/src/services/course_service.rs:45,51,53`).

- **Admin/internal/debug protection:** **Partial Pass**  
  Evidence: audit endpoint requires reauth+admin (`backend/src/routes/audit.rs:13-23`); HMAC-protected scheduled endpoint (`backend/src/routes/approvals.rs:79-86`, `backend/src/middleware/hmac_guard.rs:17-105`). However, backend direct port exposed in compose (`docker-compose.yml:58`).

---

## 7. Tests and Logging Review

- **Unit tests:** **Partial Pass**  
  Exist and numerous, but many are Python mirrors of Rust logic, not direct execution of Rust code paths.
  - Evidence: `unit_tests/backend/test_booking_rules.py:1-2`, `unit_tests/backend/test_version_diff.py:6`

- **API/integration tests:** **Partial Pass**  
  Broad suite exists, includes auth, approvals, bookings, privacy, risk, security regression. But strong docker-coupling and DB mutation via shell commands reduces portability and confidence.
  - Evidence: test file inventory under `API_tests/`; docker exec pattern e.g. `API_tests/test_auth.py:14`, `API_tests/test_booking_approval.py:14`

- **Logging categories / observability:** **Pass**  
  Structured request logs and explicit security/audit logging present.
  - Evidence: `backend/src/main.rs:100-109`, `backend/src/repositories/audit_repo.rs:4-23`, `backend/src/repositories/security_repo.rs:4-17`

- **Sensitive-data leakage risk in logs/responses:** **Partial Pass**  
  Positive: privacy masked response (`backend/src/services/privacy_service.rs:191-198`), signing_secret not returned (`backend/src/dto/risk.rs:54`).  
  Concern: default seeded credentials documented and present in seed config (`README.md:91-99`, `backend/src/services/seed.rs:23-27`).

---

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- **Unit tests exist:** yes (`unit_tests/backend/*.py`, `unit_tests/frontend/test_route_definitions.py`)
- **API/integration tests exist:** yes (`API_tests/test_*.py`)
- **Frameworks:** Python `unittest` (`run_tests.sh:17`, `run_tests.sh:53`)
- **Entry points:** `./run_tests.sh`, `python3 -m unittest discover ...` (`run_tests.sh:17,53`)
- **Docs for test commands:** yes (`README.md:215-229`)

### 8.2 Coverage Mapping Table

| Requirement / Risk Point           | Mapped Test Case(s)                                                                    | Key Assertion / Fixture / Mock          | Coverage Assessment | Gap                                                                                        | Minimum Test Addition                                                       |
| ---------------------------------- | -------------------------------------------------------------------------------------- | --------------------------------------- | ------------------- | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| Auth 401/invalid token             | `API_tests/test_auth.py:113-121`                                                       | `/auth/me` invalid/missing token => 401 | sufficient          | —                                                                                          | —                                                                           |
| Reauth enforcement                 | `API_tests/test_auth.py:153-176`, `API_tests/test_reauth_enforcement.py:80-97`         | sensitive actions return 403 pre-reauth | basically covered   | not all admin reads checked                                                                | add tests for admin read endpoints requiring/omitting reauth per policy     |
| Two-step approvals + self-approval | `API_tests/test_approvals.py:108-127`, `API_tests/test_security_regression.py:374-382` | review flow + self-approval forbidden   | basically covered   | cross-dept review-by-UUID not tested                                                       | add negative test: reviewer from other dept attempts review                 |
| Booking rule constraints           | `API_tests/test_bookings.py`, `unit_tests/backend/test_booking_rules.py:54-164`        | 90-day, duration, conflicts, caps       | basically covered   | no true concurrency race test in API suite                                                 | add parallel booking requests test targeting same slot                      |
| Booking dept approval scope        | `API_tests/test_department_booking.py:99-105`                                          | non-reviewer blocked                    | insufficient        | lacks cross-dept reviewer negative                                                         | add reviewer-other-dept approve/reject + booker-breaches negatives          |
| HMAC replay/timestamp flow         | `API_tests/test_hmac_flow.py`, `API_tests/test_envelope.py`                            | header/auth checks exercised            | basically covered   | docs mismatch not tested                                                                   | add doc-example conformance test for signing canonicalization               |
| Privacy workflows                  | `API_tests/test_privacy_workflows.py:36-162`                                           | export/delete/rectify paths             | basically covered   | encrypted-at-rest decryption integrity not tested end-to-end                               | add DB-level assertion test for ciphertext+iv format and masked retrieval   |
| Notification isolation             | `API_tests/test_security_regression.py:411+`                                           | cross-user read/mark boundaries         | sufficient          | —                                                                                          | —                                                                           |
| Department+term strict data scope  | `API_tests/test_scope_isolation.py:95-220`                                             | some role visibility checks             | insufficient        | current tests do not catch wider published-course exposure semantics and review-scope gaps | add explicit cross-dept published-course visibility policy tests per prompt |
| Version diff visibility in UI/API  | `unit_tests/backend/test_version_diff.py`                                              | diff generation only                    | missing             | no endpoint/UI verification                                                                | add API diff retrieval tests + UI rendering tests                           |

### 8.3 Security Coverage Audit

- **Authentication:** **Sufficiently covered** (login/me/invalid token tests present).
- **Route authorization:** **Basically covered** (many 401/403 checks), but not exhaustive against policy-level reauth expectations.
- **Object-level authorization:** **Insufficient** (no strong negatives for cross-dept approval review / booker-breaches).
- **Tenant/data isolation:** **Insufficient** for strict prompt interpretation; tests do not fully enforce department-boundary semantics across all role paths.
- **Admin/internal protection:** **Basically covered** for key write endpoints; deployment-level exposure (direct backend port) is not a test concern but remains a risk.

### 8.4 Final Coverage Judgment

- **Final Coverage Judgment: Partial Pass**

Major security/auth flows are tested, but uncovered object-scope and strict department-scope risks mean severe defects could still pass current suites.

---

## 9. Final Notes

- This audit is strictly static and evidence-based; no runtime claims are made.
- Primary remediation priority should be:
  1. close object-level authorization gaps,
  2. deliver missing core UI flows (media upload + immediate availability feedback),
  3. align docs with HMAC implementation,
  4. harden delivery defaults/secrets.
