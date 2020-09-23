use crate::schema::db_games;

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
