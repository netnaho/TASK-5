# Delivery Acceptance and Project Architecture Audit

## 1. Verdict
- Overall conclusion: Fail

## 2. Scope and Static Verification Boundary
- What was reviewed: `README.md`, `SELF_TEST_REPORT.md`, compose/proxy config, backend entry points, route registration, auth/security middleware, core services/repositories/models/migrations, frontend routes/pages/API client/CSS, and the `unit_tests/` plus `API_tests/` suites.
- What was not reviewed: runtime behavior, browser rendering, database state after execution, Docker orchestration success, TLS behavior in a live environment, background jobs actually running, webhook delivery reachability, and real test execution.
- What was intentionally not executed: the project, Docker, tests, builds, and external services.
- Claims requiring manual verification: actual TLS/proxy behavior, live booking concurrency under parallel requests, background-job cadence, webhook delivery/retry behavior, and real UI rendering across devices.

## 3. Repository / Requirement Mapping Summary
- Prompt core goal: one offline portal covering course authoring/versioned publishing, two-step approvals, shared-resource booking, compliance/risk alerts, local auth/RBAC, sensitive-data handling, and offline-only integrations.
- Main implementation areas mapped: Rocket backend (`backend/src/main.rs:21`, `backend/src/routes/mod.rs:1`), Dioxus frontend (`frontend/src/main.rs:24`), MySQL schema/migrations (`backend/migrations/20240101000000_initial_schema.sql:8`), auth/security middleware (`backend/src/middleware/auth_guard.rs:15`, `backend/src/middleware/reauth_guard.rs:17`, `backend/src/middleware/hmac_guard.rs:14`), and test suites (`unit_tests/backend/test_password.py:1`, `API_tests/test_auth.py:1`).
- Headline fit: the repository is a real full-stack codebase with relevant modules, but several prompt-critical behaviors are incomplete, weakly enforced, or only partially surfaced in the UI.

## 4. Section-by-section Review

### 1.1 Documentation and static verifiability
- Conclusion: Partial Pass
- Rationale: Startup/test/config instructions and entry points exist, but documentation materially overclaims runtime proof and contains static inconsistencies. The README says admins can trigger scheduled publishing, but the route actually requires HMAC verification; the repo defines HMAC tables but no HMAC key management or seed path, so the integration path is not statically verifiable end-to-end.
- Evidence: `README.md:38`, `README.md:145`, `README.md:180`, `backend/src/routes/approvals.rs:77`, `backend/src/middleware/hmac_guard.rs:22`, `backend/migrations/20240102000000_phase2_schema.sql:14`, `backend/src/routes/mod.rs:1`, `SELF_TEST_REPORT.md:107`
- Manual verification note: actual startup and HMAC-key provisioning would require running the stack or manually populating DB state, which was intentionally not done.

### 1.2 Whether the delivered project materially deviates from the Prompt
- Conclusion: Fail
- Rationale: The implementation is broadly aligned with the business domain, but it materially misses prompt-critical user flows: privacy export/deletion/rectification are placeholders, risk subscriptions/webhooks are not wired to event generation, booking approval visibility/scope is incomplete, and the frontend omits major required flows such as media handling, booking reviewer workflows, unpublish UI, and change-diff presentation.
- Evidence: `backend/src/services/privacy_service.rs:39`, `backend/src/services/risk_service.rs:92`, `backend/src/services/webhook_service.rs:103`, `backend/src/routes/bookings.rs:96`, `frontend/src/pages/course_editor.rs:117`, `frontend/src/pages/bookings.rs:109`, `frontend/src/pages/course_detail.rs:127`

