-- Phase 3: Booking engine, Risk engine, Webhooks, Privacy, Security hardening

-- ============================================================
-- BOOKING ENHANCEMENTS
-- ============================================================

-- Add resource operating hours and maintenance support
ALTER TABLE resources ADD COLUMN open_time TIME NOT NULL DEFAULT '07:00:00' AFTER description;
ALTER TABLE resources ADD COLUMN close_time TIME NOT NULL DEFAULT '22:00:00' AFTER open_time;
ALTER TABLE resources ADD COLUMN max_booking_hours INT NOT NULL DEFAULT 4 AFTER close_time;
ALTER TABLE resources ADD COLUMN requires_approval BOOLEAN NOT NULL DEFAULT FALSE AFTER max_booking_hours;

-- Resource maintenance blackouts
CREATE TABLE IF NOT EXISTS resource_blackouts (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    resource_id BIGINT NOT NULL,
    reason VARCHAR(500) NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    created_by BIGINT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_blackout_resource_time (resource_id, start_time, end_time)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Enhance bookings for reschedule tracking and breach generation
ALTER TABLE bookings ADD COLUMN reschedule_count INT NOT NULL DEFAULT 0 AFTER recurrence_rule;
ALTER TABLE bookings ADD COLUMN approved_by BIGINT AFTER reschedule_count;
ALTER TABLE bookings ADD COLUMN approved_at DATETIME NULL AFTER approved_by;
ALTER TABLE bookings ADD FOREIGN KEY (approved_by) REFERENCES users(id) ON DELETE SET NULL;

-- Enhance booking_reschedules
ALTER TABLE booking_reschedules ADD COLUMN reschedule_number INT NOT NULL DEFAULT 1 AFTER booking_id;

-- Enhance breaches for booking domain
ALTER TABLE breaches ADD COLUMN booking_id BIGINT AFTER user_id;
ALTER TABLE breaches ADD FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE SET NULL;

-- Enhance restrictions for automatic trigger tracking
ALTER TABLE restrictions ADD COLUMN breach_count INT NOT NULL DEFAULT 0 AFTER is_active;
ALTER TABLE restrictions ADD COLUMN auto_triggered BOOLEAN NOT NULL DEFAULT FALSE AFTER breach_count;

-- ============================================================
-- RISK / ANOMALY ENGINE ENHANCEMENTS
-- ============================================================

-- Enhance risk_rules with scheduling and thresholds
ALTER TABLE risk_rules ADD COLUMN schedule_interval_minutes INT NOT NULL DEFAULT 15 AFTER is_active;
ALTER TABLE risk_rules ADD COLUMN last_run_at DATETIME NULL AFTER schedule_interval_minutes;

-- Enhance risk_events for review workflow
ALTER TABLE risk_events ADD COLUMN reviewed_by BIGINT AFTER status;
ALTER TABLE risk_events ADD COLUMN reviewed_at DATETIME NULL AFTER reviewed_by;
ALTER TABLE risk_events ADD COLUMN escalated_to BIGINT AFTER reviewed_at;
ALTER TABLE risk_events ADD COLUMN notes TEXT AFTER escalated_to;
ALTER TABLE risk_events ADD FOREIGN KEY (reviewed_by) REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE risk_events ADD FOREIGN KEY (escalated_to) REFERENCES users(id) ON DELETE SET NULL;

-- Blacklisted employers
CREATE TABLE IF NOT EXISTS blacklisted_employers (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    employer_name VARCHAR(500) NOT NULL,
    reason TEXT NOT NULL,
    added_by BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_blacklist_name (employer_name),
    INDEX idx_blacklist_active (is_active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Employer postings tracking (for frequency anomaly detection)
CREATE TABLE IF NOT EXISTS employer_postings (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    employer_name VARCHAR(500) NOT NULL,
    posting_type ENUM('internship', 'job', 'adjunct') NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    compensation DOUBLE,
    posted_by BIGINT NOT NULL,
    flagged BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (posted_by) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_postings_employer (employer_name),
    INDEX idx_postings_created (created_at),
    INDEX idx_postings_type (posting_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- WEBHOOK ENHANCEMENTS
-- ============================================================

-- Enhance webhook_queue with signing and retry backoff
ALTER TABLE webhook_queue ADD COLUMN signature VARCHAR(255) AFTER response_body;
ALTER TABLE webhook_queue ADD COLUMN is_onprem BOOLEAN NOT NULL DEFAULT TRUE AFTER signature;

-- ============================================================
-- PRIVACY / SENSITIVE DATA
-- ============================================================

-- Sensitive data vault for encrypted fields
CREATE TABLE IF NOT EXISTS sensitive_data_vault (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    user_id BIGINT NOT NULL,
    field_name VARCHAR(100) NOT NULL,
    encrypted_value TEXT NOT NULL,
    iv VARCHAR(64) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY uk_user_field (user_id, field_name),
    INDEX idx_vault_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Enhance personal_data_requests with admin approval workflow
ALTER TABLE personal_data_requests ADD COLUMN approved_by BIGINT AFTER processed_by;
ALTER TABLE personal_data_requests ADD COLUMN approved_at DATETIME NULL AFTER approved_by;
ALTER TABLE personal_data_requests ADD COLUMN admin_notes TEXT AFTER approved_at;
ALTER TABLE personal_data_requests ADD FOREIGN KEY (approved_by) REFERENCES users(id) ON DELETE SET NULL;

-- Seed data (resources, risk rules) is handled by the application seed service
-- to avoid foreign key issues with user references
