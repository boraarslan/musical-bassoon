use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::Json;
use axum_macros::debug_handler;
use serde_json::{json, Value};

use crate::database::{
    get_all_tasks, get_task_by_id, get_tasks_by_state, get_tasks_by_type, insert_task_to_queue,
    remove_task,
};
use crate::task::{
    CreateTaskRequest, DeleteTaskRequest, ShowTaskByStateRequest, ShowTaskByTypeRequest,
    ShowTaskRequest, Task,
};
use crate::{handle_json_error, AppError, AppState};

#[debug_handler]
pub async fn create_task(
    State(state): State<AppState>,
    request: Result<Json<CreateTaskRequest>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    let task = Task::from(request.0);

    let task_id = insert_task_to_queue(task, &state.db).await?;

    Ok(Json(json!(task_id)))
}

#[debug_handler]
pub async fn show_all_tasks(state: State<AppState>) -> Result<Json<Value>, AppError> {
    let tasks = get_all_tasks(&state.db).await?;
    let tasks_json = json!({"total":tasks.len(), "tasks": tasks});

    Ok(Json(tasks_json))
}

#[debug_handler]
pub async fn show_tasks_by_state(
    state: State<AppState>,
    request: Result<Json<ShowTaskByStateRequest>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    let tasks = get_tasks_by_state(request.state, &state.db).await?;
    let tasks_json = json!({"total": tasks.len(), "tasks": tasks});

    Ok(Json(tasks_json))
}

#[debug_handler]
pub async fn show_tasks_by_type(
    state: State<AppState>,
    request: Result<Json<ShowTaskByTypeRequest>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    let tasks = get_tasks_by_type(request.task_type, &state.db).await?;
    let tasks_json = json!({"total": tasks.len(), "tasks": tasks});

    Ok(Json(tasks_json))
}

#[debug_handler]
pub async fn show_task(
    state: State<AppState>,
    request: Result<Json<ShowTaskRequest>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    let task = get_task_by_id(request.task_id, &state.db).await?;
    let task_json = match task {
        Some(task) => json!(task),
        None => json!("Task not found"),
    };

    Ok(Json(task_json))
}

#[debug_handler]
pub async fn delete_task(
    state: State<AppState>,
    request: Result<Json<DeleteTaskRequest>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Ok(request) = request else {
        return Err(handle_json_error(request.unwrap_err()));
    };

    let rows_affected = remove_task(request.task_id, &state.db).await?;
    let message_json = match rows_affected {
        0 => json!("Unable to delete the task. Task does not exist with the given id."),
        _ => json!("Successfully deleted the task."),
    };

    Ok(Json(message_json))
}
