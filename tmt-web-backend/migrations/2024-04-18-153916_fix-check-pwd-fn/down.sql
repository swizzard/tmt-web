DROP FUNCTION check_user_pwd;
-- from 2024-04-11-162738_user
CREATE FUNCTION check_user_pwd(email text, pwd text) RETURNS boolean AS $$
  SELECT eq_hashed(pwd, (SELECT u.password FROM users u WHERE u.email = email));
$$ LANGUAGE sql;
