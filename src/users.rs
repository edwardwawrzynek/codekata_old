use rocket::request::{Form, FromRequest, Outcome};
use rocket_contrib::json::Json;

use crate::models::{NewUser, User};
use crate::shared::{DBConn, Error, ErrorResp, SuccessResp};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use uuid::Uuid;
extern crate bcrypt;
extern crate time;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::http::{Cookie, Cookies};
use rocket::{Request, State};
use std::collections::HashMap;
use std::sync::RwLock;
use rocket_contrib::databases::diesel::connection::SimpleConnection;

const BCRYPT_COST: u32 = 8;

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// A hashed api key (safe to store in db + otherwise expose)
pub struct HashedApiKey([u8; 32]);

impl ToString for HashedApiKey {
    fn to_string(&self) -> String {
        // convert to hex string
        let mut res = String::new();
        for b in &self.0 {
            res.push(HEX_CHARS[(b & 0x0f) as usize]);
            res.push(HEX_CHARS[((b >> 4) & 0x0f) as usize]);
        }

        res
    }
}

impl TryFrom<String> for HashedApiKey {
    type Error = Error;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        let mut res = [0u8; 32];

        if key.len() != 64 {
            Err(Error::MalformedApiKey)
        } else {
            for (i, (c1, c2)) in key.chars().tuples().enumerate() {
                let v1 = HEX_CHARS.iter().position(|c| *c == c1);
                let v2 = HEX_CHARS.iter().position(|c| *c == c2);
                match (v1, v2) {
                    (Some(i1), Some(i2)) => {
                        res[i] = ((i1 & 0xf) + ((i2 << 4) & 0xf0) & 0xff) as u8;
                    }
                    (_, _) => return Err(Error::MalformedApiKey),
                }
            }

            Ok(HashedApiKey(res))
        }
    }
}

/// A non hashed api key (not safe to expose in db)
pub struct ApiKey(Uuid);

impl ApiKey {
    pub fn hash(&self) -> HashedApiKey {
        let key_hash = Sha256::digest(self.0.as_bytes());

        let mut hash = [0; 32];
        for (i, b) in key_hash.as_slice().iter().enumerate() {
            hash[i] = *b;
        }

        HashedApiKey(hash)
    }

    pub fn new() -> ApiKey {
        ApiKey(Uuid::new_v4())
    }
}

impl ToString for ApiKey {
    fn to_string(&self) -> String {
        format!("{}", self.0.simple())
    }
}

impl From<&str> for ApiKey {
    fn from(str: &str) -> ApiKey {
        match Uuid::parse_str(str) {
            Ok(uuid) => ApiKey(uuid),
            Err(_) => ApiKey(Uuid::nil()),
        }
    }
}

/// Player id number
#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Default, Debug)]
pub struct PlayerId(i32);

impl PlayerId {
    pub fn next(&self) -> PlayerId {
        PlayerId(self.0 + 1)
    }
    pub fn new(id: i32) -> PlayerId {
        PlayerId(id)
    }
    pub fn id(&self) -> i32 {
        self.0
    }
}

impl ToString for PlayerId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl User {
    pub fn check_password(&self, password: &str) -> bool {
        match bcrypt::verify(password.as_bytes(), &self.password_hash) {
            Ok(true) => true,
            _ => false,
        }
    }
}

/// wrapper around db connection that manages users
pub struct UserManager<'a> {
    db: DBConn,
    sessions: &'a RwLock<HashMap<String, PlayerId>>,
}

pub type UserManagerState<'a> = State<'a, RwLock<HashMap<String, PlayerId>>>;

impl<'a> UserManager<'a> {
    #[allow(unused_must_use)]
    pub fn new(db: DBConn, sessions: &'a RwLock<HashMap<String, PlayerId>>) -> Self {
        db.0.batch_execute("PRAGMA busy_timeout = 3000;");
        UserManager { db, sessions }
    }

    /// check that the given api key matches the user's key
    pub fn check_api_key(hashed_key: &str, key: &str) -> bool {
        let hash = ApiKey::from(key).hash();

        hash.to_string() == hashed_key
    }

