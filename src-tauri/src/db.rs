use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
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
    pub db: Arc<Mutex<Connection>>,
    pub settings: Arc<RwLock<super::AppSettings>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle, db_path: &str) -> Result<Self, String> {
        let db = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to configure database: {}", e))?;
        Ok(Self {
            app_handle,
            db: Arc::new(Mutex::new(db)),
            settings: Arc::new(RwLock::new(super::AppSettings::default())),
        })
    }
}

pub async fn init_database(state: &AppState) -> Result<(), String> {
    log::info!("Initializing database schema");

    let db = state.db.lock().map_err(|e| format!("DB lock poisoned: {}", e))?;
    db.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS cases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            client_name TEXT NOT NULL,
            stage TEXT NOT NULL DEFAULT 'intake',
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            case_id INTEGER NOT NULL,
            raw_content TEXT NOT NULL DEFAULT '',
            draft_content TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS plan_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            case_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            scheduled_date TEXT,
            completed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_notes_case_id ON notes(case_id);
        CREATE INDEX IF NOT EXISTS idx_plan_items_case_id ON plan_items(case_id);
        ",
    ).map_err(|e| format!("Failed to create schema: {}", e))?;

    drop(db);
    log::info!("Database schema initialized");
    Ok(())
}

pub async fn get_all_cases(state: &AppState) -> Result<Vec<Case>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, name, client_name, stage, status, created_at, updated_at FROM cases ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let cases = stmt
        .query_map([], |row| {
            Ok(Case {
                id: row.get(0)?,
                name: row.get(1)?,
                client_name: row.get(2)?,
                stage: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(cases)
}

pub async fn get_case(state: &AppState, case_id: i64) -> Result<Case, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, name, client_name, stage, status, created_at, updated_at FROM cases WHERE id = ?")
        .map_err(|e| e.to_string())?;

    stmt.query_row([case_id], |row| {
        Ok(Case {
            id: row.get(0)?,
            name: row.get(1)?,
            client_name: row.get(2)?,
            stage: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| format!("Case {} not found: {}", case_id, e))
}

pub async fn create_case(
    state: &AppState,
    name: &str,
    client_name: &str,
    stage: &str,
) -> Result<Case, String> {
    let now = chrono_now();
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
            params![name, client_name, stage, &now, &now],
        ).map_err(|e| format!("Failed to create case: {}", e))?;
    }

    let id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.last_insert_rowid()
    };

    log::info!("Created case {} with id {}", name, id);
    get_case(state, id).await
}

pub async fn get_notes(state: &AppState, case_id: i64) -> Result<Vec<Note>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, case_id, raw_content, draft_content, created_at, updated_at FROM notes WHERE case_id = ? ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let notes = stmt
        .query_map([case_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                case_id: row.get(1)?,
                raw_content: row.get(2)?,
                draft_content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

pub async fn save_note(
    state: &AppState,
    case_id: i64,
    raw_content: &str,
    draft_content: Option<&str>,
) -> Result<Note, String> {
    let now = chrono_now();
    let id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;

        // Check if note exists for this case
        let existing: Option<i64> = db
            .query_row(
                "SELECT id FROM notes WHERE case_id = ? ORDER BY id DESC LIMIT 1",
                [case_id],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_id) = existing {
            db.execute(
                "UPDATE notes SET raw_content = ?, draft_content = ?, updated_at = ? WHERE id = ?",
                params![raw_content, draft_content, &now, existing_id],
            ).map_err(|e| format!("Failed to update note: {}", e))?;
            existing_id
        } else {
            db.execute(
                "INSERT INTO notes (case_id, raw_content, draft_content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                params![case_id, raw_content, draft_content, &now, &now],
            ).map_err(|e| format!("Failed to create note: {}", e))?;
            db.last_insert_rowid()
        }
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, case_id, raw_content, draft_content, created_at, updated_at FROM notes WHERE id = ?")
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], |row| {
        Ok(Note {
            id: row.get(0)?,
            case_id: row.get(1)?,
            raw_content: row.get(2)?,
            draft_content: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())
}

pub async fn search_archive(state: &AppState, query: &str) -> Result<Vec<Case>, String> {
    let pattern = format!("%{}%", query);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare(
            "SELECT id, name, client_name, stage, status, created_at, updated_at
             FROM cases
             WHERE name LIKE ? OR client_name LIKE ? OR stage LIKE ?
             ORDER BY updated_at DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let cases = stmt
        .query_map([&pattern, &pattern, &pattern], |row| {
            Ok(Case {
                id: row.get(0)?,
                name: row.get(1)?,
                client_name: row.get(2)?,
                stage: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(cases)
}

pub async fn get_plan_items(state: &AppState, case_id: i64) -> Result<Vec<PlanItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, case_id, content, scheduled_date, completed, created_at, updated_at FROM plan_items WHERE case_id = ? ORDER BY scheduled_date ASC NULLS LAST, created_at ASC")
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([case_id], |row| {
            Ok(PlanItem {
                id: row.get(0)?,
                case_id: row.get(1)?,
                content: row.get(2)?,
                scheduled_date: row.get(3)?,
                completed: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

Ok(items)
}

pub async fn create_plan_item(
    state: &AppState,
    case_id: i64,
    content: &str,
    scheduled_date: Option<&str>,
) -> Result<PlanItem, String> {
    let now = chrono_now();
    let id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO plan_items (case_id, content, scheduled_date, completed, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)",
            params![case_id, content, scheduled_date, &now, &now],
        ).map_err(|e| format!("Failed to create plan item: {}", e))?;
        db.last_insert_rowid()
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, case_id, content, scheduled_date, completed, created_at, updated_at FROM plan_items WHERE id = ?")
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], |row| {
        Ok(PlanItem {
            id: row.get(0)?,
            case_id: row.get(1)?,
            content: row.get(2)?,
            scheduled_date: row.get(3)?,
            completed: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())
}

pub async fn toggle_plan_item(state: &AppState, item_id: i64) -> Result<PlanItem, String> {
    let now = chrono_now();
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE plan_items SET completed = NOT completed, updated_at = ? WHERE id = ?",
            params![&now, item_id],
        ).map_err(|e| format!("Failed to toggle plan item: {}", e))?;
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, case_id, content, scheduled_date, completed, created_at, updated_at FROM plan_items WHERE id = ?")
        .map_err(|e| e.to_string())?;

    stmt.query_row([item_id], |row| {
        Ok(PlanItem {
            id: row.get(0)?,
            case_id: row.get(1)?,
            content: row.get(2)?,
            scheduled_date: row.get(3)?,
            completed: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())
}

pub async fn delete_plan_item(state: &AppState, item_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM plan_items WHERE id = ?", [item_id])
        .map_err(|e| format!("Failed to delete plan item: {}", e))?;
    Ok(())
}

pub async fn load_settings_from_db(state: &AppState) -> Result<(), String> {
    let settings_json: Option<String> = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT value FROM settings WHERE key = 'app_settings'")
            .map_err(|e| e.to_string())?;

        stmt.query_row([], |row| row.get::<_, String>(0)).ok()
    };

    if let Some(json) = settings_json {
        if let Ok(settings) = serde_json::from_str::<super::AppSettings>(&json) {
            let mut locked = state.settings.write().await;
            *locked = settings;
            log::info!("Loaded settings from database");
        }
    }
    Ok(())
}

pub async fn save_settings_to_db(state: &AppState, settings: &super::AppSettings) -> Result<(), String> {
    let json = serde_json::to_string(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('app_settings', ?)",
        [&json],
    ).map_err(|e| format!("Failed to save settings: {}", e))?;

    log::info!("Saved settings to database");
    Ok(())
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
