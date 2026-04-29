use serde::{Deserialize, Serialize};
use tauri::Manager;

mod llm;
mod db;

pub use llm::*;
pub use db::{AppState, Case, Note, PlanItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_provider: String,
    pub llm_endpoint: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub redaction_list: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_provider: "ollama".to_string(),
            llm_endpoint: "http://localhost:11434/v1/chat/completions".to_string(),
            llm_api_key: "".to_string(),
            llm_model: "llama3.2".to_string(),
            redaction_list: vec![
                "[REDACTED NAME]".to_string(),
                "[REDACTED ORG]".to_string(),
                "[REDACTED LOCATION]".to_string(),
            ],
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
    let mut settings_lock = state.settings.write().await;
    *settings_lock = settings;
    Ok(())
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

    drop(settings);

    let result = llm::draft_note(&state, &input).await?;
    Ok(result)
}

#[tauri::command]
async fn get_cases(app: tauri::AppHandle) -> Result<Vec<Case>, String> {
    let state = app.state::<AppState>();
    db::get_all_cases(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_case(app: tauri::AppHandle, case_id: i64) -> Result<Case, String> {
    let state = app.state::<AppState>();
    db::get_case(&state, case_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_case(app: tauri::AppHandle, name: String, client_name: String, stage: String) -> Result<Case, String> {
    let state = app.state::<AppState>();
    db::create_case(&state, &name, &client_name, &stage)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_notes(app: tauri::AppHandle, case_id: i64) -> Result<Vec<Note>, String> {
    let state = app.state::<AppState>();
    db::get_notes(&state, case_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_note(
    app: tauri::AppHandle,
    case_id: i64,
    raw_content: String,
    draft_content: Option<String>,
) -> Result<Note, String> {
    let state = app.state::<AppState>();
    db::save_note(&state, case_id, &raw_content, draft_content.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_archive(app: tauri::AppHandle, query: String) -> Result<Vec<Case>, String> {
    let state = app.state::<AppState>();
    db::search_archive(&state, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_plan_items(app: tauri::AppHandle, case_id: i64) -> Result<Vec<PlanItem>, String> {
    let state = app.state::<AppState>();
    db::get_plan_items(&state, case_id)
        .await
        .map_err(|e| e.to_string())
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
            let app_handle = app.handle().clone();
            let state = AppState::new(app_handle);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            draft_case_note,
            get_cases,
            get_case,
            create_case,
            get_notes,
            save_note,
            search_archive,
            get_plan_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}