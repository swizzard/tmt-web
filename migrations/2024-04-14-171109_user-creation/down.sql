DROP TABLE invites;

DROP TYPE invite_status;

ALTER TABLE users
DROP COLUMN confirmed;