### 2.1 Whether the delivered project fully covers the core requirements explicitly stated in the Prompt
- Conclusion: Fail
- Rationale: Core requirements are only partially covered. Course version diffs are generated but not exposed as a user flow; booking approvers cannot view breaches and bookings are not department-scoped; privacy export/delete/rectify are not fully implemented; webhook subscriptions are stored but not delivered; and the frontend does not provide several required operational flows.
- Evidence: `backend/src/services/version_service.rs:24`, `backend/src/repositories/course_repo.rs:259`, `backend/src/routes/courses.rs:251`, `backend/src/routes/bookings.rs:86`, `backend/src/services/privacy_service.rs:41`, `backend/src/services/privacy_service.rs:45`, `backend/src/services/privacy_service.rs:48`, `backend/src/services/risk_service.rs:186`, `frontend/src/api/mod.rs:106`, `frontend/src/pages/bookings.rs:120`

### 2.2 Whether the delivered project represents a basic end-to-end deliverable from 0 to 1
- Conclusion: Partial Pass
- Rationale: This is not a snippet-only repository; it has a complete project skeleton, docs, migrations, backend, frontend, and tests. However, several flows stop at placeholder persistence or partially wired UI, so some advertised end-to-end scenarios are not statically complete.
- Evidence: `README.md:255`, `backend/src/main.rs:96`, `frontend/src/main.rs:24`, `backend/src/services/privacy_service.rs:41`, `backend/src/services/webhook_service.rs:103`, `frontend/src/pages/privacy.rs:80`

### 3.1 Whether the project adopts a reasonable engineering structure and module decomposition
- Conclusion: Pass
- Rationale: The backend is clearly layered by routes/services/repositories/models, and the frontend is split into pages/components/layout/API/types. Core concerns are not piled into a single file.
- Evidence: `backend/src/main.rs:3`, `backend/src/routes/mod.rs:1`, `backend/src/services/mod.rs:1`, `backend/src/repositories/mod.rs:1`, `frontend/src/main.rs:1`, `frontend/src/layouts/mod.rs:6`

### 3.2 Whether the project shows maintainability and extensibility
- Conclusion: Partial Pass
- Rationale: The module decomposition is maintainable, but several features are only half-wired, and the test suite relies heavily on Python mirrors of Rust logic rather than executing production code. That weakens change safety and makes future extension riskier.
- Evidence: `unit_tests/backend/test_password.py:6`, `unit_tests/backend/test_version_diff.py:6`, `unit_tests/backend/test_scheduled_publish.py:6`, `backend/src/services/approval_service.rs:360`

### 4.1 Whether engineering details reflect professional software practice
- Conclusion: Fail
- Rationale: There is solid structure around tracing, audit logs, JWTs, and parameterized SQL, but important professional controls are incomplete: rate limiting only applies after authentication, re-auth is not enforced for all admin actions, booking approval lacks object scope, the API success envelope is inconsistent, and privacy input validation is incomplete on the sensitive-data endpoint.
- Evidence: `backend/src/middleware/auth_guard.rs:39`, `backend/src/routes/auth.rs:16`, `API_tests/test_rate_limit.py:117`, `backend/src/routes/approvals.rs:38`, `backend/src/routes/bookings.rs:96`, `backend/src/routes/audit.rs:12`, `backend/src/utils/response.rs:5`, `backend/src/routes/auth.rs:91`, `backend/src/routes/privacy.rs:64`, `backend/src/dto/privacy.rs:29`

### 4.2 Whether the project is organized like a real product or service
- Conclusion: Partial Pass
- Rationale: The repo looks like a real product in structure, but some flows remain at the level of illustrative persistence instead of full product behavior, especially in privacy export/delete/rectify and risk-subscription delivery.
- Evidence: `backend/src/services/privacy_service.rs:39`, `backend/src/services/privacy_service.rs:48`, `backend/src/services/risk_service.rs:186`, `backend/src/services/webhook_service.rs:131`

### 5.1 Whether the project accurately understands and responds to the business goal and constraints
- Conclusion: Fail
- Rationale: The codebase understands the domain, but key prompt semantics are weakened or ignored: two-step approvals can be bypassed by admin role semantics, department/term scoping does not cover booking approvals, privacy workflows are not materially complete, and the responsive console loses navigation on mobile.
- Evidence: `backend/src/middleware/auth_guard.rs:77`, `backend/src/services/approval_service.rs:169`, `backend/src/repositories/booking_repo.rs:145`, `backend/migrations/20240101000000_initial_schema.sql:237`, `backend/src/services/privacy_service.rs:39`, `frontend/assets/main.css:192`, `frontend/src/layouts/mod.rs:45`

