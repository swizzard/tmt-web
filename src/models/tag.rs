use diesel::{deserialize::Queryable, Insertable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize, PartialEq)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone))]
pub struct Tag {
    pub id: String,
    pub user_id: String,
    pub tag: String,
}

#[derive(Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = crate::schema::tags)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewTag {
    pub user_id: String,
    pub tag: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MatchedTags {
    pub matches: Vec<Tag>,
}

impl MatchedTags {
    pub fn new(matches: Vec<Tag>) -> Self {
        Self { matches }
    }
}
