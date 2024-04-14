// @generated automatically by Diesel CLI.

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
    }
}

diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(tabs -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    tabs,
    users,
);
