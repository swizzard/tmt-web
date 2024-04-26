CREATE FUNCTION eq_hashed (val TEXT, hashed TEXT) RETURNS boolean AS
$$
  SELECT crypt(val, hashed) = hashed
$$ LANGUAGE SQL;

CREATE FUNCTION hash_pwd(pwd TEXT) RETURNS text AS
$$
  SELECT crypt(pwd, gen_salt('md5'))
$$ LANGUAGE SQL;
