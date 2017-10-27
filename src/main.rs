#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::default::Default;
use rocket::config;
use rocket_contrib::Template;
use r2d2_redis::RedisConnectionManager;

mod db;
mod domain;
mod controllers;

use domain::SessionId;
use controllers::root;
use controllers::todo;
use controllers::todos;

// DB Pool

const REDIS_DEFAULT_ADDRESS: &'static str = "redis://localhost";
const REDIS_ADDRESS_CONFIG_KEY: &'static str = "redis_connection_address";

type Pool = r2d2::Pool<RedisConnectionManager>;

fn init_redis_pool(app_config: &config::Config) -> Pool {
    let address = app_config
        .extras
        .get(REDIS_ADDRESS_CONFIG_KEY)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| REDIS_DEFAULT_ADDRESS);
    let manager = RedisConnectionManager::new(address).expect("connection manager");
    let redis_config = Default::default();

    r2d2::Pool::new(redis_config, manager).expect("db pool")
}

// User session

pub struct UserSession {
    id: SessionId,
}

// Launch

fn main() {
    let app = rocket::ignite();
    let redis_pool = { init_redis_pool(app.config()) };
    let user_session = UserSession { id: 42 }; // TODO: Generate id randomly and store in cookies.

    app.mount("/", routes![root::index])
        .mount("/todos", routes![todos::show]) // RESTful Resource for a todo item
        .mount("/todo", routes![todo::create]) // RESTful Resource for a list of items
        .manage(redis_pool)
        .manage(user_session)
        .attach(Template::fairing())
        .launch();
}
