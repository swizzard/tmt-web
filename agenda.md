# TMT Web

## curl

[x] log in
[x] log out
[x] can see content when authorized via token
[x] can't see content when not authorized via token
[ ] create user
  [ ] confirm user creation
[ ] can't login as unconfirmed user
[ ] delete user

## browser

[ ] log in
[ ] log out
[ ] can see content when authorized via token
[ ] can't see content when not authorized via 
[ ] create user
  [ ] confirm user creation
[ ] can't login as unconfirmed user
[ ] delete user

## misc

[x] pg trigger to hash password on user insert
  [x] also trigger on update iff password changed
[ ] tests (https://medium.com/intelliconnect-engineering/step-by-step-guide-to-test-driven-development-tdd-in-rust-axum-5bef05fd7366)
[x] `users.state` column
[ ] nonexistent user login shouldn't return 500