### 6.1 Whether the visual and interaction design fits the scenario and demonstrates reasonable visual quality
- Conclusion: Partial Pass
- Rationale: The frontend has a coherent design system, responsive CSS, tabs, tables, badges, toasts, modals, and status styling. However, responsive behavior is materially broken because the sidebar is hidden on mobile with no replacement navigation, and several required interaction flows are absent from the UI.
- Evidence: `frontend/assets/main.css:2`, `frontend/assets/main.css:63`, `frontend/assets/main.css:93`, `frontend/assets/main.css:191`, `frontend/src/layouts/mod.rs:45`, `frontend/src/pages/bookings.rs:109`, `frontend/src/pages/course_editor.rs:117`
- Manual verification note: visual polish, spacing, and render correctness still require human browser review.

## 5. Issues / Suggestions (Severity-Rated)

### High
- Title: Privacy export/delete/rectify workflows are placeholder implementations
  - Conclusion: Fail
  - Evidence: `backend/src/services/privacy_service.rs:39`, `backend/src/services/privacy_service.rs:45`, `backend/src/services/privacy_service.rs:48`, `backend/src/repositories/privacy_repo.rs:79`
  - Impact: Export only records a synthetic file path, delete only removes `sensitive_data_vault` rows, and rectify immediately completes with no data change. The delivered privacy workflow does not materially satisfy offline export/deletion with administrator approval.
  - Minimum actionable fix: Implement real export generation, define and execute full-account deletion/retention rules, and add a real rectification workflow with field-level targets and auditability.

- Title: Risk subscriptions and webhook delivery are stored but never triggered by risk events
  - Conclusion: Fail
  - Evidence: `backend/src/services/risk_service.rs:92`, `backend/src/services/risk_service.rs:115`, `backend/src/services/risk_service.rs:186`, `backend/src/services/webhook_service.rs:103`
  - Impact: Users can create subscriptions, but new risk events only notify hard-coded roles; the optional webhook feature is not functionally integrated, so queued-local delivery on unreachable endpoints is not actually available.
  - Minimum actionable fix: On event creation, resolve matching subscriptions, enqueue webhook deliveries, and deliver in-app notifications according to subscription settings instead of hard-coded role fan-out.

- Title: Booking approval lacks department-scoped authorization and approver breach visibility
  - Conclusion: Fail
  - Evidence: `backend/migrations/20240101000000_initial_schema.sql:237`, `backend/src/routes/bookings.rs:96`, `backend/src/services/booking_service.rs:327`, `backend/src/routes/bookings.rs:86`, `backend/src/services/booking_service.rs:406`
  - Impact: The booking domain has no department field to scope approvals, any `ReviewerGuard` user can approve/reject pending bookings, and approvers cannot inspect breach history even though the prompt requires breach visibility for approvers.
  - Minimum actionable fix: Add department/ownership scope to resources or bookings, enforce it in approve/reject service logic, and add reviewer/admin breach views tied to booking decisions.

- Title: Rate limiting and anti-scraping do not protect login or other unauthenticated entry points
  - Conclusion: Fail
  - Evidence: `backend/src/middleware/auth_guard.rs:39`, `backend/src/routes/auth.rs:16`, `API_tests/test_rate_limit.py:117`
  - Impact: Brute-force login attempts and unauthenticated scraping are outside the implemented limiter, despite the prompt requiring rate limiting and basic anti-scraping controls.
  - Minimum actionable fix: Add IP and account-aware throttling for `/auth/login` and other unauthenticated high-value endpoints, and document/test those controls.

