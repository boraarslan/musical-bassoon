use std::time::Duration;

use crate::{
    redis::{check_scheduled_tasks, get_task_from_queue, RedisPool},
    task::TaskType,
};

pub struct Worker {
    db: RedisPool,
}

impl Worker {
    pub fn new(db: &RedisPool) -> Self {
        Worker { db: db.clone() }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let next_task = get_task_from_queue(&self.db).await?;

            if next_task.is_none() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            let task = next_task.unwrap();
            match task.task_type {
                TaskType::Fizz => {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    println!("Fizz {}", task.id)
                }
                TaskType::Buzz => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    println!("Buzz {}", task.id)
                }
                TaskType::FizzBuzz => {
                    tokio::time::sleep(Duration::from_secs(15)).await;
                    println!("FizzBuzz {}", task.id)
                }
            }
        }
    }
}

pub struct ScheduledTaskManager {
    db: RedisPool,
}

impl ScheduledTaskManager {
    pub fn new(db: &RedisPool) -> Self {
        ScheduledTaskManager { db: db.clone() }
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        check_scheduled_tasks(&self.db).await
    }
}
