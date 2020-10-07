#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

extern crate dotenv;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::collections::HashMap;
use std::sync::RwLock;

pub mod game;
pub mod game_manage;
pub mod models;
pub mod schema;
pub mod shared;
pub mod users;
pub mod run_migrations;

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

pub mod gomoku;
use gomoku::Gomoku;

pub type GameType = Gomoku;

/// routes to serve frontend
#[get("/", rank = 9)]
fn frontend_root() -> Option<NamedFile> {
    NamedFile::open(Path::new("frontend/build/index.html")).ok()
}

#[get("/<file..>", rank = 10)]
fn frontend_route(file: PathBuf) -> Option<NamedFile> {
    let file = NamedFile::open(Path::new("frontend/build/").join(file));

    match file {
        Ok(file) => Some(file),
        Err(_) => {
            // serve index.html
            NamedFile::open(Path::new("frontend/build/index.html")).ok()
        }
    }
}

fn main() {
    // run db migrations
    run_migrations::run_migrations();
    // setup cors
    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Get, Method::Post, Method::Put]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    // start app
    rocket::ignite()
        .attach(cors)
        .attach(shared::DBConn::fairing())
        .manage(RwLock::new(game_manage::GameManager::<GameType>::default()))
        .manage(RwLock::new(HashMap::<String, users::PlayerId>::new()))
        .mount(
            "/api",
            routes![
                game_manage::game_get_user_authd,
                game_manage::game_get,
                game_manage::game_move_needed,
                game_manage::game_move,
                game_manage::game_new,
                game_manage::game_join,
                game_manage::game_leave,
                game_manage::game_start,
                game_manage::game_index,
                users::user_new,
                users::user_get,
                users::user_edit,
                users::session_new,
                users::session_delete,
                users::user_generate_api_key
            ],
        )
        .mount("/", routes![frontend_route, frontend_root])
        .register(catchers![users::unauthorized])
        .launch();
}
