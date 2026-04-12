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

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    /// Run both migrations on a fresh in-memory connection, matching init_db() logic.
    fn apply_migrations(conn: &Connection) {
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn.execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        // On a fresh in-memory db the strength column never exists, so always run migration 2.
        conn.execute_batch(include_str!("../migrations/002_add_strength.sql"))
            .unwrap();
    }

    fn table_exists(conn: &Connection, table: &str) -> bool {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count > 0
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
        // PRAGMA table_info returns one row per column
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |r| Ok(r.get::<_, String>(1)?))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        cols.iter().any(|c| c == column)
    }

    #[test]
    fn init_creates_all_three_tables() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn);

        assert!(table_exists(&conn, "maps"), "maps table must exist");
        assert!(table_exists(&conn, "nodes"), "nodes table must exist");
        assert!(table_exists(&conn, "edges"), "edges table must exist");
    }

    #[test]
    fn init_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn);
        // Running migration 1 again must not error (CREATE TABLE IF NOT EXISTS)
        let result = conn.execute_batch(include_str!("../migrations/001_initial.sql"));
        assert!(result.is_ok(), "second run of migration 1 must succeed");
    }

    #[test]
    fn strength_column_exists_after_migration_2() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn);
        assert!(
            column_exists(&conn, "nodes", "strength"),
            "nodes.strength must exist after migration 2"
        );
    }

    #[test]
    fn strength_migration_is_idempotent_via_guard() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn);

        // Simulate the guard in init_db: only apply migration 2 when column is absent
        let has_strength = conn.prepare("SELECT strength FROM nodes LIMIT 0").is_ok();
        // Column already exists — applying migration 2 again would fail with "duplicate column"
        // The guard means we skip it. Verify the guard itself works correctly.
        assert!(
            has_strength,
            "guard must detect existing strength column and skip migration 2"
        );
    }

    #[test]
    fn foreign_key_cascade_delete_works() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn);

        // Insert a map, a node, then delete the map — node should cascade-delete
        conn.execute(
            "INSERT INTO maps (id, title, created_at, updated_at) VALUES ('m1', 'T', '2024-01-01', '2024-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, map_id, node_type, content, x, y, width, height) VALUES ('n1', 'm1', 'claim', 'C', 0, 0, 220, 80)",
            [],
        )
        .unwrap();

        conn.execute("DELETE FROM maps WHERE id = 'm1'", [])
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes WHERE map_id = 'm1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "nodes must cascade-delete with their map");
    }
}
