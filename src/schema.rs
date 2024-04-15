// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "invite_status"))]
    pub struct InviteStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::InviteStatus;

    invites (id) {
        id -> Text,
        user_id -> Text,
        email -> Text,
        status -> InviteStatus,
        expires -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        user_id -> Text,
        expires -> Timestamp,
    }
}

diesel::table! {
    tabs (id) {
        id -> Text,
        user_id -> Text,
        url -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        modified_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        password -> Text,
        confirmed -> Bool,
    }
}

diesel::joinable!(invites -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(tabs -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    invites,
    sessions,
    tabs,
    users,
);
