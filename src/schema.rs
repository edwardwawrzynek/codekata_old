table! {
    db_games (id) {
        id -> Int4,
        title -> Varchar,
        state -> Nullable<Text>,
        owner_id -> Int4,
        players -> Varchar,
        active -> Int4,
        is_public -> Bool,
    }
}

table! {
    pages (id) {
        id -> Int4,
        url -> Text,
        content -> Text,
    }
}

table! {
    tournaments (id) {
        id -> Int4,
        name -> Text,
        players -> Array<Int4>,
        games -> Nullable<Array<Int4>>,
        owner_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        display_name -> Text,
        password_hash -> Text,
        api_key_hash -> Nullable<Text>,
        is_admin -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(db_games, pages, tournaments, users,);
