use std::time::Duration;

use anyhow::bail;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use time::OffsetDateTime;

use crate::task::Task;

pub type RedisPool = Pool<RedisConnectionManager>;
static QUEUE_KEY: &str = "queue";
static SCHEDULED_KEY: &str = "scheduled";

pub async fn schedule_task(task: Task, db: &RedisPool) -> anyhow::Result<()> {
    assert!(task.scheduled_time.is_some());
    let mut conn = db.get().await?;
    let mut cmd = redis::Cmd::new();
    let task_json = serde_json::to_string(&task)?;
    cmd.arg("ZADD")
        .arg(SCHEDULED_KEY)
        .arg(task.scheduled_time.unwrap())
        .arg(task_json);
    _ = cmd.query_async(&mut *conn).await?;
    Ok(())
}

pub async fn get_task_from_queue(db: &RedisPool) -> anyhow::Result<Option<Task>> {
    let mut conn = db.get().await?;
    let mut cmd = redis::Cmd::new();
    cmd.arg("RPOP").arg(QUEUE_KEY);

    let result: Option<String> = cmd.query_async(&mut *conn).await?;

    match result {
        Some(json_str) => {
            let task = serde_json::from_str::<Task>(&json_str)?;
            Ok(Some(task))
        }
        None => Ok(None),
    }
}

pub async fn insert_task_to_queue(task: Task, db: &RedisPool) -> anyhow::Result<()> {
    let mut conn = db.get().await?;
    let mut cmd = redis::Cmd::new();
    let task_json = serde_json::to_string(&task)?;
    cmd.arg("LPUSH").arg(QUEUE_KEY).arg(task_json);

    _ = cmd.query_async(&mut *conn).await?;

    Ok(())
}

/// Returns queued and scheduled tasks.
pub async fn fetch_all_tasks(db: &RedisPool) -> anyhow::Result<(Vec<Task>, Vec<Task>)> {
    let mut conn = db.get().await?;
    let queue_items: Vec<String> = redis::cmd("LRANGE")
        .arg(QUEUE_KEY)
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await?;
    let scheduled_items: Vec<String> = redis::cmd("ZRANGE")
        .arg(SCHEDULED_KEY)
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await?;

    let queue_items: Vec<_> = queue_items
        .iter()
        .flat_map(|task_string| serde_json::from_str(task_string.as_str()))
        .collect();
    let scheduled_items: Vec<_> = scheduled_items
        .iter()
        .flat_map(|task_string| serde_json::from_str(task_string.as_str()))
        .collect();
    Ok((queue_items, scheduled_items))
}

pub async fn fetch_task(task_id: u64, db: &RedisPool) -> anyhow::Result<Task> {
    let (mut queued, mut scheduled) = fetch_all_tasks(&db).await?;
    queued.append(&mut scheduled);
    let task = queued.iter().find(|db_task| db_task.id == task_id);

    match task {
        Some(task) => Ok(task.clone()),
        None => bail!("Task with given ID does not exist in database."),
    }
}

pub async fn remove_task(task_id: u64, db: &RedisPool) -> anyhow::Result<()> {
    let (queued, scheduled) = fetch_all_tasks(db).await?;
    let mut conn = db.get().await?;
    let remove_count = if let Some(task) = queued.iter().find(|db_task| db_task.id == task_id) {
        let task_json = serde_json::to_string(task)?;
        let remove_count: i32 = redis::cmd("LREM")
            .arg(QUEUE_KEY)
            .arg(0)
            .arg(task_json)
            .query_async(&mut *conn)
            .await?;
        remove_count
    } else if let Some(task) = scheduled.iter().find(|db_task| db_task.id == task_id) {
        let task_json = serde_json::to_string(task)?;
        let remove_count: i32 = redis::cmd("ZREM")
            .arg(SCHEDULED_KEY)
            .arg(task_json)
            .query_async(&mut *conn)
            .await?;
        remove_count
    } else {
        bail!("Task with given ID does not exist in database.")
    };

    if remove_count == 0 {
        bail!("Failed to remove task from queue.")
    }

    Ok(())
}

// This function is called inside [ScheduledTaskManager] to move tasks from the sorted set to
// task queue. In the case of a multiple [ScheduledTaskManager] being alive for a single database,
// removing from the set is being done with transactions.
pub async fn check_scheduled_tasks(db: &RedisPool) -> anyhow::Result<()> {
    let mut conn = db.get().await?;
    loop {
        redis::cmd("WATCH").arg(SCHEDULED_KEY).query_async(&mut *conn).await?;

        let mut cmd = redis::Cmd::new();
        cmd.arg("ZRANGE").arg(SCHEDULED_KEY).arg(0).arg(0);
        let task_string: Vec<String> = cmd.query_async(&mut *conn).await?;

        if task_string.is_empty() {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        let task_string = task_string[0].clone();

        let task: Task = serde_json::from_str(&task_string)?;

        let current_time = OffsetDateTime::now_utc().unix_timestamp();
        // Only scheduled tasks are stored in the ZSET
        if task.scheduled_time.unwrap() < current_time {
            redis::cmd("MULTI").query_async(&mut *conn).await?;
            redis::cmd("LPUSH")
                .arg(QUEUE_KEY)
                .arg(&task_string)
                .query_async(&mut *conn)
                .await?;
            redis::cmd("ZREM")
                .arg(SCHEDULED_KEY)
                .arg(&task_string)
                .query_async(&mut *conn)
                .await?;
            redis::cmd("EXEC").query_async(&mut *conn).await?;
        }


        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
