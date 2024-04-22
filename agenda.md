# TMT Web

## curl

- [x] log in
- [x] log out
- [x] can see content when authorized via token
- [x] can't see content when not authorized via token
- [x] create user
  - [x] confirm user creation
- [x] can't login as unconfirmed user
- [x] delete user

## browser

- [ ] log in
- [ ] log out
- [ ] can see content when authorized via token
- [ ] can't see content when not authorized via 
- [ ] create user
  - [ ] confirm user creation
- [ ] can't login as unconfirmed user
- [ ] delete user

## misc

- [x] pg trigger to hash password on user insert
  - [x] also trigger on update iff password changed
- [x] tests (https://medium.com/intelliconnect-engineering/step-by-step-guide-to-test-driven-development-tdd-in-rust-axum-5bef05fd7366)
- [x] `users.state` column
- [x] nonexistent user login shouldn't return 500


## todo

### tabs

- [ ] all user tabs
  - [ ] pagination

### tags

- [ ] create tag
- [ ] add tag to tab
- [ ] remove tag from tab
- [ ] delete tag
- [ ] list user tags
  - [ ] pagination
- [ ] return tags with tab

### users
TK
