use std::net::SocketAddr;
use std::str::FromStr;

use axum::routing::{delete, get, post};
use axum::Router;
use once_cell::sync::Lazy;
use rest_task_scheduler::endpoints::{
    create_task, delete_task, show_all_tasks, show_task, show_tasks_by_state, show_tasks_by_type,
};
use rest_task_scheduler::worker::{TaskGarbageCollector, Worker};
use rest_task_scheduler::AppState;
use sqlx::postgres::PgPoolOptions;

static DATABASE_URL: Lazy<String> = Lazy::new(|| std::env::var("DATABASE_URL").unwrap());

static LISTEN_PORT: Lazy<String> =
    Lazy::new(|| std::env::var("LISTEN_PORT").unwrap_or_else(|_| "7272".to_string()));

fn init_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/create", post(create_task))
        .route("/show", get(show_task))
        .route("/show/state", get(show_tasks_by_state))
        .route("/show/type", get(show_tasks_by_type))
        .route("/show/all", get(show_all_tasks))
        .route("/delete", delete(delete_task))
        .with_state(shared_state)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&DATABASE_URL)
        .await
        .unwrap();

    let worker = Worker::new(&db_pool);
    let garbage_collector = TaskGarbageCollector::new(&db_pool);

    tokio::spawn(async move { worker.run().await.unwrap() });
    tokio::spawn(async move { garbage_collector.run().await.unwrap() });

    let shared_state = AppState { db: db_pool };
    let addr =
        SocketAddr::from_str(&format!("0.0.0.0:{}", LISTEN_PORT.parse::<u16>().unwrap())).unwrap();
    println!("Listening on port: {}", addr.port());

    let app = init_router(shared_state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
