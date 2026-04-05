-- Add pending_unpublish to course status ENUM to support the unpublish approval workflow.
-- A course enters pending_unpublish when its owner submits an unpublish request.
-- It stays in this state until the 2-step approval is resolved (approved → unpublished,
-- rejected → published, or approved_scheduled → pending until ScheduledTransition fires).
ALTER TABLE courses MODIFY COLUMN status
    ENUM('draft', 'pending_approval', 'approved_scheduled', 'published', 'pending_unpublish', 'unpublished', 'rejected')
    NOT NULL DEFAULT 'draft';
