use diesel::{deserialize::Queryable, Insertable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize, PartialEq)]
#[diesel(table_name = crate::schema::tabs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tab {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::tabs)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewTab {
    pub user_id: String,
    pub url: String,
    pub notes: Option<String>,
}
