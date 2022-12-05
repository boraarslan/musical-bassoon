use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    #[serde(rename = "type")]
    _type: TaskType,
    #[serde(rename = "exec_secs")]
    exec_time_seconds: Option<i64>,
    #[serde(rename = "exec_mins")]
    exec_time_minutes: Option<i64>,
    #[serde(rename = "exec_hours")]
    exec_time_hours: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ShowTaskRequest {
    pub task_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ShowTaskByStateRequest {
    pub state: TaskState,
}

#[derive(Debug, Deserialize)]
pub struct ShowTaskByTypeRequest {
    pub task_type: TaskType,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTaskRequest {
    pub task_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub task_type: TaskType,
    pub scheduled_time: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct PostgresTask {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub scheduled_for: OffsetDateTime,
    pub status: TaskState,
    pub task_type: TaskType,
}

impl PostgresTask {
    pub fn new_from_task(task: Task) -> Self {
        let id: Uuid = ulid::Ulid::new().into();
        let created_at = OffsetDateTime::now_utc();
        let updated_at = OffsetDateTime::now_utc();
        let scheduled_for = match task.scheduled_time {
            Some(scheduled_time) => scheduled_time,
            None => OffsetDateTime::now_utc(),
        };
        let status = match task.scheduled_time {
            Some(_) => TaskState::Scheduled,
            None => TaskState::Queued,
        };
        let task_type = task.task_type;

        PostgresTask {
            id,
            created_at,
            updated_at,
            scheduled_for,
            status,
            task_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, sqlx::Type)]
#[repr(i32)]
pub enum TaskType {
    Fizz = 1,
    Buzz = 2,
    FizzBuzz = 3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, sqlx::Type)]
#[repr(i32)]
pub enum TaskState {
    Running = 1,
    Queued = 2,
    Scheduled = 3,
}

impl From<CreateTaskRequest> for Task {
    fn from(request: CreateTaskRequest) -> Self {
        let current = OffsetDateTime::now_utc();
        let seconds = Duration::seconds(request.exec_time_seconds.unwrap_or(0));
        let minutes = Duration::minutes(request.exec_time_minutes.unwrap_or(0));
        let hours = Duration::hours(request.exec_time_hours.unwrap_or(0));

        let scheduled_time = current
            .saturating_add(seconds)
            .saturating_add(minutes)
            .saturating_add(hours);

        let scheduled_time = if current >= scheduled_time {
            None
        } else {
            Some(scheduled_time)
        };

        Task {
            task_type: request._type,
            scheduled_time,
        }
    }
}
