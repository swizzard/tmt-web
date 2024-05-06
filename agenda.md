# TMT Web

## todo

### endpoints

#### tabs

- [x] all user tabs
  - [x] pagination
- [x] edit tab
  - top-level fields
  - add/remove/create tags

#### tags

- [x] create tag
- [x] add tag to tab
- [x] remove tag from tab
- [x] delete tag
- [x] list user tags

  - [x] pagination
  - [x] fuzzy matching?

- [x] return tags with tab
- [x] create tab with tags

#### users

- [ ] confirm user via `GET` + query param, redirect?

TK

### pages

#### users

- [x] login
- [ ] sign up
- [ ] confirm sign up
- [x] logout

#### tabs

- [ ] list user tabs
  - pagination
  - delete tab
- [ ] create tab
  - create new tags
  - retrieve existing tags
- [ ] update tab

#### tags

- [ ] list user tags
  - pagination
  - delete tag
- [ ] create tag

#### misc

- [ ] home page ?

## upcoming

- [ ] forgotten/change password flow
- [ ] tab `title: Option<String>`
- [ ] search for tab
- [ ] list tabs given tag
- [x] renew session token

## other

- [x] deletes returning 0 should throw NotFound instead?
