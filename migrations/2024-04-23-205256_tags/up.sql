CREATE TABLE tags (
  id TEXT PRIMARY KEY NOT NULL DEFAULT gen_random_uuid (),
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  tag TEXT NOT NULL
);

CREATE UNIQUE INDEX tags_user_id_tag ON tags (user_id, tag);
