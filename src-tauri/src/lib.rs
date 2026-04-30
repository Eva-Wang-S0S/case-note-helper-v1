use serde::{Deserialize, Serialize};
use tauri::{Manager, AppHandle};
use chrono::Utc;
use std::time::Duration;

mod llm;
mod db;
mod todoist;

pub use llm::*;
pub use db::{AppState, Case, Note, PlanItem, PollState};
pub use todoist::{TodoistClient, TodoistError, TodoistTask, CreateTaskRequest, UpdateTaskRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_provider: String,
    pub llm_endpoint: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub redaction_list: Vec<String>,
    pub llm_note_prompt: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_provider: "ollama".to_string(),
            llm_endpoint: "http://localhost:11434/v1/chat/completions".to_string(),
            llm_api_key: "".to_string(),
            llm_model: "llama3.2".to_string(),
            redaction_list: vec![],
            llm_note_prompt: "You are a social worker assistant helping to draft case notes. \
Given the raw observations below, write a professional, structured case note. \
Use clear headings and bullet points where appropriate. \
Focus on facts, observations, and actions taken.".to_string(),
        }
    }
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().await;
    Ok((*settings).clone())
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let state = app.state::<AppState>();
    {
        let mut settings_lock = state.settings.write().await;
        *settings_lock = settings.clone();
    }
    db::save_settings_to_db(&state, &settings).await
}

#[tauri::command]
async fn draft_case_note(
    app: tauri::AppHandle,
    raw_input: String,
    anonymize: bool,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().await;

    let input = if anonymize {
        llm::apply_redaction(&raw_input, &settings.redaction_list)
    } else {
        raw_input.clone()
    };

    let note_prompt = settings.llm_note_prompt.clone();
    drop(settings);

    let result = llm::draft_note(&state, &input, &note_prompt).await?;
    Ok(result)
}

#[tauri::command]
async fn get_cases(app: tauri::AppHandle) -> Result<Vec<Case>, String> {
    let state = app.state::<AppState>();
    db::get_all_cases(&state).await
}

#[tauri::command]
async fn get_case(app: tauri::AppHandle, case_id: i64) -> Result<Case, String> {
    let state = app.state::<AppState>();
    db::get_case(&state, case_id).await
}

#[tauri::command]
async fn create_case(app: tauri::AppHandle, name: String, client_name: String, stage: String) -> Result<Case, String> {
    let state = app.state::<AppState>();
    db::create_case(&state, &name, &client_name, &stage).await
}

#[tauri::command]
async fn update_case_stage(app: tauri::AppHandle, case_id: i64, stage: String) -> Result<Case, String> {
    let state = app.state::<AppState>();
    db::update_case_stage(&state, case_id, &stage).await
}

#[tauri::command]
async fn delete_case(app: tauri::AppHandle, case_id: i64) -> Result<(), String> {
    let state = app.state::<AppState>();
    db::delete_case(&state, case_id).await
}

#[tauri::command]
async fn get_notes(app: tauri::AppHandle, case_id: i64) -> Result<Vec<Note>, String> {
    let state = app.state::<AppState>();
    db::get_notes(&state, case_id).await
}

#[tauri::command]
async fn save_note(
    app: tauri::AppHandle,
    case_id: i64,
    raw_content: String,
    draft_content: Option<String>,
) -> Result<Note, String> {
    let state = app.state::<AppState>();
    db::save_note(&state, case_id, &raw_content, draft_content.as_deref()).await
}

#[tauri::command]
async fn search_archive(app: tauri::AppHandle, query: String) -> Result<Vec<Case>, String> {
    let state = app.state::<AppState>();
    db::search_archive(&state, &query).await
}

#[tauri::command]
async fn get_plan_items(app: tauri::AppHandle, case_id: i64) -> Result<Vec<PlanItem>, String> {
    let state = app.state::<AppState>();
    db::get_plan_items(&state, case_id).await
}

