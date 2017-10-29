#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rocket::http::{self, Cookie, RawStr};
use rocket::request::{self, FromFormValue};
use rocket::response::status;
use rocket::Outcome;
use rocket_contrib::Template;

mod static_files;
mod db;
mod domain;
mod view_models;
mod controllers;

use db::redis::db_pool;
use domain::TodoFilter;
use domain::SessionId;
use controllers::root;
use controllers::todo;
use controllers::todos;

// Cookie session

const SESSION_ID_KEY: &'static str = "session_id";

pub struct CookieSessionId(SessionId);

impl From<CookieSessionId> for SessionId {
    fn from(id: CookieSessionId) -> Self {
        id.0
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for CookieSessionId {
    type Error = ();

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<CookieSessionId, ()> {
        let id_from_cookie: Option<usize> = request
            .cookies()
            .get(SESSION_ID_KEY)
            .and_then(|cookie| cookie.value().parse().ok());

        let id = match id_from_cookie {
            None => {
                let random_id = rand::random::<usize>();
                request
                    .cookies()
                    .add(Cookie::new(SESSION_ID_KEY, format!("{}", random_id)));
                random_id
            }
            Some(id) => id,
        };

        Outcome::Success(CookieSessionId(id))
    }
}

// Query params

#[derive(FromForm)]
pub struct QueryParams {
    filter: TodoFilter,
}

pub fn todo_filter(filter: Option<QueryParams>) -> Result<TodoFilter, status::Custom<String>> {
    Ok(filter.ok_or(invalid_filter_bad_request())?.filter)
}

fn invalid_filter_bad_request() -> status::Custom<String> {
    status::Custom(http::Status::BadRequest, "Invalid filter".into())
}

impl<'v> FromFormValue<'v> for TodoFilter {
    type Error = &'v RawStr;

    fn from_form_value(value: &'v RawStr) -> Result<Self, Self::Error> {
        let variant = match value.as_str() {
            "all" => TodoFilter::All,
            "active" => TodoFilter::Active,
            "completed" => TodoFilter::Completed,
            _ => return Err(value),
        };

        Ok(variant)
    }
}

// Launch

fn main() {
    let app = rocket::ignite();
    let pool = { db_pool(app.config()) };

    app.mount("/", routes![root::index, static_files::all])
        .mount("/todo", routes![todo::create, todo::update, todo::destroy])
        .mount("/todos", routes![todos::show, todos::update])
        .manage(pool)
        .attach(Template::fairing())
        .launch();
}
