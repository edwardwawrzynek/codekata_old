#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate dotenv;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{self, Form, FromRequest, Request};
use rocket::response::Redirect;
use rocket::{Outcome, State};
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use crate::models::{DbGame, InsertDbGame, NewDbGame};
use serde::de::DeserializeOwned;
use serde::export::fmt::Display;
use serde::export::TryFrom;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::sync::{RwLock, RwLockWriteGuard};

use diesel::prelude::*;
use std::fmt::Debug;

pub mod models;
pub mod schema;
pub mod users;
use crate::users::{ApiKey, HashedApiKey, PlayerId};

pub mod tic_tac_toe;

pub type GamePlayer = u32;

/// Some type of game. It is expected to be turn based, and eventually reach an end state.
pub trait Game: Clone + Debug {
    type Move: Serialize;
    type Score: Add + Serialize + Display;
    type State: Serialize + DeserializeOwned;

    /// Check if a game can be created with the number of players
    fn check_num_players(players: usize) -> bool;
    /// Create an instance of the game with the given number of players
    fn new_with_players(players: usize) -> Self;
    /// Create an instance of the game from the given state and number of players
    fn from_state(state: Self::State, players: usize) -> Self;

    /// Get the serializable state of the game
    fn state(&self) -> Self::State;
    /// Return the state of the game (a game starts as Running and becomes Finished)
    fn finished(&self) -> bool;
    /// Check if the game is waiting on a move by the given player
    fn waiting_on(&self, player: GamePlayer) -> bool;
    /// Make a move for the given player. If the move is legal, make it and return true. If not, return false.
    fn make_move(&mut self, player: GamePlayer, move_to_make: Self::Move) -> bool;
    /// Get the score for each player. If scores are not available at the current point in the game, return None.
    fn scores(&self) -> Option<Vec<Self::Score>>;
}

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

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Default, Debug)]
struct GameId(i32);

impl GameId {
    fn next(&self) -> GameId {
        GameId(self.0 + 1)
    }
}

impl ToString for GameId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Clone, Debug)]
struct GameInstance<G: Game> {
    /// If the game has not yet started, game is None
    game: Option<Box<G>>,
    /// Players currently in the game
    players: Vec<PlayerId>,
    name: String,
    owner: PlayerId,
    id: GameId,
}

impl<G: Game> GameInstance<G> {
    fn new(name: String, owner: PlayerId, id: GameId) -> GameInstance<G> {
        return GameInstance {
            id,
            name,
            owner,
            game: None,
            players: Vec::new(),
        };
    }
}

impl<G: Game> GameInstance<G> {
    fn active(&self) -> bool {
        match &self.game {
            None => false,
            Some(g) => !g.finished(),
        }
    }
}

impl<G: Game> TryFrom<DbGame> for GameInstance<G> {
    type Error = serde_json::Error;
    fn try_from(entry: DbGame) -> Result<GameInstance<G>, Self::Error> {
        let players = serde_json::from_str::<Vec<i32>>(&entry.players)?
            .iter()
            .map(|id| PlayerId::new(*id))
            .collect::<Vec<PlayerId>>();
        let game = match entry.state {
            Some(s) => Some(Box::new(G::from_state(
                serde_json::from_str::<G::State>(&s)?,
                players.len(),
            ))),
            None => None,
        };
        Ok(GameInstance {
            id: GameId(entry.id),
            game,
            players,
            name: entry.title,
            owner: PlayerId::new(entry.owner_id),
        })
    }
}

impl<'a, G: Game> From<&'a GameInstance<G>> for InsertDbGame<'a> {
    fn from(inst: &GameInstance<G>) -> InsertDbGame {
        let state = match &inst.game {
            Some(g) => serde_json::to_string(&g.state()).ok(),
            None => None,
        };

        let players =
            serde_json::to_string(&inst.players.iter().map(|id| id.id()).collect::<Vec<i32>>())
                .unwrap_or("[]".to_string());
        InsertDbGame {
            id: inst.id.0,
            title: &inst.name,
            state,
            owner_id: inst.owner.id(),
            players,
            active: if inst.active() { 1 } else { 0 },
        }
    }
}

