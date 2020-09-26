#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::collections::HashMap;
use std::sync::RwLock;
use std::fmt::Debug;

pub mod game;
pub mod game_manage;
pub mod models;
pub mod schema;
pub mod shared;
pub mod users;

use game::{Game, GamePlayer};

pub mod tic_tac_toe;

#[derive(Clone, Debug)]
pub struct SimpleGame();

impl Game for SimpleGame {
    type Move = ();
    type Score = i32;
    type State = ();

    /// Check if a game can be created with the number of players
    fn check_num_players(players: usize) -> bool {
        true
    }
    /// Create an instance of the game with the given number of players
    fn new_with_players(players: usize) -> Self {
        SimpleGame()
    }

    fn from_state(state: (), players: usize) -> Self {
        SimpleGame()
    }

    fn state(&self) -> () {
        ()
    }

    /// Return the state of the game (a game starts as Running and becomes Finished)
    fn finished(&self) -> bool {
        true
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

fn main() {
    rocket::ignite()
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
                users::user_new,
                users::user_get,
                users::user_edit,
                users::session_new,
                users::session_delete,
                users::user_generate_api_key
            ],
        )
        .register(catchers![users::unauthorized])
        .launch();
}
