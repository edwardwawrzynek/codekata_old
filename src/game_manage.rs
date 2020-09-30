use crate::game::{Game, GamePlayer};
use crate::models::{DBGameId, DbGame, InsertDbGame, NewDbGame, User};
use crate::shared::{DBConn, Error, ErrorResp, IdResp, SuccessResp};
use crate::users::{PlayerId, UserManager, UserManagerState};
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
    fn id(&self) -> i32 {
        self.0
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
    /// check if the game has been started
    fn started(&self) -> bool {
        match &self.game {
            None => false,
            Some(_) => true,
        }
    }
    /// check if the game is started and has active player (not finished)
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
                .unwrap();
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

    /// save a game to the database
    fn save_game_to_db<'l>(
        &self,
        game: &GameInstance<G>,
        manager_lock: RwLockWriteGuard<'l, GameManager<G>>,
    ) -> Result<RwLockWriteGuard<'l, GameManager<G>>, Error> {
        use crate::schema::db_games;
        let new_entry = InsertDbGame::from(game);

        let res = diesel::update(db_games::dsl::db_games.find(game.id.id()))
            .set(&new_entry)
            .execute(&*self.db)
            .map_or_else(|e| Err(Error::DBError(e)), |r| Ok(manager_lock));

        res
    }

    /// create a new game entry in the db and in active_games
    fn new_game(&self, name: &str, owner: PlayerId) -> Result<GameId, Error> {
        use crate::schema::db_games;

        let game = NewDbGame {
            players: serde_json::to_string(&Vec::<Vec<String>>::new())?,
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
                    let mut manager = self.save_game_to_db(&res, manager)?;
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

    /// save a game
    /// possibly saves to the cache or db
    fn save_game(&self, game: GameInstance<G>) -> Result<(), Error> {
        let mut manager = self.manager.write().unwrap();
        if game.active() {
            manager.active_games.insert(game.id, game);
        } else {
            self.save_game_to_db(&game, manager)?;
        }

        Ok(())
    }

    /// add a player to the given game
    fn join_game(&self, game_id: GameId, player_id: PlayerId) -> Result<(), Error> {
        let mut game = self.get_game(game_id)?;
        if game.active() {
            Err(Error::GameAlreadyStarted)
        } else {
            if game.players.contains(&player_id) {
                Err(Error::AlreadyInGame)
            } else {
                game.players.push(player_id);
                self.save_game(game)?;

                Ok(())
            }
        }
    }

    /// remove a player from the given game (if it has not started)
    fn leave_game(&self, game_id: GameId, player_id: PlayerId) -> Result<(), Error> {
        let mut game = self.get_game(game_id)?;

        if !game.players.contains(&player_id) {
            Err(Error::NotJoinedGame)
        } else if game.started() {
            Err(Error::GameAlreadyStarted)
        } else {
            game.players
                .iter()
                .position(|id| *id == player_id)
                .map(|pos| game.players.remove(pos));
            self.save_game(game)?;
            Ok(())
        }
    }

    /// start the game with the given id (ie -- give it a state)
    /// player_id must be the owner of the game
    fn start_game(&self, game_id: GameId, player_id: PlayerId) -> Result<(), Error> {
        let mut game = self.get_game(game_id)?;

        if game.owner != player_id {
            Err(Error::NotGameOwner)
        } else if game.started() {
            Err(Error::GameAlreadyStarted)
        } else {
            let num_players = game.players.len();
            if G::check_num_players(num_players) {
                game.game = Some(Box::new(G::new_with_players(num_players)));
                self.save_game(game)?;

                Ok(())
            } else {
                Err(Error::InvalidNumPlayers)
            }
        }
    }

    /// get a list of all games ids in descending order
    fn list_games(&self) -> Result<Vec<i32>, Error> {
        use crate::schema::db_games;

        let ids = db_games::dsl::db_games
            .select((db_games::dsl::id))
            .order(db_games::id.desc())
            .load::<i32>(&*self.db)?;

        Ok(ids)
    }
}

#[derive(Serialize, Debug)]
pub struct GameResp<G: Game> {
    name: String,
    owner_id: i32,
    state: Option<G::State>,
    players: Vec<String>,
    player_ids: Vec<i32>,
    active: bool,
    started: bool,
    waiting_on: Vec<bool>,
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

    let game = app.get_game(GameId(id))?;

    let players = game
        .players
        .iter()
        .map(|id| -> Result<String, Error> {
            use crate::schema::users;

            Ok(users::dsl::users
                .find(id.id())
                .first::<User>(&*app.db)?
                .display_name)
        })
        .collect::<Result<Vec<String>, Error>>()?;

    let player_ids = game.players.iter().map(|id| id.id()).collect::<Vec<i32>>();

    let waiting_on = game
        .players
        .iter()
        .enumerate()
        .map(|(index, id)| {
            game.game
                .as_ref()
                .map_or(false, |g| g.waiting_on(index as u32))
        })
        .collect::<Vec<bool>>();

    Ok(Json(GameResp {
        owner_id: game.owner.id(),
        state: game.game.as_ref().and_then(|game| Some(game.state())),
        players,
        player_ids,
        active: game.active(),
        started: game.started(),
        waiting_on,
        name: game.name,
    }))
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
    user: User,
) -> Result<Json<IdResp>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };
    let id = app.new_game(&new_game.name, PlayerId::new(user.id));

    match id {
        Ok(id) => Ok(Json(IdResp { id: id.to_string() })),
        Err(err) => Err(Json(ErrorResp::from(err))),
    }
}

#[post("/game/<id>/join")]
pub fn game_join(
    id: i32,
    db: DBConn,
    state: State<RwLock<GameManager<crate::SimpleGame>>>,
    user: User,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };
    app.join_game(GameId(id), PlayerId::new(user.id))?;
    Ok(Json(SuccessResp { success: true }))
}

#[post("/game/<id>/leave")]
pub fn game_leave(
    id: i32,
    db: DBConn,
    state: State<RwLock<GameManager<crate::SimpleGame>>>,
    user: User,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };
    app.leave_game(GameId(id), PlayerId::new(user.id))?;
    Ok(Json(SuccessResp { success: true }))
}

#[post("/game/<id>/start")]
pub fn game_start(
    id: i32,
    db: DBConn,
    state: State<RwLock<GameManager<crate::SimpleGame>>>,
    user: User,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };
    app.start_game(GameId(id), PlayerId::new(user.id))?;
    Ok(Json(SuccessResp { success: true }))
}

#[derive(Serialize)]
pub struct IndexResp {
    games: Vec<i32>,
}

#[get("/game/index")]
pub fn game_index(
    db: DBConn,
    state: State<RwLock<GameManager<crate::SimpleGame>>>,
) -> Result<Json<IndexResp>, Json<ErrorResp>> {
    let app = AppState {
        db,
        manager: &*state,
    };
    let games = app.list_games()?;

    Ok(Json(IndexResp { games }))
}
