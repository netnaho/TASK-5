use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;

const STORAGE_TOKEN_KEY: &str = "campus_learn_token";

fn api_base() -> String {
    // Use same origin — nginx proxies /api/ and /health to the backend
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://localhost:3000".to_string())
}

pub fn get_token() -> Option<String> {
    LocalStorage::get::<String>(STORAGE_TOKEN_KEY).ok()
}

pub fn set_token(token: &str) {
    let _ = LocalStorage::set(STORAGE_TOKEN_KEY, token.to_string());
}

pub fn clear_token() {
    LocalStorage::delete(STORAGE_TOKEN_KEY);
}

pub fn is_logged_in() -> bool {
    get_token().is_some()
}

async fn do_get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", api_base(), path);
    let mut req = Request::get(&url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}: {}", resp.status(), resp.status_text()))
    }
}

async fn do_post<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, String> {
    let url = format!("{}{}", api_base(), path);
    let mut req = Request::post(&url).header("Content-Type", "application/json");
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.json(body).map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}: {}", resp.status(), resp.status_text()))
    }
}

async fn do_put<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, String> {
    let url = format!("{}{}", api_base(), path);
    let mut req = Request::put(&url).header("Content-Type", "application/json");
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.json(body).map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}: {}", resp.status(), resp.status_text()))
    }
}

async fn do_delete<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", api_base(), path);
    let mut req = Request::delete(&url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("HTTP {}: {}", resp.status(), resp.status_text()))
    }
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> { do_get(path).await }
pub async fn post<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, String> { do_post(path, body).await }
pub async fn put<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, String> { do_put(path, body).await }
pub async fn delete<T: DeserializeOwned>(path: &str) -> Result<T, String> { do_delete(path).await }
pub async fn check_health() -> Result<crate::types::HealthResponse, String> { get("/health").await }

use crate::types::*;

// Auth
pub async fn login(username: &str, password: &str) -> Result<ApiResponse<LoginResponse>, String> {
    post("/api/v1/auth/login", &serde_json::json!({"username": username, "password": password})).await
}
pub async fn get_me() -> Result<ApiResponse<UserInfo>, String> { get("/api/v1/auth/me").await }
pub async fn reauth(password: &str) -> Result<ApiResponse<String>, String> {
    post("/api/v1/auth/reauth", &serde_json::json!({"password": password})).await
}
pub async fn change_password(current: &str, new_pw: &str) -> Result<ApiResponse<String>, String> {
    post("/api/v1/auth/change-password", &serde_json::json!({"current_password": current, "new_password": new_pw})).await
}

// Courses
pub async fn list_courses() -> Result<ApiResponse<Vec<CourseResponse>>, String> { get("/api/v1/courses").await }
pub async fn get_course(uuid: &str) -> Result<ApiResponse<CourseResponse>, String> { get(&format!("/api/v1/courses/{}", uuid)).await }
pub async fn create_course(title: &str, code: &str, desc: Option<&str>) -> Result<ApiResponse<UuidResponse>, String> {
    post("/api/v1/courses", &serde_json::json!({"title": title, "code": code, "description": desc})).await
}
pub async fn update_course(uuid: &str, title: Option<&str>, desc: Option<&str>) -> Result<ApiResponse<String>, String> {
    put(&format!("/api/v1/courses/{}", uuid), &serde_json::json!({"title": title, "description": desc})).await
}
pub async fn delete_course(uuid: &str) -> Result<ApiResponse<String>, String> { delete(&format!("/api/v1/courses/{}", uuid)).await }
pub async fn get_sections(course_uuid: &str) -> Result<ApiResponse<Vec<SectionResponse>>, String> { get(&format!("/api/v1/courses/{}/sections", course_uuid)).await }
pub async fn create_section(course_uuid: &str, title: &str, sort: i32) -> Result<ApiResponse<UuidResponse>, String> {
    post(&format!("/api/v1/courses/{}/sections", course_uuid), &serde_json::json!({"title": title, "sort_order": sort})).await
}
pub async fn create_lesson(section_uuid: &str, title: &str, ct: &str, body: &str) -> Result<ApiResponse<UuidResponse>, String> {
    post(&format!("/api/v1/courses/sections/{}/lessons", section_uuid), &serde_json::json!({"title": title, "content_type": ct, "content_body": body})).await
}
pub async fn get_versions(course_uuid: &str) -> Result<ApiResponse<Vec<VersionResponse>>, String> { get(&format!("/api/v1/courses/{}/versions", course_uuid)).await }
pub async fn list_tags() -> Result<ApiResponse<Vec<TagResponse>>, String> { get("/api/v1/tags").await }
pub async fn create_tag(name: &str) -> Result<ApiResponse<TagResponse>, String> { post("/api/v1/tags", &serde_json::json!({"name": name})).await }

