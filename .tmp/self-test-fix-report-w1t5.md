# Self Test Fix Report

Date: 2026-04-05  
Scope: Static code inspection of current `repo/` implementation against issues listed in `.tmp/delivery_acceptance_architecture_audit.md`.

## Method used

- I re-checked the current backend routes/services/repositories/migrations, frontend pages/API client/CSS, README, self-test report, and API test suite.
- This is a **static verification** pass (no runtime execution in this report).

## Overall status summary

- Total audited issues from the prior report: **12** (8 High + 4 Medium)
- **Fully addressed:** 6
- **Partially addressed / mostly addressed:** 5
- **Not fully addressed:** 1

## Issue-by-issue verification matrix

### High Severity

1. **Privacy export/delete/rectify were placeholders**  
   **Status: ✅ Addressed**

Evidence found:

- Real export generation and file write: `backend/src/services/privacy_service.rs`
- Full data export query composition: `backend/src/repositories/privacy_repo.rs` (`export_user_data`)
- Deletion now anonymizes account + cleans related records: `backend/src/repositories/privacy_repo.rs` (`anonymize_user`)
- Rectification applies field-level changes (`email`, `full_name`) with audit trail: `backend/src/services/privacy_service.rs`, `backend/src/repositories/privacy_repo.rs`
- Rectify request fields supported in schema: `backend/migrations/20240112000000_add_rectify_fields.sql`

Conclusion: Prior placeholder logic has been replaced with implemented workflows.

---

2. **Risk subscriptions/webhooks were stored but not triggered by risk events**  
   **Status: ✅ Addressed**

Evidence found:

- Subscription-driven dispatch introduced: `backend/src/services/risk_service.rs` (`dispatch_risk_notifications`)
- Webhook enqueue on event creation path: `backend/src/services/risk_service.rs` + `backend/src/services/webhook_service.rs` (`enqueue_event_webhooks`)
- Queue processing/retry/dead-letter logic exists: `backend/src/services/webhook_service.rs`, `backend/src/repositories/webhook_repo.rs`
- Background job loop runs webhook processor: `backend/src/jobs/mod.rs`, `backend/src/main.rs`

Conclusion: Event-to-subscription delivery path is now wired in code.

---

3. **Booking approval lacked department scope and approver breach visibility**  
   **Status: 🟡 Partially addressed**

Evidence found:

- Department ownership added to resources: `backend/migrations/20240110000000_add_department_to_resources.sql`
- Department-scoped checks on approve/reject: `backend/src/services/booking_service.rs`
- Department-scoped pending queue: `backend/src/services/booking_service.rs`, `backend/src/routes/bookings.rs` (`/pending-approvals`)
- Booker breach endpoint for reviewers added: `backend/src/routes/bookings.rs` (`/<uuid>/booker-breaches`), `backend/src/services/booking_service.rs`

Remaining gap:

- Frontend reviewer table in `frontend/src/pages/bookings.rs` does not surface a “view breaches” action before decision; API exists but UX is incomplete.

Conclusion: Backend scope and breach access are implemented; reviewer UX for breach inspection is still incomplete.

---

4. **Rate limiting/anti-scraping did not protect login/unauthenticated endpoints**  
   **Status: 🟡 Partially addressed**

Evidence found:

- IP-based login throttling added: `backend/src/routes/auth.rs` + `backend/src/repositories/login_rate_limit_repo.rs`
- Account lockout counters added: `backend/migrations/20240111000000_add_login_rate_limits.sql`, lockout logic in auth/login flow tests
- New login throttle tests exist: `API_tests/test_login_throttle.py`

Remaining gap:

- Prior issue wording included “other unauthenticated high-value endpoints”; current explicit unauth throttling is clearly implemented for `/auth/login`, but not generalized for all unauth endpoints.

Conclusion: Critical login hardening is implemented; broad unauth endpoint throttling is only partially covered.

---

5. **Administrative re-authentication was only partially enforced**  
   **Status: ✅ Addressed**

Evidence found:

- Approval review now requires reauth reviewer guard: `backend/src/routes/approvals.rs`
- Booking approve/reject/pending/booker-breaches now require reauth reviewer guard: `backend/src/routes/bookings.rs`
- Audit access now requires reauth admin guard: `backend/src/routes/audit.rs`
- Existing risk/privacy sensitive ops remain reauth-protected: `backend/src/routes/risk.rs`, `backend/src/routes/privacy.rs`
- Reauth enforcement tests added: `API_tests/test_reauth_enforcement.py`

Conclusion: The previously called-out admin/reviewer sensitive operations are now guarded.

---

6. **Frontend omitted critical course release flows (media/tag/unpublish/diff)**  
   **Status: 🟡 Partially addressed**

Evidence found:

- Unpublish request flow now present in UI: `frontend/src/pages/course_detail.rs`
- Version list tab exists: `frontend/src/pages/course_detail.rs`
- Backend media upload/validate endpoints exist: `backend/src/routes/courses.rs`

Remaining gaps:

