#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLock;

pub mod game;
pub mod game_manage;
pub mod models;
pub mod schema;
pub mod shared;
pub mod users;

use game::{Game, GamePlayer};
use std::path::{PathBuf, Path};
use rocket::response::NamedFile;

pub mod tic_tac_toe;

#[derive(Clone, Debug)]
pub struct SimpleGame();

impl Game for SimpleGame {
    type Move = ();
    type Score = i32;
    type State = i32;

    /// Check if a game can be created with the number of players
    fn check_num_players(players: usize) -> bool {
        players == 2
    }
    /// Create an instance of the game with the given number of players
    fn new_with_players(players: usize) -> Self {
        SimpleGame()
    }

    fn from_state(state: i32, players: usize) -> Self {
        SimpleGame()
    }

    fn state(&self) -> i32 {
        42
    }

    /// Return the state of the game (a game starts as Running and becomes Finished)
    fn finished(&self) -> bool {
        false
    }
    /// Check if the game is waiting on a move by the given player
    fn waiting_on(&self, player: GamePlayer) -> bool {
        false
    }
    /// Make a move for the given player. If the move is legal, make it and return true. If not, return false.
    fn make_move(&mut self, player: GamePlayer, move_to_make: Self::Move) -> bool {
        true
    }
    /// Get the score for each player. If scores are not available at the current point in the game, return None.
    fn scores(&self) -> Option<Vec<Self::Score>> {
        Some(vec![0])
    }
}

/// route to serve frontend
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

    rocket::ignite()
        .attach(cors)
        .attach(shared::DBConn::fairing())
        .manage(RwLock::new(
            game_manage::GameManager::<SimpleGame>::default(),
        ))
        .manage(RwLock::new(HashMap::<String, users::PlayerId>::new()))
        .mount(
            "/api",
            routes![
                game_manage::game_get,
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
        ).mount("/", routes![frontend_route, frontend_root])
        .register(catchers![users::unauthorized])
        .launch();
}
