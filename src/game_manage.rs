use crate::game::Game;
use crate::models::{DbGame, InsertDbGame, NewDbGame};
use crate::shared::{DBConn, Error, ErrorResp, IdResp};
use crate::users::PlayerId;
use core::fmt::Debug;
use diesel::prelude::*;
use rocket::request::Form;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Default, Debug)]
pub struct GameId(i32);

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

pub struct GameManager<G: Game> {
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
pub struct GameResp<G: Game> {
    name: String,
    owner_id: i32,
    state: Option<G::State>,
    player_ids: Vec<i32>,
    active: bool,
}

#[get("/game/<id>")]
pub fn game_get(
    id: i32,
    db: DBConn,
    state: State<RwLock<GameManager<crate::SimpleGame>>>,
) -> Result<Json<GameResp<crate::SimpleGame>>, Json<ErrorResp>> {
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
pub struct NewGameForm {
    name: String,
}

#[post("/game/new", data = "<new_game>")]
pub fn game_new(
    new_game: Form<NewGameForm>,
    db: DBConn,
    state: State<RwLock<GameManager<crate::SimpleGame>>>,
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
