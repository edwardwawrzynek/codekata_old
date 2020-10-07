table! {
    db_games (id) {
        id -> Int4,
        title -> Varchar,
        state -> Nullable<Text>,
        owner_id -> Int4,
        players -> Varchar,
        active -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        display_name -> Text,
        password_hash -> Text,
        api_key_hash -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    db_games,
    users,
);
