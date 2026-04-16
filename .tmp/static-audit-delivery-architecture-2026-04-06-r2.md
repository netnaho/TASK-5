# CampusLearn Operations Suite — Static Delivery Acceptance & Architecture Audit (R2)

Date: 2026-04-06  
Audit mode: **Static-only** (no runtime execution)

## 1. Verdict

- **Overall conclusion: Partial Pass**

Reason: previously reported critical delivery gaps (media upload UI, approval/booking object-scope checks, HMAC docs mismatch, version diff exposure, booking availability feedback) are now statically addressed, but material **High/Medium** issues remain in security posture hardening, strict scope semantics, and test robustness.

---

## 2. Scope and Static Verification Boundary

### Reviewed

- Docs/manifests/config: `README.md`, `docker-compose.yml`, `package.json`, `run_tests.sh`
- Backend entry/security/routes/services: `backend/src/main.rs`, `backend/src/routes/*.rs`, `backend/src/services/*.rs`, `backend/src/middleware/*.rs`
- Frontend flow pages: `frontend/src/pages/course_detail.rs`, `frontend/src/pages/bookings.rs`
- Static tests: `API_tests/*.py`, `unit_tests/backend/*.py`

### Not reviewed / not executed

- No project startup, no Docker startup, no tests executed, no browser rendering checks.
- No live network/system-integration behavior validation.

### Claims requiring manual verification

- End-to-end parallel booking race outcomes under load.
- Production HTTPS/TLS deployment hardening.
- Risk scheduler cadence and retry/queue behavior in runtime conditions.
- Real browser interaction/accessibility polish.

---

## 3. Repository / Requirement Mapping Summary

### Prompt core goal (condensed)

Unified offline portal for:

1. course authoring/versioning + two-step publish/unpublish approvals,
2. resource booking with strict policy/breach controls,
3. risk/anomaly detection + notifications,
   with strong RBAC/scope/privacy/security controls.

### Main implementation areas mapped

- Rocket REST backend + MySQL schema/services/guards (`backend/src/**`)
- Dioxus role-aware UI flows (`frontend/src/pages/**`)
- Security controls: JWT + reauth, HMAC nonce/timestamp checks, audit/security events
- Validation coverage via Python API + unit/spec tests (`API_tests/**`, `unit_tests/**`)

---

## 4. Section-by-section Review

## 4.1 Hard Gates

### 4.1.1 Documentation and static verifiability

- **Conclusion: Pass**
- **Rationale:** startup/test and HMAC signed-call instructions are now statically aligned with implementation.
- **Evidence:** test commands `README.md:207-214`; HMAC canonical format/hex digest `README.md:153-169`; guard message construction `backend/src/middleware/hmac_guard.rs:90`.

### 4.1.2 Material deviation from prompt

- **Conclusion: Partial Pass**
- **Rationale:** major previously-missing UI flows are now implemented, but strict “department + term” data-scope semantics remain partially ambiguous for non-scoped roles.
- **Evidence:** media upload in UI `frontend/src/pages/course_detail.rs:179-194`; availability feedback `frontend/src/pages/bookings.rs:278,313`; faculty/student/default published-term path `backend/src/services/course_service.rs:56,158-162`.

## 4.2 Delivery Completeness

### 4.2.1 Coverage of explicit core requirements

- **Conclusion: Partial Pass**
- **Rationale:** core catalog/approval/booking/risk flows are broadly present and several prior gaps are closed; some policy hardening and verification rigor gaps remain.
- **Evidence:** two-step review route + reauth guard `backend/src/routes/approvals.rs:39-47`; review scope enforcement `backend/src/services/approval_service.rs:153-170`; booking breach scope enforcement `backend/src/services/booking_service.rs:491-503`; version diff API/UI `backend/src/routes/courses.rs:280-289`, `frontend/src/pages/course_detail.rs:282`.

### 4.2.2 End-to-end 0→1 deliverable

- **Conclusion: Partial Pass**
- **Rationale:** product-shaped implementation is present, but test strategy still includes mirror-spec tests and environment-coupled operations.
- **Evidence:** mirror-spec disclaimer `unit_tests/backend/test_booking_rules.py:1-2`; mirrored diff logic `unit_tests/backend/test_version_diff.py:6`; Docker-coupled reset helper in API tests `API_tests/test_risk.py:33-40`, `API_tests/test_hmac_flow.py:33-40`.

## 4.3 Engineering and Architecture Quality