    /// create a new user, insert into the db, and return their id
    pub fn new_user(
        &self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<PlayerId, Error> {
        use crate::schema::users;

        let matching_usernames = users::dsl::users
            .filter(users::dsl::username.eq(username))
            .load::<User>(&*self.db)?
            .into_iter();

        if matching_usernames.count() > 0 {
            Err(Error::UsernameAlreadyTaken)
        } else {
            let new_user = NewUser {
                username,
                display_name,
                password_hash: &*bcrypt::hash(password.as_bytes(), BCRYPT_COST)?,
                api_key_hash: None,
            };

            let inserted_users = (self.db).transaction::<_, diesel::result::Error, _>(|| {
                let insert_count = diesel::insert_into(users::table)
                    .values(&new_user)
                    .execute(&*self.db)?;
                assert_eq!(insert_count, 1);

                Ok(users::dsl::users
                    .order(users::id.desc())
                    .limit(insert_count as i64)
                    .load(&*self.db)?
                    .into_iter()
                    .collect::<Vec<User>>())
            })?;

            Ok(PlayerId(inserted_users[0].id))
        }
    }

    /// load a user from the db by user id
    pub fn load_user(&self, user_id: PlayerId) -> Result<User, Error> {
        use crate::schema::users;

        Ok(users::dsl::users.find(user_id.0).first::<User>(&*self.db)?)
    }

    /// load a user from the db by username
    pub fn find_user(&self, username: &str) -> Result<User, Error> {
        use crate::schema::users;

        let users = users::dsl::users
            .filter(users::dsl::username.eq(username))
            .load::<User>(&*self.db)?;
        if users.len() == 0 {
            Err(Error::NoSuchUser)
        } else {
            Ok(users[0].clone())
        }
    }

    /// save a user to the db
    pub fn save_user(&self, user: &User) -> Result<(), Error> {
        use crate::schema::users;

        diesel::update(users::dsl::users.find(user.id))
            .set(user)
            .execute(&*self.db)?;
        Ok(())
    }

    /// find the user with the specified api key
    pub fn find_user_by_api_key(&self, key: &str) -> Result<User, Error> {
        use crate::schema::users;

        let hash = ApiKey::from(key).hash();

        let users = users::dsl::users
            .filter(users::dsl::api_key_hash.eq(hash.to_string()))
            .load::<User>(&*self.db)?;
        if users.len() == 0 {
            Err(Error::InvalidApiKey)
        } else {
            Ok(users[0].clone())
        }
    }

    /// generate (or regenerate) the api key for a user, and return the key
    pub fn generate_api_key(&self, user_id: PlayerId) -> Result<String, Error> {
        use crate::schema::users;
        let key = ApiKey::new();
        let hash = key.hash();

        diesel::update(users::dsl::users.find(user_id.0))
            .set(users::dsl::api_key_hash.eq(hash.to_string()))
            .execute(&*self.db)?;
        Ok(key.to_string())
    }

    /// create a new session for the given user
    pub fn new_session(&self, user_id: PlayerId) -> String {
        let mut sessions = self.sessions.write().unwrap();
        let session_key = format!("{}", Uuid::new_v4().simple());
        sessions.insert(session_key.clone(), user_id);

        session_key
    }

    /// determine what user a session belongs to
    pub fn lookup_session(&self, session: &str) -> Option<PlayerId> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session).map(|id| *id)
    }

    /// remove a session
    pub fn end_session(&self, session: &str) {
        let mut sessions = self.sessions.write().unwrap();

        sessions.remove(session);
    }
}

/// a request guard that checks that users are authenticated with a session (cookie) or api key (X-API-KEY header)
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        // get db and sessions state guards
        let db_guard = match request
            .guard::<DBConn>()
            .map_failure(|f| (f.0, Error::GuardLoadError))
        {
            Outcome::Success(db) => db,
            Outcome::Failure(err) => return Outcome::Failure(err),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        let sessions_guard = match request
            .guard::<UserManagerState>()
            .map_failure(|f| (f.0, Error::GuardLoadError))
        {
            Outcome::Success(sessions) => sessions,
            Outcome::Failure(err) => return Outcome::Failure(err),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        let manage = UserManager::new(db_guard, &*sessions_guard);

        // check for session_key cookie
        if let Some(cookie) = request.cookies().get_private("session_key") {
            if let Some(user_id) = manage.lookup_session(cookie.value()) {
                let user = manage.load_user(user_id);
                match user {
                    Err(e) => Outcome::Failure((Status::Unauthorized, e)),
                    Ok(user) => Outcome::Success(user),
                }
            } else {
                Outcome::Failure((Status::Unauthorized, Error::Unauthorized))
            }
        } else {
            // check for api key
            let keys = request.headers().get("x-api-key").collect::<Vec<_>>();
            if keys.len() == 1 {
                match manage.find_user_by_api_key(&keys[0]) {
                    Ok(user) => Outcome::Success(user),
                    Err(e) => Outcome::Failure((Status::Unauthorized, e)),
                }
            } else {
                Outcome::Failure((Status::Unauthorized, Error::NoAuthorizationMethod))
            }
        }
    }
}

