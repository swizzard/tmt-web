DROP FUNCTION check_user_pwd;
CREATE FUNCTION check_user_pwd(in_email text, pwd text) RETURNS boolean AS $$
  DECLARE
    us users;
  BEGIN
    SELECT u.* FROM users u WHERE u.email = in_email LIMIT 1 INTO us;
    RETURN eq_hashed(pwd, us.password);
  END;
$$ LANGUAGE plpgsql;
