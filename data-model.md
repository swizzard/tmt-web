# Data Model

* User
  * id (uuid, unique)
  * email (text, unique)
  * password (text, hashed)
* Session
  * nonce (uuid)
  * user_id (uuid, references `user.id`, unique)
  * expires (timestamp)
* Invite
  * id (uuid, unique)
  * user_id (uuid, references `user.id`)
  * email (text)
  * status (`InviteStatus` enum)
  * expires (timestamp)
* Tab
  * id (uuid, unique)
  * user_id (uuid, references `user.id`)
  * url (text)
  * notes (text)
  * `has many` Tag
* Tag
  * id (uuid, unique)
  * user_id (uuid, referencess `user.id`)
  * tag (text)

## Notes

user logs in

* receive `{email, password}`

->

* `Session` created with `user_id`
* return `{user_id: user.id, nonce: session.nonce, expires: session.expires}`
  as JWT `claims`

user makes api request

* receive JWT

->

* decode JWT claims
* check db for `Session` matching user.id
  * if found
    * if expires >  now, delete `Session` and return 401
    * if nonce differs, delete `Session` and return 401
    * else, continue
  * if not found
    * return 401

 JWT -> claims (session id & user id) -> resources


user creation

POST /users `{email, password}` ->
  create `User`, `Invite`
  return `{email, invite_id, user_id}`
POST /users/invites/:invite_id ->
  changes `Invite.status` to `sent`
  (mocks scheduled task vel sim.)
POST /users/:user_id `{email, invite_id}`
  changes `User.confirmed` to `true`
