use crate::schema::db_games;
use crate::schema::pages;
use crate::schema::tournaments;
use crate::schema::users;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug)]
pub struct DbGame {
    pub id: i32,
    pub title: String,
    pub state: Option<String>,
    pub owner_id: i32,
    pub players: String,
    pub active: i32,
    pub is_public: bool,
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
    pub is_public: bool,
}

#[derive(Insertable)]
#[table_name = "db_games"]
pub struct NewDbGame<'a> {
    pub title: &'a str,
    pub state: Option<String>,
    pub owner_id: i32,
    pub players: String,
    pub active: i32,
    pub is_public: bool,
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub api_key_hash: Option<String>,
    pub is_admin: bool,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub display_name: &'a str,
    pub password_hash: &'a str,
    pub api_key_hash: Option<&'a str>,
    pub is_admin: bool,
}

#[derive(Queryable, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "tournaments"]
pub struct Tournament {
    pub id: i32,
    pub name: String,
    pub players: Vec<i32>,
    pub games: Option<Vec<i32>>,
    pub owner_id: i32,
}

#[derive(Insertable)]
#[table_name = "tournaments"]
pub struct NewTournament<'a> {
    pub name: &'a str,
    pub players: Vec<i32>,
    pub games: Option<Vec<i32>>,
    pub owner_id: i32,
}

#[derive(Queryable, Insertable, AsChangeset, Clone, Debug, FromForm, Serialize)]
#[table_name = "pages"]
pub struct Page {
    pub id: i32,
    pub url: String,
    pub content: String,
}

#[derive(Insertable)]
#[table_name = "pages"]
pub struct NewPage<'a> {
    pub url: &'a str,
    pub content: &'a str,
}
