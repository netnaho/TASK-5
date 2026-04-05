-- Add department ownership to resources for scoped booking approval.
-- Reviewers can only approve/reject bookings for resources in their department.
ALTER TABLE resources ADD COLUMN department_id BIGINT NULL AFTER requires_approval;
ALTER TABLE resources ADD CONSTRAINT fk_resources_dept FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE SET NULL;
ALTER TABLE resources ADD INDEX idx_resources_department (department_id);