#[database("db")]
pub struct DBConn(diesel::SqliteConnection);

struct GameManager<G: Game> {
    active_games: HashMap<GameId, GameInstance<G>>,
}

impl<G: Game> Default for GameManager<G> {
    fn default() -> GameManager<G> {
        GameManager {
            active_games: HashMap::new(),
        }
    }
}

struct AppState<'a, G: Game> {
    manager: &'a RwLock<GameManager<G>>,
    db: DBConn,
}

#[derive(Debug)]
pub enum Error {
    InvalidGameId,
    DBError(diesel::result::Error),
    SerializeError(serde_json::Error),
    MalformedApiKey,
    UsernameAlreadyTaken,
    HashError(bcrypt::BcryptError),
    NoSuchUser,
    InvalidPassword,
    Unauthorized,
    NoAuthorizationMethod,
    InvalidApiKey,
    GuardLoadError,
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::SerializeError(e)
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(e: bcrypt::BcryptError) -> Error {
        Error::HashError(e)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        Error::DBError(e)
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResp {
    error: String,
}

impl From<Error> for ErrorResp {
    fn from(err: Error) -> ErrorResp {
        ErrorResp {
            error: match err {
                Error::DBError(e) => format!("database error: {}", e.to_string()),
                Error::SerializeError(e) => format!("data serialization error: {}", e.to_string()),
                Error::InvalidGameId => "invalid game id".to_string(),
                Error::MalformedApiKey => "malformed api key".to_string(),
                Error::UsernameAlreadyTaken => "username already taken".to_string(),
                Error::HashError(e) => format!("bcrypt hashing error: {}", e.to_string()),
                Error::NoSuchUser => "invalid username".to_string(),
                Error::InvalidPassword => "invalid password".to_string(),
                Error::Unauthorized => "unauthorized".to_string(),
                Error::InvalidApiKey => "invalid api key".to_string(),
                Error::NoAuthorizationMethod => "no authorization method".to_string(),
                Error::GuardLoadError => "error loading a request guard".to_string(),
            },
        }
    }
}

impl From<Error> for rocket_contrib::json::Json<ErrorResp> {
    fn from(e: Error) -> Self {
        Json(ErrorResp::from(e))
    }
}

impl<G: Game> AppState<'_, G> {
    /// load a game from the database (only, not active_games)
    fn load_game_from_db(&self, game_id: GameId) -> Result<GameInstance<G>, Error> {
        use crate::schema::db_games;

        db_games::dsl::db_games
            .find(&game_id.0)
            .first::<DbGame>(&*self.db)
            .map_or_else(
                |err| Err(Error::DBError(err)),
                |entry| Ok(GameInstance::<G>::try_from(entry)?),
            )
    }

    /// save a game to the database from active_games
    fn save_game_to_db<'l>(
        &self,
        game_id: GameId,
        manager_lock: RwLockWriteGuard<'l, GameManager<G>>,
    ) -> Result<RwLockWriteGuard<'l, GameManager<G>>, Error> {
        use crate::schema::db_games;

        let game = manager_lock.active_games.get(&game_id);
        let new_entry = match game {
            Some(game) => InsertDbGame::from(game),
            None => return Err(Error::InvalidGameId),
        };

        let res = diesel::update(db_games::table)
            .set(&new_entry)
            .execute(&*self.db)
            .map_or_else(|e| Err(Error::DBError(e)), |r| Ok(manager_lock));