#[tauri::command]
async fn create_plan_item(
    app: tauri::AppHandle,
    case_id: i64,
    content: String,
    scheduled_date: Option<String>,
) -> Result<PlanItem, String> {
    let state = app.state::<AppState>();
    db::create_plan_item(&state, case_id, &content, scheduled_date.as_deref()).await
}

#[tauri::command]
async fn toggle_plan_item(app: tauri::AppHandle, item_id: i64) -> Result<PlanItem, String> {
    let state = app.state::<AppState>();
    db::toggle_plan_item(&state, item_id).await
}

#[tauri::command]
async fn delete_plan_item(app: tauri::AppHandle, item_id: i64) -> Result<(), String> {
    let state = app.state::<AppState>();
    db::delete_plan_item(&state, item_id).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoistConnectionStatus {
    pub connected: bool,
    pub last_synced_at: Option<String>,
    pub rate_limited: bool,
    pub rate_limit_until: Option<String>,
}

#[tauri::command]
async fn get_todoist_tasks(app: tauri::AppHandle) -> Result<Vec<TodoistTask>, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().await;

    let token = settings.llm_api_key.clone();
    drop(settings);

    if token.is_empty() {
        return Err("Todoist API token not configured".to_string());
    }

    let client = TodoistClient::new(token);
    client.get_tasks().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_plan_item_toggle(
    app: tauri::AppHandle,
    item_id: i64,
) -> Result<PlanItem, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().await;

    let token = settings.llm_api_key.clone();
    drop(settings);

    // Get current item to find its todoist_task_id
    let item = db::toggle_plan_item(&state, item_id).await?;

    if let Some(todoist_id) = &item.todoist_task_id {
        let client = TodoistClient::new(token);
        let update = UpdateTaskRequest {
            content: None,
            description: None,
            due_date: None,
            priority: None,
            completed: Some(item.completed),
        };

        match client.update_task(todoist_id, update).await {
            Ok(_) => {
                let now = Utc::now();
                db::set_plan_item_sync_status(&state, item_id, "synced", Some(&now.to_rfc3339())).await?;
                {
                    let mut poll = state.poll_state.write().await;
                    poll.last_write_at = Some(now);
                }
            }
            Err(TodoistError::Unauthorized) => {
                db::set_plan_item_sync_status(&state, item_id, "pending_sync", None).await?;
                return Err("Todoist disconnected".to_string());
            }
            Err(TodoistError::RateLimited(s)) => {
                db::set_plan_item_sync_status(&state, item_id, "pending_sync", None).await?;
                let mut poll = state.poll_state.write().await;
                poll.rate_limit_until = Some(Utc::now() + chrono::Duration::seconds(s as i64));
                return Err(format!("Rate limited, retry in {}s", s));
            }
            Err(e) => {
                db::set_plan_item_sync_status(&state, item_id, "sync_error", None).await?;
                return Err(format!("Sync failed: {}", e));
            }
        }
    }

    db::get_plan_items(&state, item.case_id).await?
        .into_iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| "Item not found".to_string())
}

#[tauri::command]
async fn get_todoist_connection_status(app: tauri::AppHandle) -> Result<TodoistConnectionStatus, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().await;
    let token = settings.llm_api_key.clone();
    drop(settings);

    let poll = state.poll_state.read().await;
    let connected = !token.is_empty();
    let rate_limited = poll.rate_limit_until
        .map(|rt| rt > Utc::now())
        .unwrap_or(false);
    let rate_limit_until = poll.rate_limit_until
        .filter(|rt| *rt > Utc::now())
        .map(|rt| rt.to_rfc3339());

    Ok(TodoistConnectionStatus {
        connected,
        last_synced_at: None,
        rate_limited,
        rate_limit_until,
    })
}

#[tauri::command]
async fn set_todoist_connected(app: tauri::AppHandle, connected: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut poll = state.poll_state.write().await;
    if !connected {
        poll.rate_limit_until = None;
    }
    Ok(())
}

