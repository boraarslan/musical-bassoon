use std::time::Duration;

use sqlx::PgPool;

use crate::database::{pull_task, queue_hanging_tasks, remove_task};
use crate::task::TaskType;

pub struct Worker {
    db: PgPool,
}

impl Worker {
    pub fn new(db: &PgPool) -> Self {
        Worker { db: db.clone() }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            let next_task = pull_task(&self.db).await?;

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

            remove_task(task.id, &self.db).await?;
        }
    }
}

pub struct TaskGarbageCollector {
    db: PgPool,
}

impl TaskGarbageCollector {
    pub fn new(db: &PgPool) -> Self {
        TaskGarbageCollector { db: db.clone() }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            queue_hanging_tasks(&self.db).await?;
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}
