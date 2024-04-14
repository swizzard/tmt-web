ALTER TABLE tabs ADD COLUMN created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now();
ALTER TABLE tabs ADD COLUMN modified_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now();

CREATE FUNCTION update_tabs_modified_at() RETURNS trigger AS $$
BEGIN
  NEW.modified_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_tabs_modified_at BEFORE UPDATE ON tabs
  FOR EACH ROW EXECUTE FUNCTION update_tabs_modified_at();
