use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TODOIST_API_BASE: &str = "https://api.todoist.com/rest/v2";

#[derive(Debug, thiserror::Error)]
pub enum TodoistError {
    #[error("Todoist API error: {0}")]
    Api(String),
    #[error("Unauthorized: Todoist token missing or invalid")]
    Unauthorized,
    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoistTask {
    pub id: String,
    pub content: String,
    pub description: String,
    pub completed: bool,
    pub due_date: Option<String>,
    pub due_datetime: Option<String>,
    pub priority: i32,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub content: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub content: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<i32>,
    pub completed: Option<bool>,
}

pub struct TodoistClient {
    client: Client,
    token: String,
    base_url: String,
}

impl TodoistClient {
    pub fn new(token: String) -> Self {
        Self::with_base_url(token, TODOIST_API_BASE.to_string())
    }

    pub fn with_base_url(token: String, base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, token, base_url }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn extract_retry_after(response: &reqwest::Response) -> u64 {
        response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(60)
    }

    pub async fn get_tasks(&self) -> Result<Vec<TodoistTask>, TodoistError> {
        let response = self
            .client
            .get(format!("{}/tasks", self.base_url))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let status = response.status();
        let retry_after = Self::extract_retry_after(&response);
        let body = response.text().await?;

        if status.as_u16() == 401 {
            return Err(TodoistError::Unauthorized);
        }

        if status.as_u16() == 429 {
            return Err(TodoistError::RateLimited(retry_after));
        }

        if !status.is_success() {
            return Err(TodoistError::Api(format!(
                "GET /tasks failed with {}: raw body: {}",
                status, body
            )));
        }

        let tasks: Vec<TodoistTask> = serde_json::from_str(&body)
            .map_err(|e| TodoistError::Api(format!("raw body: {}: {}", body, e)))?;
        Ok(tasks)
    }

    pub async fn get_tasks_by_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<TodoistTask>, TodoistError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let ids_param = ids.join(",");
        let url = format!("{}/tasks?ids={}", self.base_url, ids_param);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let status = response.status();
        let retry_after = Self::extract_retry_after(&response);
        let body = response.text().await?;

        if status.as_u16() == 401 {
            return Err(TodoistError::Unauthorized);
        }

        if status.as_u16() == 429 {
            return Err(TodoistError::RateLimited(retry_after));
        }

        if !status.is_success() {
            return Err(TodoistError::Api(format!(
                "GET /tasks?ids= failed with {}: raw body: {}",
                status, body
            )));
        }

        let tasks: Vec<TodoistTask> = serde_json::from_str(&body)
            .map_err(|e| TodoistError::Api(format!("raw body: {}: {}", body, e)))?;
        Ok(tasks)
    }

    pub async fn create_task(&self, req: CreateTaskRequest) -> Result<TodoistTask, TodoistError> {
        let response = self
            .client
            .post(format!("{}/tasks", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        let retry_after = Self::extract_retry_after(&response);
        let body = response.text().await?;

        if status.as_u16() == 401 {
            return Err(TodoistError::Unauthorized);
        }

        if status.as_u16() == 429 {
            return Err(TodoistError::RateLimited(retry_after));
        }

        if !status.is_success() {
            return Err(TodoistError::Api(format!(
                "POST /tasks failed with {}: raw body: {}",
                status, body
            )));
        }

        let task: TodoistTask = serde_json::from_str(&body)
            .map_err(|e| TodoistError::Api(format!("raw body: {}: {}", body, e)))?;
        Ok(task)
    }

    pub async fn update_task(
        &self,
        task_id: &str,
        req: UpdateTaskRequest,
    ) -> Result<(), TodoistError> {
        let response = self
            .client
            .post(format!("{}/tasks/{}", self.base_url, task_id))
            .header("Authorization", self.auth_header())
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        let retry_after = Self::extract_retry_after(&response);
        let body = response.text().await?;

        if status.as_u16() == 401 {
            return Err(TodoistError::Unauthorized);
        }

        if status.as_u16() == 404 {
            return Err(TodoistError::Api(format!(
                "Task {} not found in Todoist: raw body: {}",
                task_id, body
            )));
        }

        if status.as_u16() == 429 {
            return Err(TodoistError::RateLimited(retry_after));
        }

        if !status.is_success() {
            return Err(TodoistError::Api(format!(
                "POST /tasks/{} failed with {}: raw body: {}",
                task_id, status, body
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;

    fn test_token() -> String {
        "test-token-123".to_string()
    }

    #[tokio::test]
    async fn test_malformed_json_returns_err_with_context() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.path("/rest/v2/tasks");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("not valid json {{{");
        });

        let client = TodoistClient::with_base_url(test_token(), server.url("/rest/v2"));
        let result = client.get_tasks().await;
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("raw body:"),
            "error should contain 'raw body:', got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_401_returns_unauthorized_error() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.path("/rest/v2/tasks");
            then.status(401)
                .header("Content-Type", "application/json")
                .body(r#"{"error": "Unauthorized"}"#);
        });

        let client = TodoistClient::with_base_url(test_token(), server.url("/rest/v2"));
        let result = client.get_tasks().await;
        assert!(matches!(result, Err(TodoistError::Unauthorized)));
    }

    #[tokio::test]
    async fn test_429_honours_retry_after() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.path("/rest/v2/tasks");
            then.status(429)
                .header("Retry-After", "90")
                .header("Content-Type", "application/json")
                .body(r#"{"error": "Rate limit exceeded"}"#);
        });

        let client = TodoistClient::with_base_url(test_token(), server.url("/rest/v2"));
        let result = client.get_tasks().await;
        match result {
            Err(TodoistError::RateLimited(s)) => assert!(s >= 80),
            other => panic!("expected RateLimited, got {:?}", other),
        }
    }
}
