CREATE TABLE tabs (
  id TEXT PRIMARY KEY NOT NULL DEFAULT gen_random_uuid (),
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  url TEXT NOT NULL,
  notes TEXT
);
