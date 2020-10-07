use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use diesel_migrations::embed_migrations;

// embed diesel migrations
embed_migrations!("./migrations");

fn open_db() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn run_migrations() {
    let conn = open_db();
    embedded_migrations::run(&conn);
}