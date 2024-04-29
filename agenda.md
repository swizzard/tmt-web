# TMT Web

## todo

### endpoints

#### tabs

- [x] all user tabs
  - [x] pagination

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

- [ ] login
- [ ] sign up
- [ ] confirm sign up
- [ ] logout

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
- [ ] renew session token

## other

- [ ] deletes returning 0 should throw NotFound instead?
