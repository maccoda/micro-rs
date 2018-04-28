#![feature(plugin)]
#![plugin(rocket_codegen)]
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde_json;

// Rocket
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

// Gotham
extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;

use std::ops::Deref;
use std::env;

use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

mod rocket_web;
mod gotham_web;
mod schema;
mod models;

// An alias to the type for a pool of Diesel SQLite connections.

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
fn init_pool() -> Pool {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(manager).expect("db pool")
}

// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
fn main() {
    let arg = env::args()
        .nth(1)
        .expect("Did not define which framework to use")
        .to_lowercase();
    if arg.eq("gotham") {
        gotham_web::start();
    } else if arg.eq("rocket") {
        rocket_web::start();
    } else {
        println!("No framework supported for {}", arg);
    }
}
