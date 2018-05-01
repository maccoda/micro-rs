use gotham;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham::http::response::create_response;
use gotham::handler::{IntoHandlerError, IntoResponse};
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use hyper::{Body, Response, StatusCode};
use mime;
use serde_json;
use futures::{future, Future, Stream};

use models::{Task, TaskList};
use super::{init_pool, DbConn, Pool};

use std::ops::Deref;

fn router() -> Router {
    let (chain, pipelines) = single_pipeline(new_pipeline().add(DbConnMiddleware).build());
    build_router(chain, pipelines, |route| {
        route.get_or_head("/").to(index);
        route.get("/tasks").to(get_all_tasks);
        route.post("/task").to(create_task);
        route
            .put("/task/:id")
            .with_path_extractor::<PathId>()
            .to(update_task);
        route
            .delete("/task/:id")
            .with_path_extractor::<PathId>()
            .to(delete_task);
    })
}

#[derive(Clone, NewMiddleware)]
struct DbConnMiddleware;

#[derive(StateData)]
struct PoolState(Pool);

impl Deref for PoolState {
    type Target = Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Middleware for DbConnMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        if !state.has::<PoolState>() {
            // Initialize it
            state.put(PoolState(init_pool()));
        }
        chain(state)
    }
}

fn db_conn(state: &State) -> Option<DbConn> {
    state.borrow::<PoolState>().get().ok().map(|x| DbConn(x))
}

fn index(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((String::from("Hello Router!").into_bytes(), mime::TEXT_PLAIN)),
    );
    (state, res)
}

impl IntoResponse for Task {
    fn into_response(self, state: &State) -> Response {
        create_response(
            &state,
            StatusCode::Ok,
            Some((
                serde_json::to_string(&self)
                    .expect("serialized product")
                    .into_bytes(),
                mime::APPLICATION_JSON,
            )),
        )
    }
}

impl IntoResponse for TaskList {
    fn into_response(self, state: &State) -> Response {
        create_response(
            &state,
            StatusCode::Ok,
            Some((
                serde_json::to_string(&self.list)
                    .expect("serialized product")
                    .into_bytes(),
                mime::APPLICATION_JSON,
            )),
        )
    }
}

fn get_all_tasks(state: State) -> (State, TaskList) {
    let conn = db_conn(&state).expect("Failed with DB connection");
    let tasks = TaskList {
        list: Task::all(&conn),
    };
    (state, tasks)
}

fn body_handler<F>(mut state: State, f: F) -> Box<HandlerFuture>
where
    F: 'static + Fn(String, &State) -> Response,
{
    let body = Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
                let res = f(body_content, &mut state);
                future::ok((state, res))
            }
            Err(e) => return future::err((state, e.into_handler_error())),
        });
    Box::new(body)
}

fn create_task(state: State) -> Box<HandlerFuture> {
    body_handler(state, |s, state| {
        let task = serde_json::from_str(&s).expect("Failed to deserialize");
        let conn = db_conn(state).expect("Failed with DB connection");
        Task::create(&conn, task);
        create_response(state, StatusCode::Ok, None)
    })
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct PathId {
    id: u32,
}

fn update_task(mut state: State) -> Box<HandlerFuture> {
    let PathId { id } = PathId::take_from(&mut state);
    body_handler(state, move |s, state| {
        let task = serde_json::from_str(&s).expect("Failed to deserialize");
        let conn = db_conn(&state).expect("Failed with DB connection");
        Task::update(&conn, id as i32, task);
        create_response(state, StatusCode::Ok, None)
    })
}

fn delete_task(mut state: State) -> (State, Response) {
    let PathId { id } = PathId::take_from(&mut state);
    let conn = db_conn(&state).expect("Failed with DB connection");
    Task::delete(&conn, id as i32);
    let resp = create_response(&state, StatusCode::Ok, None);
    (state, resp)
}

pub fn start() {
    let addr = "127.0.0.1:8000";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}