### 4.3.1 Structure/module decomposition

- **Conclusion: Pass**
- **Rationale:** backend and frontend remain cleanly layered by responsibility.
- **Evidence:** route mounts and domain modules `backend/src/main.rs:106-129`; route-level module separation (`backend/src/routes/*.rs`).

### 4.3.2 Maintainability/extensibility

- **Conclusion: Partial Pass**
- **Rationale:** additive improvements (risk-rule update endpoint, diff endpoint, scoped checks) improve extensibility; some policy semantics remain mixed by role.
- **Evidence:** additive risk-rule update route `backend/src/routes/risk.rs:126-142`; version diff route `backend/src/routes/courses.rs:280-300`; role-branch visibility logic `backend/src/services/course_service.rs:38-56,158-162`.

## 4.4 Engineering Details and Professionalism

### 4.4.1 Error handling/logging/validation/API shape

- **Conclusion: Partial Pass**
- **Rationale:** structured error handling/logging and guard usage exist; however, key-management secret handling semantics remain weak.
- **Evidence:** request logging fairing `backend/src/main.rs:106-114`; centralized error logging `backend/src/utils/errors.rs:74`; HMAC key insert stores provided secret into `secret_hash` directly `backend/src/routes/auth.rs:133-136`; key retrieval uses same column for verification `backend/src/middleware/hmac_guard.rs:76`.

### 4.4.2 Product-grade vs demo-level

- **Conclusion: Partial Pass**
- **Rationale:** overall product behavior is substantial, but test reliability posture still allows severe defects to evade detection.
- **Evidence:** object-scope tests accept permissive outcomes (`200` or `403`) `API_tests/test_scope_isolation.py:322-326,384`; mirror tests not executing Rust code `unit_tests/backend/test_booking_rules.py:1-2`.

## 4.5 Prompt Understanding and Requirement Fit

### 4.5.1 Business goal/constraints fit

- **Conclusion: Partial Pass**
- **Rationale:** business domains are implemented and many prompt-critical fixes are in place; strict data-scope constraint and deployment hardening are not fully closed.
- **Evidence:** scoped approval review `backend/src/services/approval_service.rs:153-170`; immediate booking feedback UI `frontend/src/pages/bookings.rs:328-333`; default scoped secrets still in compose `docker-compose.yml:36,50`.

## 4.6 Aesthetics (frontend-only/full-stack)

### 4.6.1 Visual/interaction design quality

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** structure and interaction code exist, but visual quality and rendering correctness require runtime browser verification.
- **Evidence:** booking availability panel UI code `frontend/src/pages/bookings.rs:328-333`; media/diff interaction elements `frontend/src/pages/course_detail.rs:176-214,265-301`.
- **Manual verification:** required.

---

## 5. Issues / Suggestions (Severity-Rated)

### High

1. **HMAC key secret persisted in plaintext-like path despite `secret_hash` naming**

- **Conclusion:** Fail
- **Evidence:** direct insert of request secret into `secret_hash` column `backend/src/routes/auth.rs:133-136`; retrieval and signature verification based on same value `backend/src/middleware/hmac_guard.rs:76,90`.
- **Impact:** key-handling semantics are misleading and increase risk if DB is exposed or operators assume hashed-at-rest handling.
- **Minimum actionable fix:** store encrypted/peppered secret material with explicit naming (`secret_encrypted`) or hash-only + separate verifier design; update schema naming and docs for accurate semantics.

2. **Strict department+term data-scope requirement is still only partially enforced by role model**

- **Conclusion:** Partial Fail
- **Evidence:** scoped enforcement for author/reviewer `backend/src/services/course_service.rs:38-55`; default role branch allows published content by term without department predicate `backend/src/services/course_service.rs:56,158-162`.
- **Impact:** potential mismatch with prompt-level “data access restricted by department and term” interpretation.
- **Minimum actionable fix:** formally codify role-by-role scope policy and enforce department filter wherever required (or explicitly document approved exceptions in acceptance docs).

### Medium

3. **Security-sensitive fallback secrets remain in compose defaults**

- **Conclusion:** Partial Fail
- **Evidence:** default JWT secret fallback `docker-compose.yml:36`; default encryption key fallback `docker-compose.yml:50`; seeded known credentials remain static `backend/src/services/seed.rs:23-25`.
- **Impact:** accidental insecure deployment posture if dev defaults leak into shared/non-dev environments.
- **Minimum actionable fix:** require env-provided secrets for non-dev, gate seed credentials under explicit dev-only guard, and fail-fast when production mode lacks secure secrets.

