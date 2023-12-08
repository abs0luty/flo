use async_trait::async_trait;
use flo::api;

struct Task {}

#[api(TaskApiClient)]
#[async_trait]
trait TaskApi {
    #[get("/tasks")]
    async fn get_all_tasks(&self) -> Result<Vec<Task>, reqwest::Error>;

    #[post("/tasks")]
    async fn create_task(&self, #[body] task: Task) -> Result<(), reqwest::Error>;

    #[get("/tasks/{}")]
    async fn get_task(&self, #[path] task_id: i32) -> Result<Task, reqwest::Error>;

    #[delete("/tasks/{}")]
    async fn delete_task(&self, #[path] task_id: i32) -> Result<(), reqwest::Error>;
}

#[tokio::main]
async fn main() {
    let client = TaskApiClient::new("localhost:3000").with_basic_auth("username", "password");
    client.delete_task(1).await.unwrap();
}
