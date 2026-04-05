-- Add pending_scan status for media validation state machine
ALTER TABLE media_assets MODIFY COLUMN status ENUM('uploading', 'processing', 'ready', 'failed', 'pending_scan') NOT NULL DEFAULT 'uploading';
