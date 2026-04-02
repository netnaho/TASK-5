use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth::password::hash_password;
use crate::repositories::user_repo;

pub async fn seed_default_users(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    seed_departments(pool).await?;
    seed_permissions(pool).await?;

    let count = user_repo::count_users(pool).await?;
    if count > 0 {
        tracing::info!("Users already exist, skipping user seed");
        return Ok(());
    }

    tracing::info!("Seeding default accounts");

    let accounts = [
        ("admin", "Admin@12345678", "admin@campuslearn.local", "System Administrator", "admin"),
        ("author", "Author@1234567", "author@campuslearn.local", "Default Author", "staff_author"),
        ("reviewer", "Review@1234567", "reviewer@campuslearn.local", "Default Reviewer", "dept_reviewer"),
        ("faculty", "Faculty@123456", "faculty@campuslearn.local", "Default Faculty", "faculty"),
        ("student", "Student@12345", "student@campuslearn.local", "Default Student", "student"),
    ];

    for (username, password, email, full_name, role) in accounts {
        let pw_hash = hash_password(password)?;
        match user_repo::create_user(pool, &Uuid::new_v4().to_string(), username, &pw_hash, email, full_name, role).await {
            Ok(_) => tracing::info!(username = username, role = role, "Seeded user"),
            Err(e) => {
                tracing::error!(username = username, role = role, error = %e, "Failed to seed user");
                return Err(e.into());
            }
        }
    }

    tracing::info!("Default accounts seeded successfully");
    Ok(())
}

async fn seed_departments(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM departments").fetch_one(pool).await?;
    if count.0 > 0 { return Ok(()); }

    tracing::info!("Seeding default departments");
    let departments = [
        ("Computer Science", "CS", "Department of Computer Science"),
        ("Mathematics", "MATH", "Department of Mathematics"),
        ("Engineering", "ENG", "Department of Engineering"),
        ("Business", "BUS", "Department of Business Administration"),
    ];
    for (name, code, desc) in departments {
        sqlx::query("INSERT INTO departments (uuid, name, code, description) VALUES (?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string()).bind(name).bind(code).bind(desc)
            .execute(pool).await?;
    }
    Ok(())
}

async fn seed_permissions(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM permissions").fetch_one(pool).await?;
    if count.0 > 0 { return Ok(()); }
    Ok(())
}

pub async fn seed_resources_and_rules(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    // Seed resources
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM resources").fetch_one(pool).await?;
    if count.0 == 0 {
        tracing::info!("Seeding default resources");
        let resources = [
            ("Conference Room A", "room", "Building A, Floor 2", 20, "Large meeting room with projector", "07:00:00", "22:00:00", 4),
            ("Conference Room B", "room", "Building A, Floor 3", 10, "Small meeting room", "07:00:00", "22:00:00", 4),
            ("Main Clubhouse", "studio", "Student Center", 100, "Clubhouse for events", "08:00:00", "23:00:00", 8),
            ("Parking Lot A - Permit", "equipment", "North Campus", 200, "Daily parking permit", "06:00:00", "23:59:00", 24),
            ("Computer Lab 101", "lab", "Science Building, Room 101", 30, "Computer lab with 30 workstations", "07:00:00", "21:00:00", 4),
        ];
        for (name, rtype, loc, cap, desc, open, close, max_h) in resources {
            sqlx::query("INSERT INTO resources (uuid, name, resource_type, location, capacity, description, open_time, close_time, max_booking_hours, is_active) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, TRUE)")
                .bind(Uuid::new_v4().to_string()).bind(name).bind(rtype).bind(loc).bind(cap).bind(desc).bind(open).bind(close).bind(max_h)
                .execute(pool).await?;
        }
    }

    // Seed risk rules (need admin user to exist first)
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM risk_rules").fetch_one(pool).await?;
    if count.0 == 0 {
        let admin: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE role = 'admin' LIMIT 1")
            .fetch_optional(pool).await?;
        if let Some((admin_id,)) = admin {
            tracing::info!("Seeding default risk rules");
            let rules = [
                ("High Posting Frequency", "Detect >20 postings per employer in 24 hours", "posting_frequency", r#"{"max_postings": 20, "window_hours": 24}"#, "high"),
                ("Blacklisted Employer Posting", "Detect postings from blacklisted employers", "blacklisted_employer", "{}", "critical"),
                ("Abnormal Adjunct Compensation", "Detect adjunct compensation outside normal range", "abnormal_compensation", r#"{"min_amount": 500, "max_amount": 15000}"#, "high"),
                ("Suspected Duplicate Posting", "Detect duplicate internship/job postings", "duplicate_posting", r#"{"similarity_window_hours": 48}"#, "medium"),
            ];
            for (name, desc, rtype, cond, sev) in rules {
                sqlx::query("INSERT INTO risk_rules (uuid, name, description, rule_type, conditions, severity, is_active, created_by, schedule_interval_minutes) VALUES (?, ?, ?, ?, ?, ?, TRUE, ?, 15)")
                    .bind(Uuid::new_v4().to_string()).bind(name).bind(desc).bind(rtype).bind(cond).bind(sev).bind(admin_id)
                    .execute(pool).await?;
            }
        }
    }

    Ok(())
}
