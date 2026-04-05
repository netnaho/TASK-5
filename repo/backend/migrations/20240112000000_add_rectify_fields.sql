-- Support field-level rectification in privacy data requests.
-- When request_type='rectify', field_name and new_value specify the target update.
ALTER TABLE personal_data_requests ADD COLUMN field_name VARCHAR(100) NULL AFTER result_file_path;
ALTER TABLE personal_data_requests ADD COLUMN new_value VARCHAR(1000) NULL AFTER field_name;