4. **Object-scope API tests remain permissive and can mask regressions**

- **Conclusion:** Partial Fail
- **Evidence:** cross-dept approval review test allows `200` or `403` `API_tests/test_scope_isolation.py:322-326`; booker-breaches scope test allows `200` or `403` `API_tests/test_scope_isolation.py:373-384`.
- **Impact:** severe authorization regressions may still pass CI depending on fixture topology.
- **Minimum actionable fix:** build deterministic fixtures with guaranteed cross-dept separation and assert strict forbidden outcomes for out-of-scope reviewer calls.

5. **API tests include Docker-coupled mutation helpers**

- **Conclusion:** Partial Fail
- **Evidence:** direct `docker exec` in tests `API_tests/test_risk.py:33-40`, `API_tests/test_hmac_flow.py:33-40`.
- **Impact:** lower portability and brittle CI behavior outside expected local container naming/layout.
- **Minimum actionable fix:** replace shell-level DB mutation helpers with API/admin test fixtures or repository-level reset hooks.

6. **Risk rule update endpoint lacks dedicated API test coverage**

- **Conclusion:** Partial Fail
- **Evidence:** backend route exists `backend/src/routes/risk.rs:129-142`; current risk tests cover list/evaluate/events/blacklist/subscriptions, but no `PUT /risk/rules/<uuid>` case `API_tests/test_risk.py:56-220`.
- **Impact:** newly-added configurability can regress unnoticed.
- **Minimum actionable fix:** add admin reauth success/403 failure tests for `PUT /api/v1/risk/rules/<uuid>` plus payload validation checks.

---

## 6. Security Review Summary

- **Authentication entry points:** **Pass**  
  Evidence: login/reauth/me and guarded admin HMAC-key provisioning `backend/src/routes/auth.rs:16-95,125-147`.

- **Route-level authorization:** **Pass**  
  Evidence: reviewer/admin guards and reauth guards on sensitive routes (`backend/src/routes/approvals.rs:43`, `backend/src/routes/bookings.rs:100,112,134`, `backend/src/routes/audit.rs:15`, `backend/src/routes/risk.rs:39-40,131-132`).

- **Object-level authorization:** **Partial Pass**  
  Evidence: explicit dept+term check in approval review `backend/src/services/approval_service.rs:153-170`; dept-scope check in booker-breaches `backend/src/services/booking_service.rs:491-503`. Remaining uncertainty from permissive tests and role policy ambiguity.

- **Function-level authorization:** **Pass**  
  Evidence: self-approval prevention `backend/src/services/approval_service.rs:175-184`; reauth-required sensitive operations (`backend/src/routes/approvals.rs:43`, `backend/src/routes/bookings.rs:100,112,134`, `backend/src/routes/risk.rs:40,53,112,132`).

- **Tenant/user data isolation:** **Partial Pass**  
  Evidence: scoped checks for reviewer/author paths `backend/src/services/course_service.rs:38-55`; generic published-in-term branch without dept predicate `backend/src/services/course_service.rs:56,158-162`.

- **Admin/internal/debug protection:** **Pass**  
  Evidence: audit admin+reauth enforcement `backend/src/routes/audit.rs:15`; HMAC anti-replay nonce/timestamp verification `backend/src/middleware/hmac_guard.rs:55-70`.

---

## 7. Tests and Logging Review

- **Unit tests:** **Partial Pass**  
  Evidence: broad Python unit suite exists (`unit_tests/backend/**`), but some core tests are mirror/spec style (`unit_tests/backend/test_booking_rules.py:1-2`, `unit_tests/backend/test_version_diff.py:6`).

- **API / integration tests:** **Partial Pass**  
  Evidence: broad suites for auth/approvals/bookings/risk/privacy (`API_tests/test_auth.py`, `API_tests/test_approvals.py`, `API_tests/test_bookings.py`, `API_tests/test_risk.py`), but docker-coupled resets remain `API_tests/test_risk.py:33-40`, `API_tests/test_hmac_flow.py:33-40`.

- **Logging categories / observability:** **Pass**  
  Evidence: request-level logs `backend/src/main.rs:106-114`; audit/security repositories in services `backend/src/services/risk_service.rs:5,213,329`; error logging `backend/src/utils/errors.rs:74`.

