CREATE TABLE sessions (
  id TEXT PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
  user_id TEXT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
  expires TIMESTAMP WITHOUT TIME ZONE NOT NULL default now() + '15 minutes'::interval
);
