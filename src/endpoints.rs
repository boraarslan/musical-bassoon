use std::sync::Arc;

use axum::{extract::rejection::JsonRejection, Extension, Json};
use serde_json::{json, Value};

use crate::{
    handle_json_error,
    redis::{fetch_all_tasks, fetch_task, insert_task_to_queue, schedule_task, remove_task},
    task::{CreateTaskRequest, DeleteTaskRequest, ShowTaskRequest, Task},
    AppError, State,
};

pub async fn create_task(
    request: Result<Json<CreateTaskRequest>, JsonRejection>,
    state: Extension<Arc<State>>,
) -> Result<Json<Value>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    let task = Task::from(request.0);
    let task_id = task.id;

    match task.scheduled_time {
        Some(_) => schedule_task(task, &state.db).await?,
        None => insert_task_to_queue(task, &state.db).await?,
    }

    return Ok(Json(json!({ "task_id": task_id })));
}

pub async fn show_all_tasks(
    state: Extension<Arc<State>>,
) -> Result<Json<Value>, AppError> {
    let (queued, scheduled) = fetch_all_tasks(&state.db).await?;

    Ok(Json(json!({
        "queued": queued,
        "scheduled": scheduled
    })))
}

pub async fn show_task(
    request: Result<Json<ShowTaskRequest>, JsonRejection>,
    state: Extension<Arc<State>>,
) -> Result<Json<Task>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };
    let task = fetch_task(request.task_id, &state.db).await?;

    Ok(Json(task))
}

pub async fn delete_task(
    request: Result<Json<DeleteTaskRequest>, JsonRejection>,
    state: Extension<Arc<State>>,
) -> Result<(), AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    remove_task(request.task_id, &state.db).await?;

    Ok(())
}
