use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sessions)]
pub struct NewSession {
    pub user_id: String,
}