// API ROUTES
#[derive(FromForm)]
pub struct NewSessionForm {
    username: String,
    password: String,
}

#[post("/session/new", data = "<login>")]
pub fn session_new(
    login: Form<NewSessionForm>,
    db: DBConn,
    state: UserManagerState,
    mut cookies: Cookies,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    let manage = UserManager::new(db, &*state);
    let user = manage.find_user(&login.username)?;

    if user.check_password(&login.password) {
        cookies.add_private(Cookie::new(
            "session_key",
            manage.new_session(PlayerId(user.id)),
        ));

        Ok(Json(SuccessResp { success: true }))
    } else {
        Err(Json::from(Error::InvalidPassword))
    }
}

#[post("/session/delete")]
pub fn session_delete(
    db: DBConn,
    state: UserManagerState,
    mut cookies: Cookies,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    let manage = UserManager::new(db, &*state);
    if let Some(session) = cookies.get_private("session_key") {
        manage.end_session(session.value());
        cookies.remove_private(session);

        Ok(Json(SuccessResp { success: true }))
    } else {
        Err(Json(ErrorResp::from(Error::NoAuthorizationMethod)))
    }
}

#[derive(FromForm)]
pub struct NewUserForm {
    username: String,
    display_name: String,
    password: String,
}

#[post("/user/new", data = "<user>")]
pub fn user_new(
    user: Form<NewUserForm>,
    db: DBConn,
    state: UserManagerState,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    UserManager::new(db, &*state).new_user(
        &*user.username,
        &*user.display_name,
        &*user.password,
    )?;

    Ok(Json(SuccessResp { success: true }))
}

#[derive(Serialize)]
pub struct UserResp {
    username: String,
    display_name: String,
    has_api_key: bool,
    id: i32,
}

#[get("/user")]
pub fn user_get(user: User) -> Json<UserResp> {
    Json(UserResp {
        username: user.username,
        display_name: user.display_name,
        has_api_key: user.api_key_hash.is_some(),
        id: user.id,
    })
}

#[derive(FromForm)]
pub struct EditUserForm {
    username: Option<String>,
    display_name: Option<String>,
    password: Option<String>,
}

#[post("/user/edit", data = "<edit>")]
pub fn user_edit(
    edit: Form<EditUserForm>,
    db: DBConn,
    state: UserManagerState,
    mut user: User,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    let manage = UserManager::new(db, &*state);

    if let Some(username) = &edit.username {
        if *username != user.username {
            // check that username isn't already taken
            if manage.find_user(&*username).is_ok() {
                return Err(Json(ErrorResp::from(Error::UsernameAlreadyTaken)));
            }
        }
        user.username = username.clone();
    };
    if let Some(display_name) = &edit.display_name {
        user.display_name = display_name.clone();
    };
    if let Some(password) = &edit.password {
        user.password_hash = bcrypt::hash(password, BCRYPT_COST).map_err(Error::from)?;
    };

    manage.save_user(&user)?;

    Ok(Json(SuccessResp { success: true }))
}

#[derive(Serialize)]
pub struct ApiKeyResponse {
    key: String,
}

#[post("/user/generate_api")]
pub fn user_generate_api_key(
    user: User,
    db: DBConn,
    state: UserManagerState,
) -> Result<Json<ApiKeyResponse>, Json<ErrorResp>> {
    let manage = UserManager::new(db, &*state);
    let key = manage.generate_api_key(PlayerId::new(user.id))?;
    Ok(Json(ApiKeyResponse { key }))
}

#[catch(401)]
pub fn unauthorized(_: &Request) -> Json<ErrorResp> {
    Json(ErrorResp::from(Error::Unauthorized))
}
