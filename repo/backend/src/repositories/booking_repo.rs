use sqlx::MySqlPool;
use crate::models::booking::*;

pub async fn list_resources(pool: &MySqlPool) -> Result<Vec<Resource>, sqlx::Error> {
    sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE is_active = true ORDER BY name")
        .fetch_all(pool).await
}

pub async fn find_resource_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<Resource>, sqlx::Error> {
    sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn find_resource_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Resource>, sqlx::Error> {
    sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE id = ?")
        .bind(id).fetch_optional(pool).await
}

/// Create booking using SELECT FOR UPDATE to prevent double-booking under concurrency.
/// Returns Ok(last_insert_id) or Err if conflict detected within the transaction.
pub async fn create_booking_atomic(
    pool: &MySqlPool, uuid: &str, resource_id: i64, booked_by: i64,
    title: &str, description: Option<&str>, start_time: &str, end_time: &str,
    status: &str,
) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Lock the resource row to serialize booking insertions for this resource
    sqlx::query("SELECT id FROM resources WHERE id = ? FOR UPDATE")
        .bind(resource_id).execute(&mut *tx).await?;

    // Check for overlapping confirmed/pending bookings
    let conflict: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM bookings WHERE resource_id = ? AND status IN ('confirmed', 'pending') AND start_time < ? AND end_time > ? LIMIT 1"
    ).bind(resource_id).bind(end_time).bind(start_time)
    .fetch_optional(&mut *tx).await?;

    if conflict.is_some() {
        tx.rollback().await?;
        return Err(sqlx::Error::Protocol("BOOKING_CONFLICT: Time slot already booked".to_string()));
    }

    // Check for maintenance blackout overlap
    let blackout: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM resource_blackouts WHERE resource_id = ? AND start_time < ? AND end_time > ? LIMIT 1"
    ).bind(resource_id).bind(end_time).bind(start_time)
    .fetch_optional(&mut *tx).await?;

    if blackout.is_some() {
        tx.rollback().await?;
        return Err(sqlx::Error::Protocol("BLACKOUT_CONFLICT: Resource under maintenance during requested time".to_string()));
    }

    let r = sqlx::query(
        "INSERT INTO bookings (uuid, resource_id, booked_by, title, description, start_time, end_time, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    ).bind(uuid).bind(resource_id).bind(booked_by).bind(title).bind(description)
    .bind(start_time).bind(end_time).bind(status)
    .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(r.last_insert_id())
}

/// Reschedule an existing booking atomically.
///
/// Acquires a row-level lock on the resource (SELECT FOR UPDATE) inside a
/// transaction so that two concurrent reschedules to the same slot cannot both
/// pass the conflict check. Returns Ok(()) on success or a `BOOKING_CONFLICT`
/// protocol error if the new slot overlaps another booking.
pub async fn reschedule_booking_atomic(
    pool: &MySqlPool,
    booking_id: i64,
    resource_id: i64,
    reschedule_uuid: &str,
    reschedule_number: i32,
    requested_by: i64,
    orig_start: &str,
    orig_end: &str,
    new_start: &str,
    new_end: &str,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Lock resource row — serializes concurrent reschedule + create operations
    sqlx::query("SELECT id FROM resources WHERE id = ? FOR UPDATE")
        .bind(resource_id).execute(&mut *tx).await?;

    // Conflict check (exclude the booking being rescheduled)
    let conflict: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM bookings WHERE resource_id = ? AND id != ? \
         AND status IN ('confirmed', 'pending') \
         AND start_time < ? AND end_time > ? LIMIT 1"
    ).bind(resource_id).bind(booking_id).bind(new_end).bind(new_start)
    .fetch_optional(&mut *tx).await?;

    if conflict.is_some() {
        tx.rollback().await?;
        return Err(sqlx::Error::Protocol("BOOKING_CONFLICT: Time slot already booked".to_string()));
    }

    // Record reschedule history
    sqlx::query(
        "INSERT INTO booking_reschedules (uuid, booking_id, reschedule_number, requested_by, \
         original_start, original_end, new_start, new_end, reason, status) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'approved')"
    ).bind(reschedule_uuid).bind(booking_id).bind(reschedule_number).bind(requested_by)
    .bind(orig_start).bind(orig_end).bind(new_start).bind(new_end).bind(reason)
    .execute(&mut *tx).await?;

    // Update booking times and reschedule counter atomically
    sqlx::query("UPDATE bookings SET start_time = ?, end_time = ?, updated_at = NOW() WHERE id = ?")
        .bind(new_start).bind(new_end).bind(booking_id)
        .execute(&mut *tx).await?;

    sqlx::query("UPDATE bookings SET reschedule_count = reschedule_count + 1, updated_at = NOW() WHERE id = ?")
        .bind(booking_id).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn find_booking_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn find_booking_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE id = ?")
        .bind(id).fetch_optional(pool).await
}

pub async fn list_user_bookings(pool: &MySqlPool, user_id: i64) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>("SELECT * FROM bookings WHERE booked_by = ? ORDER BY start_time DESC")
        .bind(user_id).fetch_all(pool).await
}