- Title: Administrative re-authentication is only partially enforced
  - Conclusion: Fail
  - Evidence: `backend/src/routes/risk.rs:40`, `backend/src/routes/privacy.rs:51`, `backend/src/routes/auth.rs:42`, `backend/src/routes/approvals.rs:38`, `backend/src/routes/bookings.rs:96`, `backend/src/routes/audit.rs:12`
  - Impact: Some admin actions require recent re-auth, but others with clear administrative impact do not, including course approval decisions, booking approval/rejection, and audit access. This does not match the prompt's 15-minute re-auth requirement for administrative actions.
  - Minimum actionable fix: Define the admin-action set explicitly and apply `ReauthRequired` consistently to every qualifying route.

- Title: Frontend omits multiple prompt-critical course release flows
  - Conclusion: Fail
  - Evidence: `backend/src/routes/courses.rs:190`, `backend/src/routes/approvals.rs:87`, `frontend/src/api/mod.rs:106`, `frontend/src/pages/course_editor.rs:117`, `frontend/src/pages/course_detail.rs:127`
  - Impact: The backend exposes media and unpublish functionality, but the Dioxus UI only supports course details, sections, lessons, and submit-for-approval. There is no UI for media upload/validation, no tag-management flow, no unpublish flow, and no clear diff presentation between approved releases.
  - Minimum actionable fix: Add frontend media/tag/unpublish/version-diff flows and surface read-only prior-version content in the course detail/version UI.

- Title: Frontend omits booking reviewer workflow and immediate conflict UX, and mobile navigation is broken
  - Conclusion: Fail
  - Evidence: `frontend/src/pages/bookings.rs:109`, `frontend/src/api/mod.rs:139`, `frontend/assets/main.css:192`, `frontend/src/layouts/mod.rs:45`
  - Impact: The booking page only supports the current user's bookings/resources/breaches, never calls availability for immediate feedback, has no reviewer approval queue, and hides the only navigation on mobile. This falls short of the prompt's responsive console requirement.
  - Minimum actionable fix: Add a reviewer booking-approval view, wire live availability/conflict checks into booking entry, and provide a mobile navigation alternative when the sidebar is hidden.

- Title: Department/term data isolation is weakened by role-wide notification fan-out
  - Conclusion: Fail
  - Evidence: `backend/src/services/notification_service.rs:18`, `backend/src/repositories/user_repo.rs:80`, `backend/src/services/approval_service.rs:76`, `backend/src/services/booking_service.rs:161`
  - Impact: Course and booking notifications are broadcast to every user in a role, not to department-scoped reviewers. This leaks request metadata across departments and conflicts with the prompt's data-scope requirement.
  - Minimum actionable fix: Replace role-wide fan-out with department/term-scoped recipient resolution tied to the relevant entity.

### Medium
- Title: Documentation and self-test reporting overstate what is statically verifiable and contain route mismatches
  - Conclusion: Partial Pass
  - Evidence: `README.md:145`, `backend/src/routes/approvals.rs:77`, `README.md:189`, `API_tests/test_auth.py:7`, `SELF_TEST_REPORT.md:107`
  - Impact: Reviewers are told that admins can trigger scheduled transitions and that the project has already been proven at runtime, but the route protection and static audit boundary do not support those claims. This reduces trust in the delivery package.
  - Minimum actionable fix: Align README and self-test claims with actual route guards, verification paths, and what is proven statically versus only manually.

- Title: Success response contract is inconsistent with the documented API envelope
  - Conclusion: Partial Pass
  - Evidence: `README.md:27`, `backend/src/utils/response.rs:5`, `backend/src/routes/auth.rs:91`, `backend/src/routes/info.rs:13`, `backend/src/routes/health.rs:11`
  - Impact: `message: null` is omitted from standard success responses, and `/auth/me`, `/info`, and `/health` bypass the common response type entirely. Client behavior and docs can drift.
  - Minimum actionable fix: Standardize all success endpoints on one documented envelope, or explicitly document exceptions.

