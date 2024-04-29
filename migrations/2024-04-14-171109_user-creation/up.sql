ALTER TABLE users
ADD COLUMN confirmed boolean NOT NULL DEFAULT false;

CREATE TYPE invite_status AS ENUM ('created', 'sent', 'accepted', 'expired');

CREATE TABLE invites (
  id TEXT PRIMARY KEY NOT NULL DEFAULT gen_random_uuid (),
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  email TEXT NOT NULL,
  status invite_status NOT NULL DEFAULT 'created',
  expires TIMESTAMP WITHOUT TIME ZONE NOT NULL default now() + '15 minutes'::interval
);