// Approvals
pub async fn submit_for_approval(course_uuid: &str, release_notes: &str, effective_date: &str) -> Result<ApiResponse<ApprovalUuidResponse>, String> {
    post(&format!("/api/v1/approvals/{}/submit", course_uuid), &serde_json::json!({"release_notes": release_notes, "effective_date": effective_date})).await
}
pub async fn get_approval(uuid: &str) -> Result<ApiResponse<ApprovalResponse>, String> { get(&format!("/api/v1/approvals/{}", uuid)).await }
pub async fn get_approval_queue() -> Result<ApiResponse<Vec<ApprovalQueueItem>>, String> { get("/api/v1/approvals/queue").await }
pub async fn review_approval(uuid: &str, approved: bool, comments: Option<&str>) -> Result<ApiResponse<String>, String> {
    post(&format!("/api/v1/approvals/{}/review", uuid), &serde_json::json!({"approved": approved, "comments": comments})).await
}

pub async fn submit_for_unpublish(course_uuid: &str, release_notes: &str, effective_date: &str) -> Result<ApiResponse<ApprovalUuidResponse>, String> {
    post(&format!("/api/v1/approvals/{}/unpublish", course_uuid), &serde_json::json!({"release_notes": release_notes, "effective_date": effective_date})).await
}

// Bookings
pub async fn list_resources() -> Result<ApiResponse<Vec<ResourceResponse>>, String> { get("/api/v1/bookings/resources").await }
pub async fn check_availability(resource_uuid: &str, date: &str) -> Result<ApiResponse<Vec<AvailabilitySlot>>, String> {
    get(&format!("/api/v1/bookings/resources/{}/availability?date={}", resource_uuid, date)).await
}
pub async fn create_booking(resource_uuid: &str, title: &str, start: &str, end: &str) -> Result<ApiResponse<BookingResponse>, String> {
    post("/api/v1/bookings", &serde_json::json!({"resource_uuid": resource_uuid, "title": title, "start_time": start, "end_time": end})).await
}
pub async fn reschedule_booking(uuid: &str, new_start: &str, new_end: &str, reason: Option<&str>) -> Result<ApiResponse<String>, String> {
    post(&format!("/api/v1/bookings/{}/reschedule", uuid), &serde_json::json!({"new_start_time": new_start, "new_end_time": new_end, "reason": reason})).await
}
pub async fn cancel_booking(uuid: &str) -> Result<ApiResponse<String>, String> { post(&format!("/api/v1/bookings/{}/cancel", uuid), &serde_json::json!({})).await }
pub async fn my_bookings() -> Result<ApiResponse<Vec<BookingResponse>>, String> { get("/api/v1/bookings/my").await }
pub async fn my_breaches() -> Result<ApiResponse<Vec<BreachResponse>>, String> { get("/api/v1/bookings/breaches").await }
pub async fn pending_approvals() -> Result<ApiResponse<Vec<BookingResponse>>, String> { get("/api/v1/bookings/pending-approvals").await }
pub async fn approve_booking(uuid: &str) -> Result<ApiResponse<String>, String> {
    post(&format!("/api/v1/bookings/{}/approve", uuid), &serde_json::json!({})).await
}
pub async fn reject_booking(uuid: &str, reason: Option<&str>) -> Result<ApiResponse<String>, String> {
    post(&format!("/api/v1/bookings/{}/reject", uuid), &serde_json::json!({"reason": reason})).await
}
pub async fn booker_breaches(booking_uuid: &str) -> Result<ApiResponse<Vec<BreachResponse>>, String> {
    get(&format!("/api/v1/bookings/{}/booker-breaches", booking_uuid)).await
}
pub async fn my_restrictions() -> Result<ApiResponse<Vec<RestrictionResponse>>, String> { get("/api/v1/bookings/restrictions").await }

