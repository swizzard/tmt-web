use super::tag::Tag;
use diesel::{
    associations::Associations, deserialize::Queryable, Identifiable, Insertable, Selectable,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Identifiable, Deserialize, Queryable, Selectable, Serialize, PartialEq)]
#[diesel(table_name = crate::schema::tabs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone))]
pub struct Tab {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub notes: Option<String>,
}

#[derive(Debug, Identifiable, Selectable, Queryable, Associations)]
#[diesel(belongs_to(Tab))]
#[diesel(belongs_to(Tag))]
#[diesel(table_name = crate::schema::tabs_tags)]
#[diesel(primary_key(tab_id, tag_id))]
pub struct TabTag {
    pub tab_id: String,
    pub tag_id: String,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::tabs)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct NewTab {
    pub user_id: String,
    pub url: String,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::tabs_tags)]
pub struct NewTabTag {
    pub tab_id: String,
    pub tag_id: String,
}

#[derive(Debug, Deserialize, Queryable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::tabs_tags)]
pub struct CreatedTabTag {
    pub tab_id: String,
    pub tag_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TabWithTags {
    pub tab: Tab,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttachTagRequest {
    pub user_id: String,
    pub tab_id: String,
    pub tag_id: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TagAttachedResponse {
    pub user_id: String,
    pub tab_id: String,
    pub tag_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DetachTagRequest {
    pub user_id: String,
    pub tab_id: String,
    pub tag_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagDetachedResponse {
    pub user_id: String,
    pub tab_id: String,
    pub tag_id: String,
}
