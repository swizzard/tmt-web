CREATE TABLE users (
  id TEXT PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
  email TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL
);

CREATE FUNCTION hash_user_pwd_on_create() RETURNS trigger AS $$
  DECLARE
    orig_pwd text;
  BEGIN
    orig_pwd := NEW.password;
    NEW.password := hash_pwd(orig_pwd);
    RETURN NEW;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hash_user_pwd_on_create BEFORE INSERT OR UPDATE ON users
  FOR EACH ROW EXECUTE FUNCTION hash_user_pwd_on_create();

CREATE FUNCTION check_user_pwd(email text, pwd text) RETURNS boolean AS $$
  SELECT eq_hashed(pwd, (SELECT u.password FROM users u WHERE u.email = email));
$$ LANGUAGE sql;
