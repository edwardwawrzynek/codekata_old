use rocket_contrib::json::Json;
use serde::Serialize;

#[database("db")]
pub struct DBConn(pub(crate) diesel::SqliteConnection);

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
    GameAlreadyStarted,
    AlreadyInGame,
    InvalidNumPlayers,
    NotGameOwner,
    NotJoinedGame,
    GameNotStarted,
    WrongTurn,
    InvalidMove,
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
    pub error: String,
    pub success: bool,
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
                Error::GameAlreadyStarted => "game is already started".to_string(),
                Error::AlreadyInGame => "player has already joined the game".to_string(),
                Error::InvalidNumPlayers => "invalid number of players to start game".to_string(),
                Error::NotGameOwner => "player is not the owner of game".to_string(),
                Error::NotJoinedGame => "player has not joined the game".to_string(),
                Error::GameNotStarted => "game has not started yet".to_string(),
                Error::WrongTurn => "player played out of turn".to_string(),
                Error::InvalidMove => "invalid move".to_string(),
            },
            success: false,
        }
    }
}

impl From<Error> for rocket_contrib::json::Json<ErrorResp> {
    fn from(e: Error) -> Self {
        Json(ErrorResp::from(e))
    }
}

#[derive(Serialize, Debug)]
pub struct IdResp {
    pub id: String,
}

#[derive(Serialize, Debug)]
pub struct SuccessResp {
    pub success: bool,
}
