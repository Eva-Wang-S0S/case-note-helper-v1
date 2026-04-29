use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: i64,
    pub name: String,
    pub client_name: String,
    pub stage: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub case_id: i64,
    pub raw_content: String,
    pub draft_content: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: i64,
    pub case_id: i64,
    pub content: String,
    pub scheduled_date: Option<String>,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct AppState {
    pub app_handle: AppHandle,
    pub settings: Arc<RwLock<super::AppSettings>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            settings: Arc::new(RwLock::new(super::AppSettings::default())),
        }
    }
}

pub async fn init_database(_state: &AppState) -> Result<(), String> {
    log::info!("Initializing database schema");
    Ok(())
}

pub async fn get_all_cases(_state: &AppState) -> Result<Vec<Case>, String> {
    log::info!("Fetching all cases");
    Ok(vec![])
}

pub async fn get_case(_state: &AppState, case_id: i64) -> Result<Case, String> {
    log::info!("Fetching case {}", case_id);
    Err(format!("Case {} not found", case_id))
}

pub async fn create_case(
    _state: &AppState,
    name: &str,
    client_name: &str,
    stage: &str,
) -> Result<Case, String> {
    log::info!("Creating case: {} for client {}", name, client_name);
    Ok(Case {
        id: 1,
        name: name.to_string(),
        client_name: client_name.to_string(),
        stage: stage.to_string(),
        status: "active".to_string(),
        created_at: chrono_now(),
        updated_at: chrono_now(),
    })
}

pub async fn get_notes(
    _state: &AppState,
    case_id: i64,
) -> Result<Vec<Note>, String> {
    log::info!("Fetching notes for case {}", case_id);
    Ok(vec![])
}

pub async fn save_note(
    _state: &AppState,
    case_id: i64,
    raw_content: &str,
    draft_content: Option<&str>,
) -> Result<Note, String> {
    log::info!("Saving note for case {}", case_id);
    Ok(Note {
        id: 1,
        case_id,
        raw_content: raw_content.to_string(),
        draft_content: draft_content.map(String::from),
        created_at: chrono_now(),
        updated_at: chrono_now(),
    })
}

pub async fn search_archive(
    _state: &AppState,
    query: &str,
) -> Result<Vec<Case>, String> {
    log::info!("Searching archive with query: {}", query);
    Ok(vec![])
}

pub async fn get_plan_items(
    _state: &AppState,
    case_id: i64,
) -> Result<Vec<PlanItem>, String> {
    log::info!("Fetching plan items for case {}", case_id);
    Ok(vec![])
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}