        res
    }

    /// create a new game entry in the db and in active_games
    fn new_game(&self, name: &str, owner: PlayerId) -> Result<GameId, Error> {
        use crate::schema::db_games;

        let game = NewDbGame {
            players: "[]".to_string(),
            active: 1,
            owner_id: owner.id(),
            title: name,
            state: None,
        };

        // in order to get the assigned id, we do an INSERT than SELECT for highest id (sqlite doesn't support RETURNING)
        let inserted_games = self.db.transaction::<_, diesel::result::Error, _>(|| {
            let insert_count = diesel::insert_into(db_games::table)
                .values(&game)
                .execute(&*self.db)?;
            assert_eq!(insert_count, 1);

            Ok(db_games::dsl::db_games
                .order(db_games::id.desc())
                .limit(insert_count as i64)
                .load(&*self.db)?
                .into_iter()
                .collect::<Vec<DbGame>>())
        })?;

        let id = GameId(inserted_games[0].id);

        let mut manager = self.manager.write().unwrap();
        manager.active_games.insert(
            id,
            GameInstance::<G> {
                game: None,
                players: vec![],
                name: name.to_string(),
                owner,
                id,
            },
        );

        Ok(id)
    }

    /// get the game with the given id.
    /// possibly loads it from the database/cache, and may remove or insert it into the cache
    fn get_game(&self, game_id: GameId) -> Result<GameInstance<G>, Error> {
        // check active_games for cached game
        let mut manager = self.manager.write().unwrap();
        let cached = manager.active_games.get(&game_id);
        match cached {
            Some(game) => {
                let res = game.clone();
                // if game isn't active, remove from active games
                if !res.active() {
                    let mut manager = self.save_game_to_db(game_id, manager)?;
                    manager.active_games.remove(&game_id);
                }

                Ok(res)
            }
            None => {
                let game = self.load_game_from_db(game_id)?;
                // if the game isn't finished, put it into active_games
                if game.active() {
                    manager.active_games.insert(game_id, game.clone());
                }

                Ok(game)
            }
        }
    }
}

#[derive(Serialize, Debug)]
struct GameResp<G: Game> {
    name: String,
    owner_id: i32,
    state: Option<G::State>,
    player_ids: Vec<i32>,
    active: bool,
}

#[get("/game/<id>")]
fn game_get(
    id: i32,
    db: DBConn,
    state: State<RwLock<GameManager<SimpleGame>>>,
) -> Result<Json<GameResp<SimpleGame>>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };

    let game = app.get_game(GameId(id));
    match game {
        Ok(game) => Ok(Json(GameResp {
            owner_id: game.owner.id(),
            state: game.game.as_ref().and_then(|game| Some(game.state())),
            player_ids: game.players.iter().map(|id| id.id()).collect::<Vec<i32>>(),
            active: game.active(),
            name: game.name,
        })),
        Err(err) => Err(Json(ErrorResp::from(err))),
    }
}

#[derive(FromForm)]
struct NewGameForm {
    name: String,
}

#[derive(Serialize, Debug)]
pub struct IdResp {
    id: String,
}

#[post("/game/new", data = "<new_game>")]
fn game_new(
    new_game: Form<NewGameForm>,
    db: DBConn,
    state: State<RwLock<GameManager<SimpleGame>>>,
) -> Result<Json<IdResp>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };
    let id = app.new_game(&new_game.name, PlayerId::new(0));

    match id {
        Ok(id) => Ok(Json(IdResp { id: id.to_string() })),
        Err(err) => Err(Json(ErrorResp::from(err))),
    }
}

fn main() {
    rocket::ignite()
        .attach(DBConn::fairing())
        .manage(RwLock::new(GameManager::<SimpleGame>::default()))
        .manage(RwLock::new(HashMap::<String, PlayerId>::new()))
        .mount(
            "/api",
            routes![
                game_get,
                game_new,
                users::user_new,
                users::session_new,
                users::session_delete,
                users::user_generate_api_key
            ],
        )
        .register(catchers![users::unauthorized])
        .launch();
}