pub async fn count_active_bookings_for_resource(pool: &MySqlPool, user_id: i64, resource_id: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bookings WHERE booked_by = ? AND resource_id = ? AND status IN ('confirmed', 'pending') AND end_time > NOW()"
    ).bind(user_id).bind(resource_id).fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn approve_booking(pool: &MySqlPool, id: i64, approved_by: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE bookings SET status = 'confirmed', approved_by = ?, approved_at = NOW(), updated_at = NOW() WHERE id = ? AND status = 'pending'")
        .bind(approved_by).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn reject_booking(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE bookings SET status = 'cancelled', updated_at = NOW() WHERE id = ? AND status = 'pending'")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn update_booking_status(pool: &MySqlPool, id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE bookings SET status = ?, updated_at = NOW() WHERE id = ?")
        .bind(status).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn increment_reschedule_count(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE bookings SET reschedule_count = reschedule_count + 1, updated_at = NOW() WHERE id = ?")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn update_booking_times(pool: &MySqlPool, id: i64, start: &str, end: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE bookings SET start_time = ?, end_time = ?, updated_at = NOW() WHERE id = ?")
        .bind(start).bind(end).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn create_reschedule_record(
    pool: &MySqlPool, uuid: &str, booking_id: i64, reschedule_number: i32, requested_by: i64,
    original_start: &str, original_end: &str, new_start: &str, new_end: &str, reason: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO booking_reschedules (uuid, booking_id, reschedule_number, requested_by, original_start, original_end, new_start, new_end, reason, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'approved')"
    ).bind(uuid).bind(booking_id).bind(reschedule_number).bind(requested_by)
    .bind(original_start).bind(original_end).bind(new_start).bind(new_end).bind(reason)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

// Blackouts
pub async fn list_blackouts(pool: &MySqlPool, resource_id: i64) -> Result<Vec<ResourceBlackout>, sqlx::Error> {
    sqlx::query_as::<_, ResourceBlackout>("SELECT * FROM resource_blackouts WHERE resource_id = ? AND end_time > NOW() ORDER BY start_time")
        .bind(resource_id).fetch_all(pool).await
}

pub async fn create_blackout(pool: &MySqlPool, uuid: &str, resource_id: i64, reason: &str, start: &str, end: &str, created_by: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO resource_blackouts (uuid, resource_id, reason, start_time, end_time, created_by) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(uuid).bind(resource_id).bind(reason).bind(start).bind(end).bind(created_by)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

// Breaches
pub async fn create_breach(pool: &MySqlPool, uuid: &str, user_id: i64, booking_id: Option<i64>, breach_type: &str, severity: &str, description: &str) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO breaches (uuid, user_id, booking_id, breach_type, severity, description, status) VALUES (?, ?, ?, ?, ?, ?, 'open')"
    ).bind(uuid).bind(user_id).bind(booking_id).bind(breach_type).bind(severity).bind(description)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn count_recent_breaches(pool: &MySqlPool, user_id: i64, days: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM breaches WHERE user_id = ? AND created_at > DATE_SUB(NOW(), INTERVAL ? DAY)"
    ).bind(user_id).bind(days).fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn list_user_breaches(pool: &MySqlPool, user_id: i64) -> Result<Vec<Breach>, sqlx::Error> {
    sqlx::query_as::<_, Breach>("SELECT * FROM breaches WHERE user_id = ? ORDER BY created_at DESC")
        .bind(user_id).fetch_all(pool).await
}

// Restrictions
pub async fn create_restriction(pool: &MySqlPool, uuid: &str, user_id: i64, restriction_type: &str, reason: &str, imposed_by: i64, starts_at: &str, expires_at: Option<&str>, breach_count: i32, auto_triggered: bool) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO restrictions (uuid, user_id, restriction_type, reason, imposed_by, starts_at, expires_at, is_active, breach_count, auto_triggered) VALUES (?, ?, ?, ?, ?, ?, ?, TRUE, ?, ?)"
    ).bind(uuid).bind(user_id).bind(restriction_type).bind(reason).bind(imposed_by)
    .bind(starts_at).bind(expires_at).bind(breach_count).bind(auto_triggered)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn has_active_restriction(pool: &MySqlPool, user_id: i64, restriction_type: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM restrictions WHERE user_id = ? AND restriction_type = ? AND is_active = TRUE AND (expires_at IS NULL OR expires_at > NOW()) LIMIT 1"
    ).bind(user_id).bind(restriction_type).fetch_optional(pool).await?;
    Ok(row.is_some())
}

pub async fn list_active_restrictions(pool: &MySqlPool, user_id: i64) -> Result<Vec<Restriction>, sqlx::Error> {
    sqlx::query_as::<_, Restriction>(
        "SELECT * FROM restrictions WHERE user_id = ? AND is_active = TRUE AND (expires_at IS NULL OR expires_at > NOW()) ORDER BY created_at DESC"
    ).bind(user_id).fetch_all(pool).await
}

// Pending bookings - for reviewer approval queues
pub async fn list_all_pending(pool: &MySqlPool) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE status = 'pending' ORDER BY start_time ASC"
    ).fetch_all(pool).await
}

pub async fn list_pending_by_department(pool: &MySqlPool, department_id: i64) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        "SELECT b.* FROM bookings b INNER JOIN resources r ON b.resource_id = r.id WHERE b.status = 'pending' AND r.department_id = ? ORDER BY b.start_time ASC"
    ).bind(department_id).fetch_all(pool).await
}

// Availability check - find existing bookings for a resource in a date range
pub async fn find_conflicts(pool: &MySqlPool, resource_id: i64, start: &str, end: &str) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE resource_id = ? AND status IN ('confirmed', 'pending') AND start_time < ? AND end_time > ?"
    ).bind(resource_id).bind(end).bind(start).fetch_all(pool).await
}