- Media flow in UI is mostly instructional text, not an actual upload/validate interaction: `frontend/src/pages/course_detail.rs`
- No explicit version-diff visualization flow in frontend (despite backend diff model/repo support): `backend/src/dto/course.rs`, `backend/src/repositories/course_repo.rs`, no matching frontend API/route usage
- Tag UX is incomplete (create tag exists, but no full attach/manage cycle exposed in clear UI workflow)

Conclusion: Improved, but still short of fully implemented end-user course release UX.

---

7. **Frontend omitted booking reviewer flow, conflict UX, and mobile nav was broken**  
   **Status: 🟡 Partially addressed**

Evidence found:

- Reviewer pending approvals tab and actions now implemented: `frontend/src/pages/bookings.rs`
- Mobile navigation fixed with hamburger + overlay sidebar: `frontend/src/layouts/mod.rs`, `frontend/assets/main.css`

Remaining gap:

- Immediate conflict UX via live availability check is not wired; `availability_slots` signal is present but unused: `frontend/src/pages/bookings.rs`

Conclusion: Reviewer workflow and mobile nav are fixed; live conflict pre-check UX is still missing.

---

8. **Department/term isolation weakened by role-wide notification fan-out**  
   **Status: 🟡 Mostly addressed**

Evidence found:

- Department-scoped notifier added: `backend/src/services/notification_service.rs` (`notify_department_role`)
- Approval submission and booking pending notifications now use department-scoped delivery: `backend/src/services/approval_service.rs`, `backend/src/services/booking_service.rs`

Remaining caveat:

- Some role-wide notifications still exist for admin-level escalation/fallback (`notify_role("admin")`) in specific paths.

Conclusion: Core reviewer leakage path is reduced substantially; some role-wide admin broadcasts remain by design.

---

### Medium Severity

9. **Docs/self-test overclaimed runtime verification and had route mismatch notes**  
   **Status: 🟡 Partially addressed**

Evidence found:

- README now documents HMAC requirement for scheduled transitions and includes explicit headers/signing instructions: `README.md`

Remaining issue:

- `SELF_TEST_REPORT.md` still presents broad PASS assertions that imply runtime confidence, and some inventory counts/details are stale compared to current tree.

Conclusion: Documentation alignment improved, but reporting claims are still stronger than strictly static proof.

---

10. **Success response contract was inconsistent (envelope drift)**  
    **Status: ✅ Addressed**

Evidence found:

- `/health`, `/info`, and `/auth/me` now return `ApiResponse` envelope: `backend/src/routes/health.rs`, `backend/src/routes/info.rs`, `backend/src/routes/auth.rs`
- Envelope regression tests added: `API_tests/test_envelope.py`

Conclusion: Prior response-envelope inconsistency has been corrected.

---

11. **HMAC integration support/provisioning incomplete**  
    **Status: ✅ Addressed**

Evidence found:

- HMAC schema exists: `backend/migrations/20240102000000_phase2_schema.sql`
- Dev HMAC key seeded: `backend/src/services/seed.rs`
- Admin key provisioning endpoint added (reauth-protected): `backend/src/routes/auth.rs` (`/auth/hmac-keys`)
- HMAC flow tests added: `API_tests/test_hmac_flow.py`

Conclusion: Provisioning and operational path are now present in-code.

---

12. **Unit tests were mostly Python mirrors; divergence risk from Rust implementation**  
    **Status: ❌ Not fully addressed**

Evidence found:

- API test coverage improved significantly (new auth/reauth/hmac/department/privacy/envelope suites).
- But there is still no Rust-native `backend/tests/*.rs` test suite present.
- Unit tests remain predominantly Python mirror logic.

Conclusion: Coverage improved at API level, but the specific issue of native Rust unit-test parity is still unresolved.

## Additional findings relevant to prior audit gaps

- Background jobs now run scheduled transitions, risk evaluation, and webhook delivery processing in `main` loop: `backend/src/main.rs`, `backend/src/jobs/mod.rs`.
- Booking and approval paths now show stronger department/term scoping than before.
- No API tests in current suite were found targeting HTTPS proxy paths (`https://localhost`), so proxy/TLS verification remains mostly a manual/runtime check.

## Final judgment

The implementation has improved substantially versus the original audit baseline and resolves most of the highest-risk backend control gaps.  
However, it is **not yet fully closed** against all originally reported issues due to:

1. incomplete frontend parity on media/diff/live-availability reviewer UX, and
2. lack of Rust-native backend unit tests to replace/anchor Python mirror tests.

---

## Recommended next fixes (highest impact)

1. Add frontend flows for:
   - real media upload + validation actions,
   - version-diff display between releases,
   - reviewer “view booker breaches” action,
   - live availability/conflict check while composing booking.
2. Add Rust-native backend tests for critical invariants:
   - approval step enforcement and actor separation,
   - privacy export/delete/rectify semantics,
   - webhook enqueue/queue state transitions,
   - booking department scope enforcement.
3. Align `SELF_TEST_REPORT.md` language to clearly separate static verification from runtime execution claims.
