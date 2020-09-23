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
