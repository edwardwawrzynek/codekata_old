use crate::schema::db_games;
use crate::schema::users;

#[derive(Queryable, Debug)]
pub struct DbGame {
    pub id: i32,
    pub title: String,
    pub state: Option<String>,
    pub owner_id: i32,
    pub players: String,
    pub active: i32,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "db_games"]
pub struct InsertDbGame<'a> {
    pub id: i32,
    pub title: &'a str,
    pub state: Option<String>,
    pub owner_id: i32,
    pub players: String,
    pub active: i32,
}

#[derive(Insertable)]
#[table_name = "db_games"]
pub struct NewDbGame<'a> {
    pub title: &'a str,
    pub state: Option<String>,
    pub owner_id: i32,
    pub players: String,
    pub active: i32,
}

#[derive(Queryable)]
pub struct DBGameId {
    pub id: i32,
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub api_key_hash: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub display_name: &'a str,
    pub password_hash: &'a str,
    pub api_key_hash: Option<&'a str>,
}
