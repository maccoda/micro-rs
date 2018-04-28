use rocket_contrib::{Json, Value};
use rocket;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

use models::{NewTask, Task};
use super::{init_pool, DbConn, Pool};

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
    let id = Task::update(&conn, id as i32, task.into_inner());
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

pub fn start() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/task", routes![create_task, delete_task, update_task])
        .mount("/tasks", routes![get_all_tasks])
        .manage(init_pool())
        .catch(errors![not_found])
        .launch();
}
