use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{NaiveDateTime, Duration, Utc, NaiveTime, Timelike};

use crate::dto::booking::*;
use crate::models::booking::*;
use crate::repositories::{booking_repo, audit_repo, security_repo};
use crate::services::{notification_service, term_service};
use crate::utils::errors::AppError;

pub async fn list_resources(pool: &MySqlPool) -> Result<Vec<ResourceResponse>, AppError> {
    let resources = booking_repo::list_resources(pool).await?;
    Ok(resources.into_iter().map(|r| ResourceResponse {
        uuid: r.uuid, name: r.name, resource_type: r.resource_type,
        location: r.location, capacity: r.capacity, description: r.description,
        open_time: r.open_time.format("%H:%M").to_string(),
        close_time: r.close_time.format("%H:%M").to_string(),
        max_booking_hours: r.max_booking_hours,
        requires_approval: r.requires_approval, is_active: r.is_active,
        department_id: r.department_id,
    }).collect())
}

pub async fn check_availability(pool: &MySqlPool, resource_uuid: &str, date: &str) -> Result<Vec<AvailabilitySlot>, AppError> {
    let resource = booking_repo::find_resource_by_uuid(pool, resource_uuid).await?
        .ok_or_else(|| AppError::NotFound("Resource not found".to_string()))?;

    let start_str = format!("{} {}", date, resource.open_time.format("%H:%M:%S"));
    let close_str = format!("{} {}", date, resource.close_time.format("%H:%M:%S"));

    let existing = booking_repo::find_conflicts(pool, resource.id, &start_str, &close_str).await?;
    let blackouts = booking_repo::list_blackouts(pool, resource.id).await?;

    // Generate hourly slots
    let mut slots = Vec::new();
    let open_hour = resource.open_time.hour() as i64;
    let close_hour = resource.close_time.hour() as i64;

    for hour in open_hour..close_hour {
        let slot_start = format!("{} {:02}:00:00", date, hour);
        let slot_end = format!("{} {:02}:00:00", date, hour + 1);

        let conflict = existing.iter().any(|b| {
            let bs = b.start_time.format("%Y-%m-%d %H:%M:%S").to_string();
            let be = b.end_time.format("%Y-%m-%d %H:%M:%S").to_string();
            bs < slot_end && be > slot_start
        });

        let blackout = blackouts.iter().any(|bl| {
            let bs = bl.start_time.format("%Y-%m-%d %H:%M:%S").to_string();
            let be = bl.end_time.format("%Y-%m-%d %H:%M:%S").to_string();
            bs < slot_end && be > slot_start
        });

        let (available, reason) = if blackout {
            (false, Some("Maintenance blackout".to_string()))
        } else if conflict {
            (false, Some("Already booked".to_string()))
        } else {
            (true, None)
        };

        slots.push(AvailabilitySlot {
            start: slot_start, end: slot_end, available, conflict_reason: reason,
        });
    }
    Ok(slots)
}

