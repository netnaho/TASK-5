# CampusLearn Operations Suite API Specification

## Base URLs
- **Health Check**: `/`
- **Application Info**: `/api/v1/info`
- **API Base Path**: `/api/v1`

## Authentication & Authorization
The API uses JWT-based authentication. Most endpoints require an `Authorization: Bearer <token>` header.
Roles mentioned in endpoint protections:
- **AuthenticatedUser**: Any valid logged-in user.
- **CourseAuthorGuard**: Users with the `Author` or `Admin` role.
- **ReviewerGuard**: Users with the `Reviewer` or `Admin` role.
- **AdminGuard**: Users with the `Admin` role only.

Note: Sensitive operations may prompt for `sudo_mode` re-authentication within a 15-minute window.

---

## 1. Authentication (`/api/v1/auth`)

### `POST /login`
- **Visibility**: Public
- **Request Body**: `LoginRequest` (username, password)
- **Response**: `ApiResponse<LoginResponse>` (token, user details)
- **Description**: Authenticates a user and returns a JWT token.

### `POST /change-password`
- **Visibility**: AuthenticatedUser
- **Request Body**: `ChangePasswordRequest` (current_password, new_password)
- **Response**: `ApiResponse<String>`
- **Description**: Changes the password of the active user.

### `POST /reauth`
- **Visibility**: AuthenticatedUser
- **Request Body**: `ReauthRequest` (password)
- **Response**: `ApiResponse<String>`
- **Description**: Re-authenticates for sensitive administrative actions (enables `sudo_mode`).

### `GET /me`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Value>` (user profile data)
- **Description**: Retrieves current logged-in user details.

---

## 2. Courses Catalog (`/api/v1/courses`)

### `GET /`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<CourseResponse>>`
- **Description**: Lists courses within the user's role/term scope.

### `POST /`
- **Visibility**: CourseAuthorGuard
- **Request Body**: `CreateCourseRequest`
- **Response**: `ApiResponse<{ uuid: String }>`

### `GET /<uuid>`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<CourseResponse>`

### `PUT /<uuid>`
- **Visibility**: CourseAuthorGuard
- **Request Body**: `UpdateCourseRequest`
- **Response**: `ApiResponse<String>`

### `DELETE /<uuid>`
- **Visibility**: CourseAuthorGuard
- **Response**: `ApiResponse<String>`

### Sections & Lessons
- `GET /<course_uuid>/sections` (AuthenticatedUser) -> `ApiResponse<Vec<SectionResponse>>`
- `POST /<course_uuid>/sections` (CourseAuthorGuard, `CreateSectionRequest`) -> `ApiResponse<{ uuid: String }>`
- `PUT /sections/<uuid>` (CourseAuthorGuard, `UpdateSectionRequest`) -> `ApiResponse<String>`
- `DELETE /sections/<uuid>` (CourseAuthorGuard) -> `ApiResponse<String>`
- `POST /sections/<section_uuid>/lessons` (CourseAuthorGuard, `CreateLessonRequest`) -> `ApiResponse<{ uuid: String }>`
- `PUT /lessons/<uuid>` (CourseAuthorGuard, `UpdateLessonRequest`) -> `ApiResponse<String>`
- `DELETE /lessons/<uuid>` (CourseAuthorGuard) -> `ApiResponse<String>`

### Media
- `POST /media` (CourseAuthorGuard, `CreateMediaRequest`) -> `ApiResponse<MediaResponse>`

### Versions
- `GET /<course_uuid>/versions` (AuthenticatedUser) -> `ApiResponse<Vec<VersionResponse>>`

---

## 3. Approvals (`/api/v1/approvals`)

### `POST /<course_uuid>/submit`
- **Visibility**: CourseAuthorGuard
- **Request Body**: `SubmitApprovalRequest`
- **Response**: `ApiResponse<{ approval_uuid: String }>`

### `POST /<approval_uuid>/review`
- **Visibility**: ReviewerGuard
- **Request Body**: `ReviewApprovalRequest` (approved, notes)
- **Response**: `ApiResponse<String>`

### `GET /<uuid>`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<ApprovalResponse>`

### `GET /queue`
- **Visibility**: ReviewerGuard
- **Response**: `ApiResponse<Vec<ApprovalQueueItem>>`

