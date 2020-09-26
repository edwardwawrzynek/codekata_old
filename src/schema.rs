table! {
    db_games (id) {
        id -> Integer,
        title -> Text,
        state -> Nullable<Text>,
        owner_id -> Integer,
        players -> Text,
        active -> Integer,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
        display_name -> Text,
        password_hash -> Text,
        api_key_hash -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(db_games, users,);
