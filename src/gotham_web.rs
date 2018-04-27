use gotham;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::State;
use gotham::http::response::create_response;
use gotham::handler::IntoResponse;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use hyper::{Response, StatusCode};
use mime;
use serde_json;
use futures::future;

use models::Task;
use super::{init_pool, DbConn, Pool};

use std::ops::Deref;

fn router() -> Router {
    let (chain, pipelines) = single_pipeline(new_pipeline().add(DbConnMiddleware).build());
    build_router(chain, pipelines, |route| {
        route.get_or_head("/").to(index);
        route.get("/tasks").to(get_all_tasks);
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

fn get_all_tasks(state: State) -> (State, Task) {
    let conn = db_conn(&state).expect("Failed with DB connection");
    let tasks = Task::all(&conn)[0].clone();
    (state, tasks)
}
pub fn start() {
    let addr = "127.0.0.1:8000";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}