### `POST /process-scheduled`
- **Visibility**: AdminGuard
- **Response**: `ApiResponse<{ transitions_processed: i64 }>`

---

## 4. Tags (`/api/v1/tags`)

- `GET /` (AuthenticatedUser) -> `ApiResponse<Vec<TagResponse>>`
- `POST /` (CourseAuthorGuard, `CreateTagRequest`) -> `ApiResponse<TagResponse>`

---

## 5. Bookings & Shared Resources (`/api/v1/bookings`)

### `GET /resources`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<ResourceResponse>>`

### `GET /resources/<uuid>/availability?<date>`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<AvailabilitySlot>>`

### `POST /`
- **Visibility**: AuthenticatedUser
- **Request Body**: `CreateBookingRequest`
- **Response**: `ApiResponse<BookingResponse>`

### `POST /<uuid>/reschedule`
- **Visibility**: AuthenticatedUser
- **Request Body**: `RescheduleRequest`
- **Response**: `ApiResponse<String>`

### `POST /<uuid>/cancel`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<String>`

### `GET /my`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<BookingResponse>>`

### `GET /breaches`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<BreachResponse>>`

### `GET /restrictions`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<RestrictionResponse>>`

---

## 6. Audit & Logging (`/api/v1/audit`)

### `GET /?<entity_type>&<entity_id>&<limit>`
- **Visibility**: AdminGuard
- **Response**: `ApiResponse<Vec<AuditLog>>`
- **Description**: Retrieves immutable operation audit logs (7-year retention).

---

## 7. Risk & Anomaly Engine (`/api/v1/risk`)

### `GET /rules`
- **Visibility**: AdminGuard
- **Response**: `ApiResponse<Vec<RiskRuleResponse>>`

### `GET /events?<limit>`
- **Visibility**: AdminGuard
- **Response**: `ApiResponse<Vec<RiskEventResponse>>`

### `PUT /events/<uuid>`
- **Visibility**: AdminGuard
- **Request Body**: `UpdateRiskEventRequest`
- **Response**: `ApiResponse<String>`

### `POST /evaluate`
- **Visibility**: AdminGuard
- **Response**: `ApiResponse<{ events_created: i64 }>`

### `POST /subscriptions`
- **Visibility**: AuthenticatedUser
- **Request Body**: `CreateSubscriptionRequest`
- **Response**: `ApiResponse<SubscriptionResponse>`

### `GET /subscriptions`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<SubscriptionResponse>>`

### `POST /postings` & `POST /blacklist`
- `POST /postings` (AuthenticatedUser, `CreatePostingRequest`) -> `ApiResponse<{ uuid: String }>`
- `POST /blacklist` (AdminGuard, `AddBlacklistRequest`) -> `ApiResponse<{ uuid: String }>`

---

## 8. Privacy & GDPR Export (`/api/v1/privacy`)

### `POST /requests`
- **Visibility**: AuthenticatedUser
- **Request Body**: `CreateDataRequest`
- **Response**: `ApiResponse<{ uuid: String }>`
- **Description**: Self-service data export / deletion request.

### `GET /requests?<status>`
- **Visibility**: AdminGuard
- **Response**: `ApiResponse<Vec<DataRequestResponse>>`

### `GET /requests/my`
- **Visibility**: AuthenticatedUser
- **Response**: `ApiResponse<Vec<DataRequestResponse>>`

### `POST /requests/<uuid>/review`
- **Visibility**: AdminGuard
- **Request Body**: `AdminReviewDataRequest`
- **Response**: `ApiResponse<String>`
- **Description**: Admin approval for offline personal-data deletion.

### `POST /sensitive` & `GET /sensitive`
- `POST /sensitive` (AuthenticatedUser, `StoreSensitiveDataRequest`) -> `ApiResponse<String>`
- `GET /sensitive` (AuthenticatedUser) -> `ApiResponse<Vec<MaskedFieldResponse>>`
- **Description**: Storage and retrieval of field-level masked encrypted data (e.g., SSN, Bank Details).

---

## 9. Global/System Health

### `GET /health` (Mounted at `/`)
- **Visibility**: Public
- **Response**: `HealthResponse` (`status`, `service`)

### `GET /api/v1/info`
- **Visibility**: Public
- **Response**: `InfoResponse` (`name`, `version`, `description`, `api_version`)
