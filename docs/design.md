# System Architecture & Design Document
**Project**: CampusLearn Operations Suite

## 1. System Overview
CampusLearn Operations Suite is an enterprise-grade platform for managing an institution’s offline learning catalog, shared facilities, and compliance risk signals. Developed to run completely offline without external internet dependencies, the system prioritizes strict role-based access control, concurrency-safe transactions, compliance auditing, and sensitive data protection.

## 2. High-Level Architecture
The system follows a standard three-tier architecture, heavily utilizing the Rust ecosystem for both the backend and frontend to ensure memory safety, performance, and type-safe contracts.

```text
┌─────────────────────────┐      ┌─────────────────────────┐      ┌─────────────────────────┐
│       Presentation      │      │     Application API     │      │        Data Tier        │
│                         │      │                         │      │                         │
│   Dioxus (WASM) SPA     │ HTTP │   Rust / Rocket API     │ TCP  │    MySQL 8.0 Server     │
│   Served via Nginx      │─────▶│   Port: 8000            │─────▶│    Port: 3306           │
│   Port: 3000            │ JSON │   RESTful Interfaces    │ SQL  │    Strict Schema        │
└─────────────────────────┘      └─────────────────────────┘      └─────────────────────────┘
```

## 3. Backend Design

The backend is a monolithic Rust application built using the **Rocket** web framework, connecting to a **MySQL 8.0** database using **SQLx** for compile-time verified queries.

### 3.1 Layered Architecture
To maintain separation of concerns, the backend is strictly divided into four layers:
- **Routes (`src/routes`)**: Handles HTTP parsing, payload validation (via the `validator` crate), and mapping to specific security guards (e.g., `AdminGuard`, `ReviewerGuard`).
- **Services (`src/services`)**: Contains the core business logic. Enforces limits, rule engines, and orchestrates database calls. 
- **Repositories (`src/repositories`)**: Encapsulates all SQL queries and data persistence logic using SQLx. 
- **Models / DTOs (`src/models`, `src/dto`)**: Strongly-typed data structures representing database entities and API request/response payloads.

### 3.2 Authentication & Authorization (RBAC)
- **Authentication**: Local username and password verification (using `bcrypt` hashing). Sessions are managed statelessly using signed **JSON Web Tokens (JWT)**.
- **Role-Based Access Control (RBAC)**: Implemented via Rocket request guards. Roles include:
  - `Admin`: Full system access, Risk & Compliance, Privacy management.
  - `Staff Author`: Course creation, editing, and submission.
  - `Department Reviewer`: Approval queues operations.
  - `Faculty` & `Student`: Read-only access to published catalogs; facility booking privileges.

### 3.3 Concurrency & Database Transactions
Booking functionalities rely on explicit database transactions and pessimistic locking where necessary. Unique time-range conflict checks are performed within SQL transactions to prevent double-booking under parallel request loads.

### 3.4 Background Jobs & Asynchronous Engines
The backend implements an internal scheduler (running asynchronously) to manage ongoing operations:
- **Publishing Engine**: Periodically checks for `approved_scheduled` courses whose effective date has passed and moves them to `published`.
- **Booking Breach Engine**: Checks for late cancellations or no-shows, tallying breaches and automatically applying restrictions (e.g., 30-day suspension for 3 violations in 60 days).
- **Risk & Anomaly Engine**: Runs every 15 minutes to evaluate data against configured rules (e.g., duplicated job postings, abnormal compensation, blacklisted employers) and queues local/on-prem webhooks.

## 4. Frontend Design

The frontend is a Single Page Application (SPA) built entirely in Rust using **Dioxus**, compiled to WebAssembly (WASM).

- **Routing**: Client-side routing with role-aware navigation guards. If a `Student` attempts to access the `/approvals` route, they are securely redirected.
- **State Management**: Utilizes Dioxus signals and context providers to manage global authentication state and UI themes.
- **API Client**: A strongly-typed internal API client wraps standard `reqwest` calls, automatically injecting the JWT Bearer token into headers and gracefully handling 401 Unauthorized responses to trigger re-logins.
- **UI Components**: Employs a reusable component library (Modals, DataTables, StatusBadges) styled with a dark, enterprise-grade tailored theme using vanilla CSS.

## 5. Security & Compliance Measures

As a system handling student data and institution compliance, security is baked into the architecture at every level.

### 5.1 Defense in Depth
- **Replay Attacks**: Handled via signed API requests containing nonces with strict 5-minute expiration windows for integrations.
- **Rate Limiting**: Enforced rate limiting middleware (e.g., 120 requests/minute per account) prevents abusive scraping of the offline catalog.
- **Input Validation**: Strict field-level validation against XSS and SQL Injection on the API boundary.

### 5.2 Privacy & Sensitive Data
- **Encryption at Rest**: Sensitive fields (like Social Security Numbers or banking details) are encrypted in MySQL using AES-256-GCM. 
- **Field-Level Masking**: APIs automatically mask sensitive data upon retrieval (e.g., returning `***-**-1234`), requiring a specific "unmask" action (which generates an audit log) to view the clear text.
- **Data Subject Requests**: Built-in workflows for user-initiated data export and deletion, enforced by an Admin review process.

### 5.3 High-Risk Operations (Sudo Mode)
Administrative actions or viewing raw sensitive data require the user to re-authenticate if their session is older than 15 minutes. This is enforced by a `Sudo Mode` claim on the JWT, requiring a call to `POST /api/v1/auth/reauth`.

### 5.4 Immutable Audit Logging
A dedicated `audit_logs` table records every write, update, and deletion action across the system. This log tracks the actor's User ID, the correlation ID for the request, the entity affected, and the before/after state. The business rules guarantee these logs are immutable and enforce a strict 7-year retention policy.
