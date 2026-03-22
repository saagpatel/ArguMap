use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub type DbPool = Mutex<Connection>;

fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".argumap").join("argumap.db")
}

pub fn init_db() -> Result<DbPool, String> {
    let path = db_path();

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create ~/.argumap/: {e}"))?;
    }

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {e}"))?;

    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("Failed to set WAL mode: {e}"))?;

    // Enable foreign keys — required for ON DELETE CASCADE
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to enable foreign keys: {e}"))?;

    // Run migration 1 — initial schema (idempotent via IF NOT EXISTS)
    let migration1 = include_str!("../migrations/001_initial.sql");
    conn.execute_batch(migration1)
        .map_err(|e| format!("Failed to run migration 1: {e}"))?;

    // Run migration 2 (add strength column) — idempotent via column existence check
    let has_strength: bool = conn.prepare("SELECT strength FROM nodes LIMIT 0").is_ok();
    if !has_strength {
        let migration2 = include_str!("../migrations/002_add_strength.sql");
        conn.execute_batch(migration2)
            .map_err(|e| format!("Failed to run migration 2: {e}"))?;
    }

    Ok(Mutex::new(conn))
}