pub async fn create_booking(
    pool: &MySqlPool, req: &CreateBookingRequest, user_id: i64, correlation_id: Option<&str>,
) -> Result<BookingResponse, AppError> {
    let resource = booking_repo::find_resource_by_uuid(pool, &req.resource_uuid).await?
        .ok_or_else(|| AppError::NotFound("Resource not found".to_string()))?;

    // Check user doesn't have active booking restriction
    if booking_repo::has_active_restriction(pool, user_id, "booking_suspended").await? {
        return Err(AppError::Forbidden("You have an active booking restriction due to policy breaches".to_string()));
    }

    // Enforce active term acceptance
    term_service::check_active_term_accepted(pool, user_id).await?;

    // Parse times
    let start_dt = parse_datetime(&req.start_time)?;
    let end_dt = parse_datetime(&req.end_time)?;
    let now = Utc::now().naive_utc();

    // Validate: end > start
    if end_dt <= start_dt {
        return Err(AppError::Validation("End time must be after start time".to_string()));
    }

    // Validate: max 90 days ahead
    let max_future = now + Duration::days(MAX_ADVANCE_DAYS);
    if start_dt > max_future {
        return Err(AppError::Validation(format!("Cannot book more than {} days in advance", MAX_ADVANCE_DAYS)));
    }

    // Validate: within operating hours
    let start_time_only = start_dt.time();
    let end_time_only = end_dt.time();
    if start_time_only < resource.open_time || end_time_only > resource.close_time {
        return Err(AppError::Validation(format!(
            "Booking must be within resource hours: {} - {}",
            resource.open_time.format("%H:%M"), resource.close_time.format("%H:%M")
        )));
    }

    // Validate: max duration for room type (use minutes for precision)
    let duration_minutes = (end_dt - start_dt).num_minutes();
    if duration_minutes > (resource.max_booking_hours as i64) * 60 {
        return Err(AppError::Validation(format!(
            "Maximum booking duration for this resource is {} hours", resource.max_booking_hours
        )));
    }

    // Validate: max 2 active reservations per user per resource
    let active_count = booking_repo::count_active_bookings_for_resource(pool, user_id, resource.id).await?;
    if active_count >= MAX_ACTIVE_PER_RESOURCE as i64 {
        return Err(AppError::Validation(format!(
            "Maximum {} active reservations per resource", MAX_ACTIVE_PER_RESOURCE
        )));
    }

    let uuid = Uuid::new_v4().to_string();
    let start_str = start_dt.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    // Determine initial status based on resource approval requirement
    let initial_status = if resource.requires_approval { "pending" } else { "confirmed" };

    // Atomic booking with conflict prevention
    booking_repo::create_booking_atomic(
        pool, &uuid, resource.id, user_id, &req.title,
        req.description.as_deref(), &start_str, &end_str, initial_status,
    ).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("BOOKING_CONFLICT") {
            AppError::Validation("Time slot conflict: this resource is already booked for the requested time".to_string())
        } else if msg.contains("BLACKOUT_CONFLICT") {
            AppError::Validation("Resource is under maintenance during the requested time".to_string())
        } else {
            AppError::Database(e)
        }
    })?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "booking.create",
        "booking", None, None,
        Some(&serde_json::json!({"resource": resource.name, "start": start_str, "end": end_str, "status": initial_status})),
        None, None, correlation_id,
    ).await;

    if initial_status == "pending" {
        let _ = notification_service::notify_user(
            pool, user_id,
            "Booking Submitted for Approval",
            &format!("Your booking '{}' has been submitted and is pending approval.", req.title),
            "booking", Some("booking"), Some(&uuid),
        ).await;
        let _ = notification_service::notify_department_role(
            pool, resource.department_id, "dept_reviewer",
            "New Booking Awaiting Approval",
            &format!("A booking '{}' for '{}' requires approval.", req.title, resource.name),
            "booking", Some("booking"), Some(&uuid),
        ).await;
    } else {
        let _ = notification_service::notify_user(
            pool, user_id,
            "Booking Confirmed",
            &format!("Your booking '{}' has been confirmed.", req.title),
            "booking", Some("booking"), Some(&uuid),
        ).await;
    }

    Ok(BookingResponse {
        uuid, resource_id: resource.id, resource_name: Some(resource.name),
        booked_by: user_id, title: req.title.clone(),
        description: req.description.clone(),
        start_time: start_str, end_time: end_str,
        status: initial_status.to_string(), reschedule_count: 0,
        approved_by: None, approved_at: None,
        created_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    })
}

