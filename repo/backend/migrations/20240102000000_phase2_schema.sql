-- Phase 2: Enhanced schema for all business domains
-- Note: Role ENUMs are already correct from initial migration (admin, staff_author, dept_reviewer, faculty, student, integration)

-- Rate limiting
CREATE TABLE IF NOT EXISTS rate_limit_entries (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    window_start DATETIME NOT NULL,
    request_count INT NOT NULL DEFAULT 1,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_rate_user_window (user_id, window_start)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- HMAC integration keys
CREATE TABLE IF NOT EXISTS hmac_keys (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    key_id VARCHAR(100) NOT NULL UNIQUE,
    secret_hash VARCHAR(255) NOT NULL,
    description VARCHAR(500),
    owner_user_id BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NULL,
    FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_hmac_key_id (key_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Nonce anti-replay
CREATE TABLE IF NOT EXISTS used_nonces (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    nonce VARCHAR(100) NOT NULL UNIQUE,
    key_id VARCHAR(100) NOT NULL,
    used_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    INDEX idx_nonce_expires (expires_at),
    INDEX idx_nonce_value (nonce)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Re-auth tracking
ALTER TABLE users ADD COLUMN last_reauth_at DATETIME NULL AFTER last_login_at;

-- Update courses status enum for full workflow
ALTER TABLE courses MODIFY COLUMN status ENUM('draft', 'pending_approval', 'approved_scheduled', 'published', 'unpublished', 'rejected') NOT NULL DEFAULT 'draft';

-- Add missing course fields
ALTER TABLE courses ADD COLUMN release_notes TEXT AFTER max_enrollment;
ALTER TABLE courses ADD COLUMN effective_date DATETIME NULL AFTER release_notes;
ALTER TABLE courses ADD COLUMN updated_on DATETIME NULL AFTER effective_date;
ALTER TABLE courses ADD COLUMN current_version INT NOT NULL DEFAULT 0 AFTER updated_on;
ALTER TABLE courses ADD COLUMN created_by BIGINT AFTER current_version;
ALTER TABLE courses ADD FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- Rich text content support for lessons (already has LONGTEXT content_body)
-- Add content_html for rendered version
ALTER TABLE lessons ADD COLUMN content_html LONGTEXT AFTER content_body;

-- Tags system
CREATE TABLE IF NOT EXISTS tags (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_tags_slug (slug)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS course_tags (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    course_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    UNIQUE KEY uk_course_tag (course_id, tag_id),
    INDEX idx_course_tags_course (course_id),
    INDEX idx_course_tags_tag (tag_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Update approval_requests for richer workflow
ALTER TABLE approval_requests ADD COLUMN release_notes TEXT AFTER notes;
ALTER TABLE approval_requests ADD COLUMN effective_date DATETIME NULL AFTER release_notes;
ALTER TABLE approval_requests ADD COLUMN version_number INT AFTER effective_date;
ALTER TABLE approval_requests MODIFY COLUMN status ENUM('pending_step1', 'pending_step2', 'approved', 'approved_scheduled', 'rejected', 'cancelled') NOT NULL DEFAULT 'pending_step1';

-- Scheduled transitions for approved_scheduled courses
CREATE TABLE IF NOT EXISTS scheduled_transitions (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    course_id BIGINT NOT NULL,
    approval_request_id BIGINT NOT NULL,
    target_status ENUM('published', 'unpublished') NOT NULL,
    scheduled_at DATETIME NOT NULL,
    executed_at DATETIME NULL,
    is_executed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (approval_request_id) REFERENCES approval_requests(id) ON DELETE CASCADE,
    INDEX idx_scheduled_pending (is_executed, scheduled_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Enhance audit_logs with correlation_id and retention metadata
ALTER TABLE audit_logs ADD COLUMN correlation_id VARCHAR(36) AFTER user_agent;
ALTER TABLE audit_logs ADD COLUMN retention_expires_at DATETIME NULL AFTER correlation_id;
ALTER TABLE audit_logs ADD INDEX idx_audit_correlation (correlation_id);
ALTER TABLE audit_logs ADD INDEX idx_audit_retention (retention_expires_at);

-- Enhance security_events
ALTER TABLE security_events ADD COLUMN correlation_id VARCHAR(36) AFTER metadata;
ALTER TABLE security_events ADD INDEX idx_security_correlation (correlation_id);

-- Media validation fields
ALTER TABLE media_assets ADD COLUMN validated BOOLEAN NOT NULL DEFAULT FALSE AFTER status;
ALTER TABLE media_assets ADD COLUMN validation_error VARCHAR(500) AFTER validated;

-- Course version retention tracking
ALTER TABLE course_versions ADD COLUMN expires_at DATETIME NULL AFTER created_at;
ALTER TABLE course_versions ADD INDEX idx_versions_expires (expires_at);

-- Rebuild permissions for new roles
DELETE FROM permissions;
INSERT INTO permissions (role, resource, action) VALUES
    ('admin', 'users', 'manage'),
    ('admin', 'courses', 'manage'),
    ('admin', 'bookings', 'manage'),
    ('admin', 'approvals', 'manage'),
    ('admin', 'compliance', 'manage'),
    ('admin', 'audit', 'read'),
    ('admin', 'tags', 'manage'),
    ('admin', 'hmac_keys', 'manage'),
    ('staff_author', 'courses', 'create'),
    ('staff_author', 'courses', 'read'),
    ('staff_author', 'courses', 'update'),
    ('staff_author', 'courses', 'delete'),
    ('staff_author', 'media', 'create'),
    ('staff_author', 'media', 'read'),
    ('staff_author', 'tags', 'create'),
    ('staff_author', 'tags', 'read'),
    ('staff_author', 'approvals', 'create'),
    ('staff_author', 'approvals', 'read'),
    ('dept_reviewer', 'courses', 'read'),
    ('dept_reviewer', 'approvals', 'read'),
    ('dept_reviewer', 'approvals', 'approve'),
    ('dept_reviewer', 'tags', 'read'),
    ('faculty', 'courses', 'read'),
    ('faculty', 'bookings', 'create'),
    ('faculty', 'bookings', 'read'),
    ('student', 'courses', 'read'),
    ('integration', 'courses', 'read'),
    ('integration', 'approvals', 'read');