- Title: HMAC integration support is incomplete from a delivery-verification standpoint
  - Conclusion: Partial Pass
  - Evidence: `backend/migrations/20240102000000_phase2_schema.sql:14`, `backend/src/middleware/hmac_guard.rs:74`, `backend/src/routes/mod.rs:1`, `backend/src/services/seed.rs:101`
  - Impact: The repo contains HMAC verification logic and schema, but no route or seed path provisions usable HMAC keys, leaving the integration path incomplete for a static reviewer.
  - Minimum actionable fix: Add documented key provisioning/rotation management and static instructions or migration seeds for safe local verification.

- Title: Unit tests are mostly non-executing mirrors and sometimes diverge from the Rust implementation
  - Conclusion: Fail
  - Evidence: `unit_tests/backend/test_password.py:6`, `unit_tests/backend/test_version_diff.py:6`, `unit_tests/backend/test_scheduled_publish.py:8`, `backend/src/services/approval_service.rs:360`, `unit_tests/backend/test_api_response_shape.py:8`
  - Impact: These tests can pass while the real Rust code behaves differently. The scheduled-publish unit test already documents formats that the Rust service does not accept.
  - Minimum actionable fix: Add Rust unit tests or higher-value black-box API tests for critical rules, and remove or clearly label mirror tests that are not executable verification of production code.

## 6. Security Review Summary

- Authentication entry points: Partial Pass
  - Evidence: `backend/src/routes/auth.rs:16`, `backend/src/auth/jwt.rs:18`, `backend/src/middleware/auth_guard.rs:22`
  - Reasoning: JWT-based auth and `/auth/me` guards are present, but login is not rate-limited and HMAC integration provisioning is incomplete.

- Route-level authorization: Partial Pass
  - Evidence: `backend/src/routes/courses.rs:17`, `backend/src/routes/approvals.rs:38`, `backend/src/routes/risk.rs:15`, `backend/src/routes/audit.rs:12`
  - Reasoning: Most routes use explicit guards, but re-auth is not applied consistently and some sensitive actions rely only on broad role guards.

- Object-level authorization: Fail
  - Evidence: `backend/src/services/course_service.rs:57`, `backend/src/services/booking_service.rs:327`, `backend/src/services/approval_service.rs:145`
  - Reasoning: Course ownership checks are present, but booking approval/rejection lacks entity scope checks, and approval review does not enforce department/term scope at the mutation point.

- Function-level authorization: Partial Pass
  - Evidence: `backend/src/services/approval_service.rs:153`, `backend/src/middleware/auth_guard.rs:77`, `backend/src/services/booking_service.rs:257`
  - Reasoning: Self-approval prevention and ownership checks exist, but admin-bypass semantics and missing re-auth weaken higher-risk functions.

- Tenant / user isolation: Partial Pass
  - Evidence: `backend/src/services/course_service.rs:27`, `backend/src/services/approval_service.rs:282`, `backend/src/services/notification_service.rs:18`, `backend/src/routes/bookings.rs:86`
  - Reasoning: Course and approval reads are partly scoped by department/term, but notifications are faned out by role only, and booking approvals/breach visibility do not implement comparable scoping.

- Admin / internal / debug protection: Partial Pass
  - Evidence: `backend/src/routes/risk.rs:18`, `backend/src/routes/audit.rs:15`, `backend/src/routes/approvals.rs:80`, `docker-compose.override.yml:1`
  - Reasoning: Admin-only and HMAC-protected routes exist, but the internal scheduled-transition trigger is not statically operable end-to-end, and dev HTTP exposure remains documented in the delivery.

## 7. Tests and Logging Review

- Unit tests: Fail
  - Evidence: `unit_tests/backend/test_password.py:6`, `unit_tests/backend/test_scheduled_publish.py:8`, `unit_tests/backend/test_api_response_shape.py:8`
  - Reasoning: The unit suite largely reimplements rules in Python instead of executing Rust code; some mirrors already diverge from the backend.