pub async fn reschedule_booking(
    pool: &MySqlPool, booking_uuid: &str, req: &RescheduleRequest, user_id: i64,
) -> Result<(), AppError> {
    let booking = booking_repo::find_booking_by_uuid(pool, booking_uuid).await?
        .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.booked_by != user_id {
        return Err(AppError::Forbidden("You can only reschedule your own bookings".to_string()));
    }

    if booking.reschedule_count >= MAX_RESCHEDULES {
        return Err(AppError::Validation(format!("Maximum {} reschedules allowed", MAX_RESCHEDULES)));
    }

    if booking.status != "confirmed" {
        return Err(AppError::Validation("Only confirmed bookings can be rescheduled".to_string()));
    }

    let new_start = parse_datetime(&req.new_start_time)?;
    let new_end = parse_datetime(&req.new_end_time)?;

    if new_end <= new_start {
        return Err(AppError::Validation("End time must be after start time".to_string()));
    }

    let resource = booking_repo::find_resource_by_id(pool, booking.resource_id).await?
        .ok_or_else(|| AppError::Internal("Resource not found".to_string()))?;

    // Check operating hours
    if new_start.time() < resource.open_time || new_end.time() > resource.close_time {
        return Err(AppError::Validation("New time must be within resource operating hours".to_string()));
    }

    // Check duration (use minutes for precision)
    let duration_minutes = (new_end - new_start).num_minutes();
    if duration_minutes > (resource.max_booking_hours as i64) * 60 {
        return Err(AppError::Validation(format!("Max {} hours", resource.max_booking_hours)));
    }

    let new_start_str = new_start.format("%Y-%m-%d %H:%M:%S").to_string();
    let new_end_str = new_end.format("%Y-%m-%d %H:%M:%S").to_string();

    let orig_start = booking.start_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let orig_end = booking.end_time.format("%Y-%m-%d %H:%M:%S").to_string();

    booking_repo::reschedule_booking_atomic(
        pool, booking.id, booking.resource_id,
        &Uuid::new_v4().to_string(),
        booking.reschedule_count + 1,
        user_id,
        &orig_start, &orig_end,
        &new_start_str, &new_end_str,
        req.reason.as_deref(),
    ).await.map_err(|e| {
        if e.to_string().contains("BOOKING_CONFLICT") {
            AppError::Validation("New time conflicts with existing booking".to_string())
        } else {
            AppError::Internal(e.to_string())
        }
    })?;

    Ok(())
}

pub async fn cancel_booking(
    pool: &MySqlPool, booking_uuid: &str, user_id: i64, role: &str, correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let booking = booking_repo::find_booking_by_uuid(pool, booking_uuid).await?
        .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.booked_by != user_id && role != "admin" {
        return Err(AppError::Forbidden("Not your booking".to_string()));
    }

    if booking.status == "cancelled" {
        return Err(AppError::Validation("Booking already cancelled".to_string()));
    }

    booking_repo::update_booking_status(pool, booking.id, "cancelled").await?;

    // Late cancellation check: within 2 hours of start creates a breach (only for confirmed bookings)
    let now = Utc::now().naive_utc();
    let hours_until_start = (booking.start_time - now).num_hours();

    if booking.status == "confirmed" && hours_until_start < LATE_CANCEL_HOURS && booking.start_time > now {
        let breach_uuid = Uuid::new_v4().to_string();
        booking_repo::create_breach(
            pool, &breach_uuid, booking.booked_by, Some(booking.id),
            "late_cancellation", "medium",
            &format!("Booking cancelled within {} hours of start time", LATE_CANCEL_HOURS),
        ).await?;

        let _ = notification_service::notify_user(
            pool, booking.booked_by,
            "Late Cancellation Recorded",
            "A late cancellation breach has been recorded on your account.",
            "booking", None, None,
        ).await;

        // Check if user hit breach threshold -> auto-restrict
        let breach_count = booking_repo::count_recent_breaches(pool, booking.booked_by, BREACH_WINDOW_DAYS).await?;
        if breach_count >= BREACH_THRESHOLD {
            let already_restricted = booking_repo::has_active_restriction(pool, booking.booked_by, "booking_suspended").await?;
            if !already_restricted {
                let expires = (Utc::now() + Duration::days(30)).format("%Y-%m-%d %H:%M:%S").to_string();
                booking_repo::create_restriction(
                    pool, &Uuid::new_v4().to_string(), booking.booked_by,
                    "booking_suspended",
                    &format!("{} breaches in {} days triggered automatic booking suspension", breach_count, BREACH_WINDOW_DAYS),
                    user_id, &Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    Some(&expires), breach_count as i32, true,
                ).await?;

                let _ = security_repo::create_security_event(
                    pool, &Uuid::new_v4().to_string(), "booking_auto_restriction", "warning",
                    Some(booking.booked_by), None,
                    &format!("Auto-restricted: {} breaches in {} days", breach_count, BREACH_WINDOW_DAYS),
                    None, correlation_id,
                ).await;

                let _ = notification_service::notify_user(
                    pool, booking.booked_by,
                    "Booking Privileges Suspended",
                    &format!("{} policy breaches in {} days triggered an automatic booking suspension.", breach_count, BREACH_WINDOW_DAYS),
                    "booking", None, None,
                ).await;
            }
        }
    }

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "booking.cancel",
        "booking", Some(booking.id), None,
        Some(&serde_json::json!({"hours_until_start": hours_until_start})),
        None, None, correlation_id,
    ).await;

    Ok(())
}

