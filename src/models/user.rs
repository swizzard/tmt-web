use diesel::{associations::Identifiable, deserialize::Queryable, prelude::Insertable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize, Identifiable, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub confirmed: bool,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewUser {
    pub email: String,
    pub password: String,
}

#[cfg(test)]
#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewConfirmedUser {
    pub email: String,
    pub password: String,
    pub confirmed: bool,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct CreatedUser {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct DeconfirmedUser {
    pub id: String,
    pub email: String,
}