- API / integration tests: Partial Pass
  - Evidence: `API_tests/test_auth.py:52`, `API_tests/test_courses.py:34`, `API_tests/test_approvals.py:34`, `API_tests/test_bookings.py:33`, `API_tests/test_security_regression.py:81`
  - Reasoning: Coverage breadth is good on paper, but several critical areas are not covered: HMAC/process-scheduled, webhook queueing/delivery, booking approval scope, full privacy export/delete semantics, and HTTPS/proxy behavior.

- Logging categories / observability: Partial Pass
  - Evidence: `backend/src/main.rs:23`, `backend/src/repositories/audit_repo.rs:4`, `backend/src/repositories/security_repo.rs:4`, `backend/src/middleware/correlation.rs:1`
  - Reasoning: Structured tracing, audit logs, security events, and correlation IDs are present. This is a strength of the delivery.

- Sensitive-data leakage risk in logs / responses: Partial Pass
  - Evidence: `backend/src/models/user.rs:10`, `backend/src/services/auth_service.rs:29`, `backend/src/repositories/audit_repo.rs:14`, `backend/src/routes/notifications.rs:22`
  - Reasoning: Password hashes are not serialized and I found no direct plaintext SSN logging, but broad notification fan-out leaks operational metadata, and static analysis cannot fully prove runtime log hygiene.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist under `unit_tests/` and use Python `unittest`: `run_tests.sh:19`, `unit_tests/backend/test_password.py:1`, `unit_tests/frontend/test_route_definitions.py:1`.
- API / integration tests exist under `API_tests/` and also use Python `unittest`: `run_tests.sh:59`, `API_tests/test_auth.py:1`, `API_tests/test_security_regression.py:1`.
- Documentation provides test commands: `README.md:180`, `CLAUDE.md:13`, `run_tests.sh:16`.
- Important boundary: API tests default to direct HTTP backend access, not the HTTPS proxy path highlighted elsewhere: `README.md:94`, `README.md:191`, `API_tests/test_auth.py:7`.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| JWT login and authenticated `/me` | `API_tests/test_auth.py:53`, `API_tests/test_auth.py:84` | Successful token retrieval and `401` on missing/invalid token: `API_tests/test_auth.py:55`, `API_tests/test_auth.py:92` | Basically covered | Does not cover login throttling or proxy/TLS path | Add auth tests for login throttling, HTTPS proxy path, and token expiration behavior |
| Password complexity | `unit_tests/backend/test_password.py:22` | Pure-Python mirror function: `unit_tests/backend/test_password.py:6` | Insufficient | Does not execute Rust validator; could drift silently | Add Rust unit tests for `validate_password_complexity` and API tests for change-password validation errors |
| Course ownership and draft visibility | `API_tests/test_courses.py:126`, `API_tests/test_courses.py:168`, `API_tests/test_scope_isolation.py:116` | Cross-author `403` and student draft denial: `API_tests/test_courses.py:222`, `API_tests/test_courses.py:291` | Basically covered | No test for version-diff exposure because no route exists | Add tests for version-diff endpoint once implemented and author/reviewer term-scope edge cases |
| Two-step approval happy path and rejection | `API_tests/test_approvals.py:34`, `API_tests/test_approvals.py:123`, `API_tests/test_approvals.py:254` | Step sequencing and self-approval block: `API_tests/test_approvals.py:68`, `API_tests/test_approvals.py:73` | Basically covered | No test that admin re-auth is required for approval decisions; no test for same-admin multi-step bypass | Add approval tests for re-auth enforcement and per-step actor separation |
| Booking creation, conflicts, reschedule, cancel | `API_tests/test_bookings.py:33`, `API_tests/test_booking_approval.py:33`, `unit_tests/backend/test_booking_rules.py:77` | Conflict and ownership assertions: `API_tests/test_bookings.py:75`, `API_tests/test_bookings.py:208` | Basically covered | No coverage for reviewer scope, approver breach visibility, or live availability UX | Add API tests for department-scoped booking approval and reviewer breach inspection |
| Re-auth enforcement on admin actions | `API_tests/test_auth.py:119`, `API_tests/test_privacy.py:133`, `API_tests/test_risk.py:165`, `API_tests/test_security_regression.py:14` | `403` before `/auth/reauth`, success after: `API_tests/test_privacy.py:145`, `API_tests/test_risk.py:175` | Insufficient | Approvals, booking approvals, and audit access are not tested for re-auth because the code does not enforce it | Add tests for every admin mutation/read classified as sensitive |
| Risk subscriptions and webhook support | `API_tests/test_risk.py:99`, `unit_tests/backend/test_webhook_endpoint_validation.py:217` | Only creation/validation paths are asserted: `API_tests/test_risk.py:115`, `unit_tests/backend/test_webhook_endpoint_validation.py:220` | Missing | No test that a risk event enqueues webhook deliveries or honors subscriptions | Add API/black-box tests that create subscriptions, generate events, and verify queue rows / notification delivery |
| Privacy export/delete/rectify behavior | `API_tests/test_privacy.py:32`, `API_tests/test_privacy.py:76` | Only status `200` on request/review: `API_tests/test_privacy.py:46`, `API_tests/test_privacy.py:100` | Insufficient | No assertion that export content exists, deletion removes all personal data, or rectification changes data | Add tests for export artifact contents, complete deletion semantics, and rectification target updates |
| HMAC anti-replay and scheduled transitions endpoint | None covering live route | N/A | Missing | No tests for `X-HMAC-*` validation, nonce replay, or `/approvals/process-scheduled` | Add API tests for valid signature, replay rejection, expired timestamps, and scheduled transition execution |
| API response envelope consistency | `unit_tests/backend/test_api_response_shape.py:5` | Static dictionary assertions only: `unit_tests/backend/test_api_response_shape.py:8` | Insufficient | Does not hit live handlers; real routes already diverge from the documented envelope | Add API tests that assert actual JSON envelopes for representative success and error routes |
| HTTPS / proxy verification path | None | N/A | Missing | API tests use `http://localhost:8000` by default | Add proxy-path tests against `https://localhost` for health, auth, and a protected route |