- **Sensitive-data leakage risk in logs / responses:** **Partial Pass**  
  Positive: masked-field response path `backend/src/services/privacy_service.rs:164-172`; webhook signing secret omitted in tests expectation `API_tests/test_risk.py:156-158`.  
  Concern: predictable seeded creds remain in code `backend/src/services/seed.rs:23-25`.

---

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- Unit tests exist: **Yes** (`unit_tests/backend/*.py`, `unit_tests/frontend/*.py`)
- API/integration tests exist: **Yes** (`API_tests/test_*.py`)
- Framework: Python `unittest`
- Entry points: `run_tests.sh` (`run_tests.sh:19,62`)
- Documentation for test commands: **Yes** (`README.md:207-214`)

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                      | Mapped Test Case(s)                                                                                                   | Key Assertion / Fixture / Mock                         | Coverage Assessment | Gap                                                             | Minimum Test Addition                                                      |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------- | --------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Auth 401/403 and reauth gating                | `API_tests/test_auth.py:112-114,150-158`                                                                              | missing token -> 401; no reauth -> 403                 | sufficient          | —                                                               | —                                                                          |
| Two-step approval flow + scheduling semantics | `API_tests/test_approvals.py:90-91,109-139,283-289`                                                                   | release/effective date + step approvals                | basically covered   | deterministic cross-dept denial not strict                      | enforce fixture-separated dept + assert strict 403                         |
| Approval object-scope regression              | `API_tests/test_scope_isolation.py:270-326`                                                                           | cross-dept review attempt accepts 200/403              | insufficient        | permissive assertion can hide authz regressions                 | assert exact 403 for out-of-scope reviewer                                 |
| Booking availability + conflict checks        | `API_tests/test_bookings.py:84-88,107-120`                                                                            | availability endpoint + conflict status 400/409        | basically covered   | UI integration not API-tested                                   | add frontend interaction tests or API-contract + UI state tests            |
| Booker-breaches scope control                 | `API_tests/test_scope_isolation.py:341-384`                                                                           | reviewer breach call accepts 200/403                   | insufficient        | non-deterministic scope assertion                               | construct forced cross-dept fixture and assert 403                         |
| HMAC management guarding                      | `API_tests/test_hmac_flow.py:45-103`                                                                                  | admin+reauth required; non-admin blocked               | basically covered   | no canonical signing-string conformance test                    | add signed process-scheduled request tests for exact canonicalization      |
| Risk engine core + webhook constraints        | `API_tests/test_risk.py:56-169`                                                                                       | admin-only rules, evaluate, on-prem webhook validation | basically covered   | no `PUT /risk/rules/<uuid>` coverage                            | add update-rule success/failure/validation tests                           |
| Privacy masking workflows                     | `API_tests/test_privacy_workflows.py` (suite), service masking path `backend/src/services/privacy_service.rs:164-172` | masked response expectations                           | basically covered   | encrypted-at-rest integrity not statically proven by tests here | add DB-integrity assertion tests for ciphertext/iv/key_version consistency |
| Version diff visibility                       | backend route `backend/src/routes/courses.rs:280-289`; UI fetch `frontend/src/pages/course_detail.rs:282`             | implementation present                                 | cannot confirm      | no dedicated API/UI test located in reviewed files              | add API + UI test for diff retrieval/render                                |

### 8.3 Security Coverage Audit

- **Authentication:** Basically covered (good 401/403 and reauth tests).
- **Route authorization:** Basically covered for major admin/reviewer routes.
- **Object-level authorization:** Insufficiently covered due permissive assertions (`200` or `403`) in key cross-dept tests.
- **Tenant/data isolation:** Partially covered; course visibility policy semantics still not tightly asserted against strict prompt interpretation.
- **Admin/internal protection:** Basically covered for reauth/admin flows; HMAC process-route canonical signed-request behavior not comprehensively tested.

### 8.4 Final Coverage Judgment

- **Final Coverage Judgment: Partial Pass**

Major auth and role-guard paths are tested, but object-scope and strict-scope policy regressions could still pass current tests due permissive assertions and missing dedicated coverage for newly added risk-rule update and version-diff flows.

---

## 9. Final Notes

- This report is strictly static; no runtime behavior is claimed.
- Compared with earlier audit state, several prior blockers are now resolved in code.
- Remaining priority should focus on: (1) key-secret handling semantics, (2) strict scope policy clarity/enforcement, (3) deterministic authorization tests, and (4) reducing Docker-coupled test setup patterns.
