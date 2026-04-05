-- Add per-subscription webhook endpoint and optional signing secret.
-- target_url: required when channel = 'webhook'; NULL for in_app/email channels.
-- signing_secret: optional HMAC-SHA256 key sent as X-Webhook-Signature header;
--   NULL means no signature header is attached to outbound webhook deliveries.
ALTER TABLE subscriptions
    ADD COLUMN target_url VARCHAR(1000) NULL
        COMMENT 'On-prem webhook endpoint; required when channel = webhook'
    AFTER channel,
    ADD COLUMN signing_secret VARCHAR(255) NULL
        COMMENT 'Per-subscription HMAC-SHA256 signing secret; NULL = no signature'
    AFTER target_url;