// Risk
pub async fn list_risk_rules() -> Result<ApiResponse<Vec<RiskRuleResponse>>, String> { get("/api/v1/risk/rules").await }
pub async fn list_risk_events() -> Result<ApiResponse<Vec<RiskEventResponse>>, String> { get("/api/v1/risk/events").await }
pub async fn update_risk_event(uuid: &str, status: &str, notes: Option<&str>) -> Result<ApiResponse<String>, String> {
    put(&format!("/api/v1/risk/events/{}", uuid), &serde_json::json!({"status": status, "notes": notes})).await
}
pub async fn run_risk_evaluation() -> Result<ApiResponse<CountResponse>, String> { post("/api/v1/risk/evaluate", &serde_json::json!({})).await }
pub async fn list_subscriptions() -> Result<ApiResponse<Vec<SubscriptionResponse>>, String> { get("/api/v1/risk/subscriptions").await }
pub async fn create_subscription(event_type: &str) -> Result<ApiResponse<SubscriptionResponse>, String> {
    post("/api/v1/risk/subscriptions", &serde_json::json!({"event_type": event_type})).await
}

// Privacy
pub async fn create_data_request(request_type: &str, reason: Option<&str>) -> Result<ApiResponse<UuidResponse>, String> {
    post("/api/v1/privacy/requests", &serde_json::json!({"request_type": request_type, "reason": reason})).await
}
pub async fn my_data_requests() -> Result<ApiResponse<Vec<DataRequestResponse>>, String> { get("/api/v1/privacy/requests/my").await }
pub async fn list_all_data_requests() -> Result<ApiResponse<Vec<DataRequestResponse>>, String> { get("/api/v1/privacy/requests").await }
pub async fn review_data_request(uuid: &str, approved: bool, notes: Option<&str>) -> Result<ApiResponse<String>, String> {
    post(&format!("/api/v1/privacy/requests/{}/review", uuid), &serde_json::json!({"approved": approved, "admin_notes": notes})).await
}
pub async fn store_sensitive(field: &str, value: &str) -> Result<ApiResponse<String>, String> {
    post("/api/v1/privacy/sensitive", &serde_json::json!({"field_name": field, "value": value})).await
}
pub async fn get_masked_fields() -> Result<ApiResponse<Vec<MaskedFieldResponse>>, String> { get("/api/v1/privacy/sensitive").await }

// Audit
pub async fn list_audit_logs(limit: Option<i64>) -> Result<ApiResponse<Vec<AuditLogEntry>>, String> {
    let q = limit.map(|l| format!("?limit={}", l)).unwrap_or_default();
    get(&format!("/api/v1/audit{}", q)).await
}

// Notifications
pub async fn get_notifications() -> Result<ApiResponse<Vec<NotificationItem>>, String> {
    get("/api/v1/notifications/").await
}
pub async fn get_unread_count() -> Result<ApiResponse<UnreadCount>, String> {
    get("/api/v1/notifications/unread-count").await
}
pub async fn mark_notification_read(uuid: &str) -> Result<ApiResponse<String>, String> {
    put(&format!("/api/v1/notifications/{}/read", uuid), &serde_json::json!({})).await
}
pub async fn mark_all_notifications_read() -> Result<ApiResponse<String>, String> {
    put("/api/v1/notifications/read-all", &serde_json::json!({})).await
}
