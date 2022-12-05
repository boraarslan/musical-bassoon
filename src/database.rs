use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::task::{PostgresTask, Task, TaskState, TaskType};

pub async fn pull_task(db: &PgPool) -> Result<Option<PostgresTask>, sqlx::Error> {
    let now = OffsetDateTime::now_utc();
    let query = "UPDATE queue SET status = $1, updated_at = $2 WHERE id IN (
        SELECT id FROM queue WHERE
        status != $1 AND scheduled_for <= $2
        ORDER BY scheduled_for
        FOR UPDATE SKIP LOCKED
        LIMIT 1
    )
    RETURNING *";

    let task: Option<PostgresTask> = sqlx::query_as(query)
        .bind(TaskState::Running)
        .bind(now)
        .fetch_optional(db)
        .await?;
    Ok(task)
}

pub async fn insert_task_to_queue(task: Task, db: &PgPool) -> Result<Uuid, sqlx::Error> {
    let postgres_task = PostgresTask::new_from_task(task);
    let query = "INSERT INTO queue (id, created_at, updated_at, scheduled_for, status, task_type) \
                 VALUES ($1, $2, $3, $4, $5, $6)";

    sqlx::query(query)
        .bind(postgres_task.id)
        .bind(postgres_task.created_at)
        .bind(postgres_task.updated_at)
        .bind(postgres_task.scheduled_for)
        .bind(postgres_task.status)
        .bind(postgres_task.task_type)
        .execute(db)
        .await?;

    Ok(postgres_task.id)
}

pub async fn remove_task(task_id: Uuid, db: &PgPool) -> Result<u64, sqlx::Error> {
    let query = "DELETE FROM queue WHERE id = $1";

    let rows_affected = sqlx::query(query)
        .bind(task_id)
        .execute(db)
        .await?
        .rows_affected();

    Ok(rows_affected)
}

pub async fn get_tasks_by_state(
    state: TaskState,
    db: &PgPool,
) -> Result<Vec<PostgresTask>, sqlx::Error> {
    let query = "SELECT * FROM queue WHERE status = $1";
    let tasks = sqlx::query_as(query).bind(state).fetch_all(db).await?;

    Ok(tasks)
}

pub async fn get_tasks_by_type(
    task_type: TaskType,
    db: &PgPool,
) -> Result<Vec<PostgresTask>, sqlx::Error> {
    let query = "SELECT * FROM queue WHERE task_type = $1";
    let tasks = sqlx::query_as(query).bind(task_type).fetch_all(db).await?;

    Ok(tasks)
}

pub async fn get_task_by_id(
    task_id: Uuid,
    db: &PgPool,
) -> Result<Option<PostgresTask>, sqlx::Error> {
    let query = "SELECT * FROM queue WHERE id = $1";
    let task: Option<PostgresTask> = sqlx::query_as(query)
        .bind(task_id)
        .fetch_optional(db)
        .await?;

    Ok(task)
}

pub async fn get_all_tasks(db: &PgPool) -> Result<Vec<PostgresTask>, sqlx::Error> {
    let query = "SELECT * FROM queue";
    let tasks = sqlx::query_as(query).fetch_all(db).await?;

    Ok(tasks)
}

pub async fn queue_hanging_tasks(db: &PgPool) -> Result<(), sqlx::Error> {
    let now = OffsetDateTime::now_utc();
    let query = "UPDATE queue SET status = $1, updated_at = $2 WHERE id IN (
        SELECT id FROM queue WHERE 
        status = $3 AND updated_at <= $4 - interval '1 minute'
        FOR UPDATE SKIP LOCKED
    )";

    sqlx::query(query)
        .bind(TaskState::Queued)
        .bind(now)
        .bind(TaskState::Running)
        .bind(now)
        .execute(db)
        .await?;

    Ok(())
}
