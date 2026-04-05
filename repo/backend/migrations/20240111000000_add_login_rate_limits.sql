-- IP-based rate limiting for unauthenticated endpoints (login, etc.)
-- and account lockout after repeated failed login attempts.
CREATE TABLE IF NOT EXISTS ip_rate_limits (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    ip_address VARCHAR(45) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    window_start DATETIME NOT NULL,
    request_count INT NOT NULL DEFAULT 1,
    INDEX idx_ip_rate (ip_address, endpoint, window_start)
) ENGINE=InnoDB;

ALTER TABLE users ADD COLUMN failed_login_count INT NOT NULL DEFAULT 0 AFTER last_reauth_at;
ALTER TABLE users ADD COLUMN locked_until DATETIME NULL AFTER failed_login_count;
