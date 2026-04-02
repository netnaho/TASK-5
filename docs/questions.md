# Required Document Description: Business Logic Questions Log

### 1. Two-step signoff for publish/unpublish workflow
**Question**: Prompt mentions a "two-step signoff" for publishing or unpublishing courses by Department Reviewers, but doesn't detail what these two steps entail or who effectively performs them.

**My Understanding**: To adhere to strict segregation of duties and prevent self-approval, two distinct authorities must review the course: a Department Reviewer for the initial check, and an Admin for the final publishing step.

**Solution**: Implemented a state machine where a submitted course enters a `pending_step_1` state for Reviewers, and then `pending_step_2` for Admins before moving to `published` or `approved_scheduled` (as validated by the `ReviewerGuard` and system tests).

---

### 2. Data sources for Risk Engine evaluations
**Question**: The anomaly and risk engine flags "abnormal compensation entries for adjunct assignments" and "duplicate internship/job postings", yet the primary UI descriptions only cover the learning catalog and facility bookings. Where does this HR data originate if the system is completely offline?

**My Understanding**: Since the system operates offline without external API dependencies, this compliance data must be ingested locally from other on-premise HR/Career systems before the scheduled risk engine can analyze it.

**Solution**: Created dedicated API endpoints (e.g., `POST /api/v1/risk/postings`) to ingest internal compliance data, which the risk engine then processes via its background scheduler to generate Risk Events and trigger local subscriptions/webhooks.

---

### 3. Scope of administrative re-auth within 15 minutes
**Question**: The prompt states "administrative actions require re-auth within 15 minutes", but it is unclear whether this applies generally to any catalog editing an Admin performs, or only to highly sensitive operations.

**My Understanding**: Requiring a password repeatedly for normal catalog workflows would degrade UX. The 15-minute re-auth should strictly apply to security-critical actions such as data export/deletion requests or system configurations.

**Solution**: Introduced a `sudo_mode` concept in the JWT session. Normal API calls use the standard token, while sensitive endpoints require calling `POST /api/v1/auth/reauth` to activate a 15-minute `sudo_mode` authorization window.

---

### 4. Handling of scheduled campus catalog updates
**Question**: The prompt requires "an effective date/time so a catalog update can be scheduled for the next term." What specifically transitions the schedule to live status when the date is reached while the system is offline?

**My Understanding**: The system needs to support an internal queue of approved releases and automatically transition their status without relying on external scheduling services.

**Solution**: Implemented an `approved_scheduled` course state and a `POST /api/v1/approvals/process-scheduled` endpoint (triggered manually by an Admin or a background CRON) to execute pending transitions when the effective date and time arrives.

---

### 5. Booking restriction enforcement and manual overrides
**Question**: The prompt states that 3 cancellation breaches (within 2 hours of start) in 60 days "can trigger temporary booking restrictions." It is unspecified how long the restriction lasts and whether human intervention is involved.

**My Understanding**: The system should automatically apply a standard suspension period to prevent immediate resource abuse, but should also expose these restrictions to the administration for potential overrides.

**Solution**: Modeled a schema tracking breaches and exposed `GET /api/v1/bookings/restrictions`. The background breach engine automatically suspends booking privileges under these conditions, visible via the UI notification center.

---

### 6. 180-day course version retention and media cleanup
**Question**: The prompt enforces "read-only access to prior versions for 180 days." How is large media (up to 500 MB per file) handled when a version expires, especially given offline storage limitations?

**My Understanding**: Orphaned video/PDF files must be securely pruned from the local disk when their associated course version ages out after 180 days, unless a newer version still references the exact same file.

**Solution**: Designed a versioning schema with `expires_at` tracking and a reference-counted media registry that safely purges unreferenced media files to recover disk space when pulling `/versions`.

---

### 7. Cross-department data access restrictions
**Question**: The prompt specifies that data access is "restricted by department and term," but it is ambiguous how this applies to Reviewers versus Students who may take cross-departmental courses.

**My Understanding**: Authors and Reviewers are strictly siloed to their administrative department boundary, whereas Students/Faculty access is based on explicit section enrollments regardless of the owning department.

**Solution**: Implemented layered RBAC using request guards (`CourseAuthorGuard` / `ReviewerGuard`) that enforce a strict `department_id` constraint, whereas general cross-department student queries filter transparently based on active term enrollment graphs.