pub async fn approve_booking(
    pool: &MySqlPool, booking_uuid: &str, reviewer_id: i64,
    reviewer_role: &str, reviewer_department_id: Option<i64>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let booking = booking_repo::find_booking_by_uuid(pool, booking_uuid).await?
        .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.status != "pending" {
        return Err(AppError::Validation("Only pending bookings can be approved".to_string()));
    }

    // Department scope check: non-admin reviewers can only approve bookings for resources in their department
    if reviewer_role != "admin" {
        let resource = booking_repo::find_resource_by_id(pool, booking.resource_id).await?
            .ok_or_else(|| AppError::Internal("Resource not found".to_string()))?;
        if let Some(res_dept) = resource.department_id {
            if reviewer_department_id != Some(res_dept) {
                return Err(AppError::Forbidden("You can only approve bookings for resources in your department".to_string()));
            }
        }
    }

    booking_repo::approve_booking(pool, booking.id, reviewer_id).await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(reviewer_id), "booking.approve",
        "booking", Some(booking.id), None,
        Some(&serde_json::json!({"booking_uuid": booking_uuid})),
        None, None, correlation_id,
    ).await;

    let _ = notification_service::notify_user(
        pool, booking.booked_by,
        "Booking Approved",
        &format!("Your booking '{}' has been approved.", booking.title),
        "booking", Some("booking"), Some(&booking.uuid),
    ).await;

    Ok(())
}

pub async fn reject_booking(
    pool: &MySqlPool, booking_uuid: &str, reviewer_id: i64,
    reviewer_role: &str, reviewer_department_id: Option<i64>,
    reason: Option<&str>, correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let booking = booking_repo::find_booking_by_uuid(pool, booking_uuid).await?
        .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.status != "pending" {
        return Err(AppError::Validation("Only pending bookings can be rejected".to_string()));
    }

    // Department scope check
    if reviewer_role != "admin" {
        let resource = booking_repo::find_resource_by_id(pool, booking.resource_id).await?
            .ok_or_else(|| AppError::Internal("Resource not found".to_string()))?;
        if let Some(res_dept) = resource.department_id {
            if reviewer_department_id != Some(res_dept) {
                return Err(AppError::Forbidden("You can only reject bookings for resources in your department".to_string()));
            }
        }
    }

    booking_repo::reject_booking(pool, booking.id).await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(reviewer_id), "booking.reject",
        "booking", Some(booking.id), None,
        Some(&serde_json::json!({"booking_uuid": booking_uuid, "reason": reason})),
        None, None, correlation_id,
    ).await;

    let _ = notification_service::notify_user(
        pool, booking.booked_by,
        "Booking Rejected",
        &format!("Your booking '{}' has been rejected.{}", booking.title,
            reason.map(|r| format!(" Reason: {}", r)).unwrap_or_default()),
        "booking", Some("booking"), Some(&booking.uuid),
    ).await;

    Ok(())
}