fn spawn_poll_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let poll_interval = Duration::from_secs(60);

        loop {
            tokio::time::sleep(poll_interval).await;

            let state = match app.try_state::<AppState>() {
                Some(s) => s,
                None => continue,
            };

            // Check rate limit
            {
                let poll = state.poll_state.read().await;
                if let Some(until) = poll.rate_limit_until {
                    if until > Utc::now() {
                        log::debug!("Poll skipped: rate limited until {}", until);
                        continue;
                    }
                }
            }

            let settings = state.settings.read().await;
            let token = settings.llm_api_key.clone();
            drop(settings);

            if token.is_empty() {
                continue;
            }

            // Get items with todoist_task_id
            let items = match db::get_plan_items_with_todoist_ids(&state).await {
                Ok(items) => items,
                Err(e) => {
                    log::error!("Poll: failed to get items: {}", e);
                    continue;
                }
            };

            if items.is_empty() {
                continue;
            }

            let now = Utc::now();
            let ids: Vec<String> = items
                .iter()
                .filter(|item| {
                    if let Some(last_write) = item.updated_at.parse::<i64>().ok() {
                        let last_write_dt = chrono::DateTime::from_timestamp(last_write, 0)
                            .unwrap_or_default();
                        let age = now.signed_duration_since(last_write_dt);
                        age.num_seconds() >= 120
                    } else {
                        true
                    }
                })
                .filter_map(|item| item.todoist_task_id.clone())
                .collect();

            if ids.is_empty() {
                continue;
            }

            let client = TodoistClient::new(token);
            let remote_tasks = match client.get_tasks_by_ids(&ids).await {
                Ok(tasks) => tasks,
                Err(TodoistError::RateLimited(s)) => {
                    log::warn!("Poll: rate limited for {}s", s);
                    let mut poll = state.poll_state.write().await;
                    poll.rate_limit_until = Some(Utc::now() + chrono::Duration::seconds(s as i64));
                    continue;
                }
                Err(TodoistError::Unauthorized) => {
                    log::warn!("Poll: Todoist unauthorized");
                    continue;
                }
                Err(e) => {
                    log::error!("Poll: failed to fetch tasks: {}", e);
                    continue;
                }
            };

            for item in &items {
                let remote = remote_tasks.iter().find(|t| Some(&t.id) == item.todoist_task_id.as_ref());
                match remote {
                    Some(task) => {
                        if task.completed != item.completed {
                            log::info!("Poll: updating local item {} to completed={}", item.id, task.completed);
                            // Update local completed state
                            let db = state.db.lock().map_err(|e| e.to_string());
                            if let Ok(db) = db {
                                let now_str = Utc::now().timestamp().to_string();
                                db.execute(
                                    "UPDATE plan_items SET completed = ?, updated_at = ? WHERE id = ?",
                                    rusqlite::params![task.completed, now_str, item.id],
                                ).ok();
                            }
                        }
                    }
                    None => {
                        // Task not returned means it was deleted in Todoist
                        log::info!("Poll: task {} not found in Todoist, clearing reference", item.id);
                        db::clear_todoist_task_id(&state, item.id).await.ok();
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting CaseHelper v0.1.0");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .build(),
        )
        .setup(|app| {
            log::info!("Setting up CaseHelper app state");

            let app_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;

            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create app data dir: {}", e))?;

            let db_path = app_dir.join("casehelper.db");
            log::info!("Database path: {:?}", db_path);

            let state = AppState::new(app.handle().clone(), db_path.to_str().unwrap())
                .map_err(|e| format!("Failed to create app state: {}", e))?;

            app.manage(state);

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                if let Err(e) = db::init_database(&state).await {
                    log::error!("Failed to initialize database: {}", e);
                }
                if let Err(e) = db::load_settings_from_db(&state).await {
                    log::error!("Failed to load settings from database: {}", e);
                }
            });

            spawn_poll_worker(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            draft_case_note,
            get_cases,
            get_case,
            create_case,
            update_case_stage,
            delete_case,
            get_notes,
            save_note,
            search_archive,
            get_plan_items,
            create_plan_item,
            toggle_plan_item,
            delete_plan_item,
            get_todoist_tasks,
            sync_plan_item_toggle,
            get_todoist_connection_status,
            set_todoist_connected,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
