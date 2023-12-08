# Flo

API clients made easy using Rust proc macros:

```rs
use reqwest::Error;
use serde::{Serialize, Deserialize};
use flo::api;
use async_trait::async_trait;

#[derive(Serialize, Deserialize)]
struct Task {
    description: String,
}

#[api(TaskApiClient)]
#[async_trait]
trait TaskApi {
    /// Returns all the recently created tasks.
    #[get("/tasks")]
    async fn get_tasks(
        &self,
        #[query_param] page: Option<i32>,
    ) -> Result<Vec<Task>, Error>;

    /// Returns information about a task with the given id.
    #[get("/tasks/{}")]
    async fn get_tasks(
        &self,
        #[path_param] task_id: i32,
    ) -> Result<Task, Error>;

    /// Creates a new task.
    #[post("/tasks")]
    async fn create_task(
        &self,
        #[body] task: Task,
    ) -> Result<(), Error>;

    /// Updates information of the task with the given id.
    #[put("/tasks/{}")]
    async fn update_task(
        &self,
        #[path_param] task_id: i32,
        #[body] task: Task,
    ) -> Result<(), Error>;

    /// Deletes a task with the given id.
    #[delete("/tasks/{}")]
    async fn delete_task(
        &self,
        #[path_param] task_id: i32,
    ) -> Result<(), Error>;
}

#[tokio::main]
async fn main() {
    let client = TaskApiClient::new("localhost:3000")
        .with_basic_auth("username", "password")
        .with_default_header("test", true);
    client.create_task(Task { description: "test".to_owned() }).await.unwrap();
    client.delete_task(1).await.unwrap();
}
```
