use async_trait::async_trait;
use flo::api;
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Task {
    description: String,
}

#[api(TaskApiClient)]
#[async_trait]
trait TaskApi {
    #[get("/tasks")]
    async fn get_all_tasks(&self) -> Result<Response, Error>;

    #[post("/tasks")]
    async fn create_task(&self, #[body] task: Task) -> Result<Response, Error>;

    #[get("/tasks/{}")]
    async fn get_task(&self, #[path_param] task_id: i32) -> Result<Response, Error>;

    #[delete("/tasks/{}")]
    async fn delete_task(&self, #[path_param] task_id: i32) -> Result<Response, Error>;
}

#[tokio::main]
async fn main() {
    let client = TaskApiClient::new("localhost:3000").with_basic_auth("username", "password");
    let tasks = client.get_all_tasks().await.unwrap().json::<Vec<Task>>();
    client.delete_task(1).await.unwrap();
}
