use rocket::request::{Form, FromRequest, FromSegments, Outcome};
use rocket_contrib::json::Json;

use crate::models::{NewPage, NewUser, Page, User};
use crate::shared::{DBConn, Error, ErrorResp, IdResp, SuccessResp};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use uuid::Uuid;
extern crate bcrypt;
extern crate time;
use diesel::prelude::*;
use rocket::http::uri::Segments;
use rocket::http::Status;
use rocket::http::{Cookie, Cookies};
use rocket::{Request, State};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(FromForm)]
pub struct NewPageForm {
    url: String,
    content: String,
}

#[post("/pages/new", data = "<page>")]
pub fn page_new(
    page: Form<NewPageForm>,
    db: DBConn,
    user: User,
) -> Result<Json<IdResp>, Json<ErrorResp>> {
    use crate::schema::pages;

    if !user.is_admin {
        Err(Json::from(Error::NotAdmin))
    } else {
        let new_entry = NewPage {
            url: &page.url,
            content: &page.content,
        };

        let inserted = diesel::insert_into(pages::table)
            .values(&new_entry)
            .get_result::<Page>(&*db)
            .map_err(|e| Error::from(e))?;

        Ok(Json(IdResp {
            id: inserted.id.to_string(),
        }))
    }
}

#[post("/pages/edit", data = "<page>")]
pub fn page_edit(
    page: Form<Page>,
    db: DBConn,
    user: User,
) -> Result<Json<SuccessResp>, Json<ErrorResp>> {
    use crate::schema::pages;

    if !user.is_admin {
        Err(Json::from(Error::NotAdmin))
    } else {
        diesel::update(pages::dsl::pages.find(page.id))
            .set(&*page)
            .execute(&*db)
            .map_err(|e| Error::from(e))?;

        Ok(Json(SuccessResp { success: true }))
    }
}

pub struct PageUrl(String);

impl<'a> FromSegments<'a> for PageUrl {
    type Error = ();

    fn from_segments(segments: Segments<'a>) -> Result<Self, Self::Error> {
        let mut res = String::new();
        for segment in segments.into_iter() {
            res.push_str(segment);
            res.push('/');
        }
        res.pop();

        Ok(PageUrl(res))
    }
}

#[get("/pages/<path..>")]
pub fn page_get(path: PageUrl, db: DBConn) -> Result<Json<Page>, Json<ErrorResp>> {
    use crate::schema::pages;

    let page = pages::dsl::pages
        .filter(pages::dsl::url.eq(path.0))
        .first::<Page>(&*db)
        .map_err(|e| Error::from(e))?;

    Ok(Json(page))
}
