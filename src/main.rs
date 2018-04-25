#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::{Json, Value};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

use std::env;

mod schema;
mod models;

use models::{NewTask, Task};

struct TodoStore {
    tasks: Vec<Task>,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/")]
fn get_all_tasks(conn: DbConn) -> Json<Vec<Task>> {
    Json(Task::all(&conn))
}

#[post("/", data = "<task>")]
fn create_task(task: Json<NewTask>, conn: DbConn) -> Json<String> {
    Task::create(&conn, task.into_inner());
    Json("Task added".to_owned())
}

#[put("/<id>", data = "<task>")]
fn update_task(id: u32, task: Json<Task>, conn: DbConn) -> Json<i32> {
    let id = Task::update(&conn, task.into_inner());
    Json(id)
}

#[delete("/<id>")]
fn delete_task(id: u32, conn: DbConn) -> Json<Value> {
    Task::delete(&conn, id as i32);
    Json(json!({"status": "ok"}))
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}

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

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/task", routes![create_task, delete_task, update_task])
        .mount("/tasks", routes![get_all_tasks])
        .manage(init_pool())
        .catch(errors![not_found])
        .launch();
}