pub async fn list_user_bookings(pool: &MySqlPool, user_id: i64) -> Result<Vec<BookingResponse>, AppError> {
    let bookings = booking_repo::list_user_bookings(pool, user_id).await?;
    let mut result = Vec::new();
    for b in bookings {
        let resource = booking_repo::find_resource_by_id(pool, b.resource_id).await?;
        result.push(BookingResponse {
            uuid: b.uuid, resource_id: b.resource_id,
            resource_name: resource.map(|r| r.name),
            booked_by: b.booked_by, title: b.title, description: b.description,
            start_time: b.start_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            end_time: b.end_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            status: b.status, reschedule_count: b.reschedule_count,
            approved_by: b.approved_by,
            approved_at: b.approved_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            created_at: b.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        });
    }
    Ok(result)
}

pub async fn list_breaches(pool: &MySqlPool, user_id: i64) -> Result<Vec<BreachResponse>, AppError> {
    let breaches = booking_repo::list_user_breaches(pool, user_id).await?;
    Ok(breaches.into_iter().map(|b| BreachResponse {
        uuid: b.uuid, user_id: b.user_id, booking_id: b.booking_id,
        breach_type: b.breach_type, severity: b.severity, description: b.description,
        status: b.status, created_at: b.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }).collect())
}

pub async fn list_restrictions(pool: &MySqlPool, user_id: i64) -> Result<Vec<RestrictionResponse>, AppError> {
    let restrictions = booking_repo::list_active_restrictions(pool, user_id).await?;
    Ok(restrictions.into_iter().map(|r| RestrictionResponse {
        uuid: r.uuid, user_id: r.user_id, restriction_type: r.restriction_type,
        reason: r.reason,
        starts_at: r.starts_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        expires_at: r.expires_at.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
        is_active: r.is_active, auto_triggered: r.auto_triggered,
    }).collect())
}

pub async fn list_pending_approvals(
    pool: &MySqlPool, reviewer_role: &str, reviewer_department_id: Option<i64>,
) -> Result<Vec<BookingResponse>, AppError> {
    let bookings = if reviewer_role == "admin" {
        booking_repo::list_all_pending(pool).await?
    } else {
        match reviewer_department_id {
            Some(dept_id) => booking_repo::list_pending_by_department(pool, dept_id).await?,
            None => vec![],
        }
    };
    let mut result = Vec::new();
    for b in bookings {
        let resource = booking_repo::find_resource_by_id(pool, b.resource_id).await?;
        result.push(BookingResponse {
            uuid: b.uuid, resource_id: b.resource_id,
            resource_name: resource.map(|r| r.name),
            booked_by: b.booked_by, title: b.title, description: b.description,
            start_time: b.start_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            end_time: b.end_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            status: b.status, reschedule_count: b.reschedule_count,
            approved_by: b.approved_by,
            approved_at: b.approved_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            created_at: b.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        });
    }
    Ok(result)
}

pub async fn get_booker_breaches(pool: &MySqlPool, booking_uuid: &str) -> Result<Vec<BreachResponse>, AppError> {
    let booking = booking_repo::find_booking_by_uuid(pool, booking_uuid).await?
        .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;
    let breaches = booking_repo::list_user_breaches(pool, booking.booked_by).await?;
    Ok(breaches.into_iter().map(|b| BreachResponse {
        uuid: b.uuid, user_id: b.user_id, booking_id: b.booking_id,
        breach_type: b.breach_type, severity: b.severity, description: b.description,
        status: b.status, created_at: b.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }).collect())
}

fn parse_datetime(s: &str) -> Result<NaiveDateTime, AppError> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M"))
        .map_err(|_| AppError::Validation("Invalid datetime format. Use YYYY-MM-DD HH:MM:SS".to_string()))
}