### 8.3 Security Coverage Audit
- Authentication: Basically covered by `API_tests/test_auth.py:52`, but severe login-bruteforce defects could still remain undetected because login throttling is not tested and not implemented.
- Route authorization: Partially covered by `API_tests/test_courses.py:78`, `API_tests/test_risk.py:74`, and `API_tests/test_audit.py:56`, but several sensitive admin endpoints are untested for re-auth because they are not guarded.
- Object-level authorization: Partially covered for course ownership in `API_tests/test_courses.py:168`, but severe defects can still remain in booking approval scope because no test exercises department-scoped reviewer boundaries.
- Tenant / data isolation: Partially covered for course/approval reads in `API_tests/test_scope_isolation.py:75` and `API_tests/test_security_regression.py:287`, but notification fan-out and booking reviewer scope are effectively untested.
- Admin / internal protection: Insufficient. There are tests for some admin-only routes, but none for HMAC-protected internal operations or end-to-end integration-key handling.

### 8.4 Final Coverage Judgment
- Fail
- Major risks covered: basic auth happy path, course ownership checks, draft visibility, two-step approval happy path, booking conflict basics, and some re-auth checks.
- Major uncovered risks: login throttling, HMAC/internal endpoints, webhook delivery, full privacy export/delete/rectify semantics, booking reviewer scope, department-scoped notification leakage, HTTPS/proxy behavior, and actual Rust unit-level verification.
- Bottom line: the tests could still pass while severe authorization and delivery defects remain.

## 9. Final Notes
- This audit stayed within the static-only boundary and does not claim runtime success.
- The repository has credible architecture and relevant domain modules, but the current delivery is not acceptable against the prompt because multiple high-severity gaps remain in security boundaries, privacy workflows, subscription/webhook delivery, and prompt-critical UI coverage.
