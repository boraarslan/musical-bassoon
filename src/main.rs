use std::{sync::Arc, net::SocketAddr, str::FromStr};

use axum::{
    routing::{delete, get, post},
    Extension, Router,
};
use bb8_redis::RedisConnectionManager;
use once_cell::sync::Lazy;
use rest_task_scheduler::{
    endpoints::{create_task, delete_task, show_task, show_all_tasks},
    worker::{ScheduledTaskManager, Worker}, State,
};

static REDIS_URL: Lazy<String> = Lazy::new(|| std::env::var("REDIS_URL").unwrap());
static LISTEN_PORT: Lazy<String> = Lazy::new(|| std::env::var("LISTEN_PORT").unwrap_or("7272".to_string()));

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let manager = RedisConnectionManager::new(REDIS_URL.as_str()).unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    let worker = Worker::new(&pool);
    let scheduled_task_manager = ScheduledTaskManager::new(&pool);
    tokio::spawn(async move { worker.run().await });
    tokio::spawn(async move { scheduled_task_manager.run().await });

    let state = Arc::new(State { db: pool });
    let addr = SocketAddr::from_str(&format!("0.0.0.0:{}", LISTEN_PORT.parse::<u16>().unwrap())).unwrap();
    println!("Listening on port: {}", addr.port());

    let app = Router::new()
        .route("/task/create", post(create_task))
        .route("/task/show", get(show_task))
        .route("/task/show/all", get(show_all_tasks))
        .route("/task/delete", delete(delete_task))
        .layer(Extension(state));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
