use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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
    pub task_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTaskRequest {
    pub task_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u64,
    pub task_type: TaskType,
    pub scheduled_time: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskType {
    Fizz,
    Buzz,
    FizzBuzz,
}

impl From<CreateTaskRequest> for Task {
    fn from(request: CreateTaskRequest) -> Self {
        let id = rand::random();
        let current = OffsetDateTime::now_utc().unix_timestamp();

        let scheduled_time = current
            + request.exec_time_seconds.unwrap_or(0)
            + (request.exec_time_minutes.unwrap_or(0) * 60)
            + (request.exec_time_hours.unwrap_or(0) * 60 * 60);

        let scheduled_time = if current == scheduled_time {
            None
        } else {
            Some(scheduled_time)
        };

        Task {
            id,
            task_type: request._type,
            scheduled_time,
        }
    }
}
