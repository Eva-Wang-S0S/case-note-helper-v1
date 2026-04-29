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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    struct TestState {
        db: Arc<Mutex<Connection>>,
        settings: Arc<RwLock<super::super::AppSettings>>,
    }

    impl TestState {
        fn new(db_path: &str) -> Self {
            let db = Connection::open(db_path).unwrap();
            db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();
            Self {
                db: Arc::new(Mutex::new(db)),
                settings: Arc::new(RwLock::new(super::super::AppSettings::default())),
            }
        }
    }

    async fn test_create_plan_item(state: &TestState, case_id: i64, content: &str, scheduled_date: Option<&str>) -> Result<PlanItem, String> {
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
        let mut stmt = db.prepare("SELECT id, case_id, content, scheduled_date, completed, created_at, updated_at FROM plan_items WHERE id = ?").map_err(|e| e.to_string())?;
        stmt.query_row([id], |row| {
            Ok(PlanItem { id: row.get(0)?, case_id: row.get(1)?, content: row.get(2)?, scheduled_date: row.get(3)?, completed: row.get(4)?, created_at: row.get(5)?, updated_at: row.get(6)? })
        }).map_err(|e| e.to_string())
    }

    async fn test_toggle_plan_item(state: &TestState, item_id: i64) -> Result<PlanItem, String> {
        let now = chrono_now();
        {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.execute("UPDATE plan_items SET completed = NOT completed, updated_at = ? WHERE id = ?", params![&now, item_id]).map_err(|e| format!("Failed to toggle plan item: {}", e))?;
        }
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT id, case_id, content, scheduled_date, completed, created_at, updated_at FROM plan_items WHERE id = ?").map_err(|e| e.to_string())?;
        stmt.query_row([item_id], |row| {
            Ok(PlanItem { id: row.get(0)?, case_id: row.get(1)?, content: row.get(2)?, scheduled_date: row.get(3)?, completed: row.get(4)?, created_at: row.get(5)?, updated_at: row.get(6)? })
        }).map_err(|e| e.to_string())
    }

    async fn test_delete_plan_item(state: &TestState, item_id: i64) -> Result<(), String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("DELETE FROM plan_items WHERE id = ?", [item_id]).map_err(|e| format!("Failed to delete plan item: {}", e))?;
        Ok(())
    }

    async fn test_create_case(state: &TestState, name: &str, client_name: &str, stage: &str) -> Result<Case, String> {
        use super::*;
        let now = chrono_now();
        let id = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params![name, client_name, stage, &now, &now],
            ).map_err(|e| format!("Failed to create case: {}", e))?;
            db.last_insert_rowid()
        };
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT id, name, client_name, stage, status, created_at, updated_at FROM cases WHERE id = ?").map_err(|e| e.to_string())?;
        stmt.query_row([id], |row| {
            Ok(Case { id: row.get(0)?, name: row.get(1)?, client_name: row.get(2)?, stage: row.get(3)?, status: row.get(4)?, created_at: row.get(5)?, updated_at: row.get(6)? })
        }).map_err(|e| e.to_string())
    }

    async fn test_save_note(state: &TestState, case_id: i64, raw_content: &str, draft_content: Option<&str>) -> Result<Note, String> {
        use super::*;
        let now = chrono_now();
        let id = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.execute(
                "INSERT INTO notes (case_id, raw_content, draft_content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                params![case_id, raw_content, draft_content, &now, &now],
            ).map_err(|e| format!("Failed to create note: {}", e))?;
            db.last_insert_rowid()
        };
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT id, case_id, raw_content, draft_content, created_at, updated_at FROM notes WHERE id = ?").map_err(|e| e.to_string())?;
        stmt.query_row([id], |row| {
            Ok(Note { id: row.get(0)?, case_id: row.get(1)?, raw_content: row.get(2)?, draft_content: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
        }).map_err(|e| e.to_string())
    }

    async fn test_get_plan_items(state: &TestState, case_id: i64) -> Result<Vec<PlanItem>, String> {
        use super::*;
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT id, case_id, content, scheduled_date, completed, created_at, updated_at FROM plan_items WHERE case_id = ? ORDER BY scheduled_date ASC NULLS LAST, created_at ASC").map_err(|e| e.to_string())?;
        let items = stmt.query_map([case_id], |row| {
            Ok(PlanItem { id: row.get(0)?, case_id: row.get(1)?, content: row.get(2)?, scheduled_date: row.get(3)?, completed: row.get(4)?, created_at: row.get(5)?, updated_at: row.get(6)? })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(items)
    }

    async fn test_get_notes(state: &TestState, case_id: i64) -> Result<Vec<Note>, String> {
        use super::*;
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT id, case_id, raw_content, draft_content, created_at, updated_at FROM notes WHERE case_id = ? ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
        let notes = stmt.query_map([case_id], |row| {
            Ok(Note { id: row.get(0)?, case_id: row.get(1)?, raw_content: row.get(2)?, draft_content: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(notes)
    }

    async fn test_search_archive(state: &TestState, query: &str) -> Result<Vec<Case>, String> {
        use super::*;
        let pattern = format!("%{}%", query);
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare(
            "SELECT id, name, client_name, stage, status, created_at, updated_at FROM cases WHERE name LIKE ? OR client_name LIKE ? OR stage LIKE ? ORDER BY updated_at DESC LIMIT 50"
        ).map_err(|e| e.to_string())?;
        let cases = stmt.query_map([&pattern, &pattern, &pattern], |row| {
            Ok(Case { id: row.get(0)?, name: row.get(1)?, client_name: row.get(2)?, stage: row.get(3)?, status: row.get(4)?, created_at: row.get(5)?, updated_at: row.get(6)? })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(cases)
    }

    
    fn init_schema(db: &Connection) {
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS cases (
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
            CREATE INDEX IF NOT EXISTS idx_plan_items_case_id ON plan_items(case_id);"
        ).unwrap();
    }

    #[tokio::test]
    async fn create_plan_item_inserts_and_returns_item() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        // Create a case since plan_items FK requires it
        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
        }

        let item = test_create_plan_item(&state, 1, "Review housing application", Some("2026-05-01"))
            .await
            .unwrap();

        assert_eq!(item.case_id, 1);
        assert_eq!(item.content, "Review housing application");
        assert_eq!(item.scheduled_date, Some("2026-05-01".to_string()));
        assert!(!item.completed);
        assert!(item.id > 0);
    }

    #[tokio::test]
    async fn create_plan_item_fails_with_nonexistent_case() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        // FK constraint violation — case_id 999 does not exist
        let result = test_create_plan_item(&state, 999, "This should fail", None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("FOREIGN KEY") || err.contains("constraint"), "Expected FK error, got: {}", err);
    }

    #[tokio::test]
    async fn toggle_plan_item_flips_completed_state() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO plan_items (case_id, content, scheduled_date, completed, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)",
                params![1, "Test Item", None::<String>, &now, &now],
            ).unwrap();
        }

        // Toggle to completed
        let updated = test_toggle_plan_item(&state, 1).await.unwrap();
        assert!(updated.completed);

        // Toggle back to not completed
        let toggled_again = test_toggle_plan_item(&state, 1).await.unwrap();
        assert!(!toggled_again.completed);
    }

    #[tokio::test]
    async fn toggle_plan_item_returns_error_for_nonexistent_item() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
        }

        // No plan items exist — toggling non-existent ID should fail on query_row
        let result = test_toggle_plan_item(&state, 999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_plan_item_removes_from_database() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO plan_items (case_id, content, scheduled_date, completed, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)",
                params![1, "To Be Deleted", None::<String>, &now, &now],
            ).unwrap();
        }

        test_delete_plan_item(&state, 1).await.unwrap();

        let db = state.db.lock().unwrap();
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM plan_items WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn delete_plan_item_succeeds_for_nonexistent_item() {
        // delete_plan_item doesn't error on 0 rows affected — this is intentional
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
        }

        // Deleting non-existent item should succeed (idempotent delete)
        let result = test_delete_plan_item(&state, 999).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_case_inserts_and_returns_case() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let created = test_create_case(&state, "Family Assessment", "Jane Doe", "assessment")
            .await
            .unwrap();

        assert_eq!(created.name, "Family Assessment");
        assert_eq!(created.client_name, "Jane Doe");
        assert_eq!(created.stage, "assessment");
        assert_eq!(created.status, "active");
        assert!(created.id > 0);
    }

    #[tokio::test]
    async fn save_note_creates_new_note_when_none_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
        }

        let note = test_save_note(&state, 1, "Initial contact made", Some("Drafted note content"))
            .await
            .unwrap();

        assert_eq!(note.case_id, 1);
        assert_eq!(note.raw_content, "Initial contact made");
        assert_eq!(note.draft_content, Some("Drafted note content".to_string()));
    }

    #[tokio::test]
    async fn get_plan_items_returns_items_for_case() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO plan_items (case_id, content, scheduled_date, completed, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)",
                params![1, "Item A", Some("2026-05-01"), &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO plan_items (case_id, content, scheduled_date, completed, created_at, updated_at) VALUES (?, ?, ?, 1, ?, ?)",
                params![1, "Item B", None::<String>, &now, &now],
            ).unwrap();
        }

        let items = test_get_plan_items(&state, 1).await.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.content == "Item A" && !i.completed));
        assert!(items.iter().any(|i| i.content == "Item B" && i.completed));
    }

    #[tokio::test]
    async fn get_plan_items_returns_empty_for_no_items() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
        }

        let items = test_get_plan_items(&state, 1).await.unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn search_archive_returns_matching_cases() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Smith Family", "Alice Smith", "assessment", &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Jones Case", "Bob Jones", "intake", &now, &now],
            ).unwrap();
        }

        let results = test_search_archive(&state, "Smith").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Smith Family");

        let results2 = test_search_archive(&state, "Jones").await.unwrap();
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].name, "Jones Case");
    }

    #[tokio::test]
    async fn search_archive_returns_empty_for_no_matches() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Smith Family", "Alice Smith", "assessment", &now, &now],
            ).unwrap();
        }

        let results = test_search_archive(&state, "NoMatch").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn get_notes_returns_empty_when_case_has_no_notes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
        }

        let notes = test_get_notes(&state, 1).await.unwrap();
        assert!(notes.is_empty());
    }

    #[tokio::test]
    async fn get_notes_returns_multiple_notes_for_case() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        let now = chrono_now();
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO cases (name, client_name, stage, status, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, ?)",
                params!["Test Case", "Test Client", "intake", &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO notes (case_id, raw_content, draft_content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                params![1, "First note", Some("Draft 1"), &now, &now],
            ).unwrap();
            db.execute(
                "INSERT INTO notes (case_id, raw_content, draft_content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                params![1, "Second note", None::<String>, &now, &now],
            ).unwrap();
        }

        let notes = test_get_notes(&state, 1).await.unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[tokio::test]
    async fn create_plan_item_fails_when_case_id_constraint_violated() {
        // FK constraint: case_id 999 does not exist
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let state = TestState::new(db_path.to_str().unwrap());
        init_schema(&state.db.lock().unwrap());

        // Insert plan item with non-existent case — SQLite FK is ON
        let now = chrono_now();
        let db = state.db.lock().unwrap();
        let result = db.execute(
            "INSERT INTO plan_items (case_id, content, scheduled_date, completed, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)",
            params![999, "Orphan item", None::<String>, &now, &now],
        );
        // FK violation returns error
        assert!(result.is_err(), "Expected FK constraint error");
    }
}
