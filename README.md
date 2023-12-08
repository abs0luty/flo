# Flo

API clients made easy using Rust proc macros:

```rs
use reqwest::{Error, Response};
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
        #[query_param] page: i32,
    ) -> Result<Response, Error>;

    /// Returns information about a task with the given id.
    #[get("/tasks/{}")]
    async fn get_task(
        &self,
        #[path_param] task_id: i32,
    ) -> Result<Response, Error>;

    /// Creates a new task.
    #[post("/tasks")]
    async fn create_task(
        &self,
        #[body] task: Task,
    ) -> Result<Response, Error>;

    /// Updates information of the task with the given id.
    #[put("/tasks/{}")]
    async fn update_task(
        &self,
        #[path_param] task_id: i32,
        #[body] task: Task,
    ) -> Result<Response, Error>;

    /// Deletes a task with the given id.
    #[delete("/tasks/{}")]
    async fn delete_task(
        &self,
        #[path_param] task_id: i32,
    ) -> Result<Response, Error>;
}

#[tokio::main]
async fn main() {
    let client = TaskApiClient::new("localhost:3000")
        .with_basic_auth("username", "password")
        .with_default_header("test", true);
    client.create_task(Task { description: "test".to_owned() }).await.unwrap();

    let tasks1 = client.get_tasks(0).await.unwrap().json::<Vec<Task>>();
    client.delete_task(0).await.unwrap();

    let tasks2 = client.get_tasks(0).await.unwrap().json::<Vec<Task>>();
    assert_ne!(tasks1, tasks2);
}
```
