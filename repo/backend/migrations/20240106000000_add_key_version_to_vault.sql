-- Add key_version to sensitive_data_vault for encryption key rotation tracking.
--
-- key_version = 1 : value was encrypted with SHA256(jwt_secret) — the old insecure derivation
-- key_version = 2 : value was encrypted with the dedicated DATA_ENCRYPTION_KEY
--
-- Existing rows default to 1 so they remain identifiable as old-scheme entries.
-- New writes via the application will always store key_version = 2.
ALTER TABLE sensitive_data_vault
    ADD COLUMN key_version TINYINT UNSIGNED NOT NULL DEFAULT 1
        COMMENT '1=legacy SHA256(jwt_secret), 2=DATA_ENCRYPTION_KEY'
    AFTER iv;
