CREATE TABLE tabs_tags (
  ix BIGSERIAL PRIMARY KEY,
  tab_id TEXT NOT NULL REFERENCES tabs (id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX tabs_tags_unique_ix ON tabs_tags (tab_id, tag_id);
