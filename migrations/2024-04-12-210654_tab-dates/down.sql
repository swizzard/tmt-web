DROP TRIGGER update_tabs_modified_at;

DROP FUNCTION update_tabs_modified_at;

ALTER TABLE tabs
DROP COLUMN modified_at;

ALTER TABLE tabs
DROP COLUMN created_at;
