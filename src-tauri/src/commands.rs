use crate::db::DbPool;
use crate::models::{ArgEdge, ArgNode, EdgePayload, Map, NodePayload};
use crate::research_adapter::{
    export_research_package, import_research_package, ResearchProjection,
};
use rusqlite::{params, Connection};
use tauri::State;

#[tauri::command]
pub fn get_maps(db: State<'_, DbPool>) -> Result<Vec<Map>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, description, created_at, updated_at FROM maps ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let maps = stmt
        .query_map([], |row| {
            Ok(Map {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(maps)
}

#[tauri::command]
pub fn create_map(
    db: State<'_, DbPool>,
    title: String,
    description: Option<String>,
) -> Result<Map, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO maps (id, title, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, title, description, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Map {
        id,
        title,
        description,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_map(db: State<'_, DbPool>, map_id: String) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM maps WHERE id = ?1", params![map_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn rename_map(db: State<'_, DbPool>, map_id: String, title: String) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE maps SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, map_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_map(db: State<'_, DbPool>, map_id: String) -> Result<serde_json::Value, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    // Load nodes
    let mut node_stmt = conn
        .prepare("SELECT id, map_id, node_type, content, source, x, y, width, height, strength FROM nodes WHERE map_id = ?1")
        .map_err(|e| e.to_string())?;

    let nodes: Vec<ArgNode> = node_stmt
        .query_map(params![map_id], |row| {
            Ok(ArgNode {
                id: row.get(0)?,
                map_id: row.get(1)?,
                node_type: row.get(2)?,
                content: row.get(3)?,
                source: row.get(4)?,
                x: row.get(5)?,
                y: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                strength: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Load edges
    let mut edge_stmt = conn
        .prepare("SELECT id, map_id, source_node_id, target_node_id, edge_type, label FROM edges WHERE map_id = ?1")
        .map_err(|e| e.to_string())?;

    let edges: Vec<ArgEdge> = edge_stmt
        .query_map(params![map_id], |row| {
            Ok(ArgEdge {
                id: row.get(0)?,
                map_id: row.get(1)?,
                source_node_id: row.get(2)?,
                target_node_id: row.get(3)?,
                edge_type: row.get(4)?,
                label: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "nodes": nodes, "edges": edges }))
}

#[tauri::command]
pub fn save_map_state(
    db: State<'_, DbPool>,
    map_id: String,
    nodes: Vec<NodePayload>,
    edges: Vec<EdgePayload>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Full upsert in a single transaction
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| e.to_string())?;

    // Delete existing nodes and edges for this map
    conn.execute("DELETE FROM edges WHERE map_id = ?1", params![map_id])
        .map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK;");
            e.to_string()
        })?;
    conn.execute("DELETE FROM nodes WHERE map_id = ?1", params![map_id])
        .map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK;");
            e.to_string()
        })?;

    // Insert all nodes
    for node in &nodes {
        conn.execute(
            "INSERT INTO nodes (id, map_id, node_type, content, source, x, y, width, height, strength) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![node.id, map_id, node.node_type, node.content, node.source, node.x, node.y, node.width, node.height, node.strength],
        )
        .map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK;");
            e.to_string()
        })?;
    }

    // Insert all edges
    for edge in &edges {
        conn.execute(
            "INSERT INTO edges (id, map_id, source_node_id, target_node_id, edge_type, label) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![edge.id, map_id, edge.source_node_id, edge.target_node_id, edge.edge_type, edge.label],
        )
        .map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK;");
            e.to_string()
        })?;
    }

    // Update map's updated_at
    conn.execute(
        "UPDATE maps SET updated_at = ?1 WHERE id = ?2",
        params![now, map_id],
    )
    .map_err(|e| {
        let _ = conn.execute_batch("ROLLBACK;");
        e.to_string()
    })?;

    conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn inspect_research_package(raw: String, map_id: String) -> Result<ResearchProjection, String> {
    import_research_package(&raw, &map_id)
}

#[tauri::command]
pub fn export_canonical_research_package(raw: String, map_id: String) -> Result<String, String> {
    let projection = import_research_package(&raw, &map_id)?;
    export_research_package(&projection)
}

#[tauri::command]
pub fn import_research_package_into_map(
    db: State<'_, DbPool>,
    raw: String,
    map_id: String,
) -> Result<ResearchProjection, String> {
    let projection = import_research_package(&raw, &map_id)?;
    let conn = db.lock().map_err(|error| error.to_string())?;
    persist_research_projection(&conn, &map_id, &projection)?;
    Ok(projection)
}

#[tauri::command]
pub fn load_persisted_research_package(
    db: State<'_, DbPool>,
    map_id: String,
) -> Result<Option<ResearchProjection>, String> {
    let conn = db.lock().map_err(|error| error.to_string())?;
    load_persisted_research_projection(&conn, &map_id)
}

#[tauri::command]
pub fn export_persisted_canonical_research_package(
    db: State<'_, DbPool>,
    map_id: String,
) -> Result<String, String> {
    let conn = db.lock().map_err(|error| error.to_string())?;
    conn.query_row(
        "SELECT canonical_package FROM research_packages WHERE map_id = ?1",
        params![map_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn persist_research_projection(
    conn: &Connection,
    map_id: &str,
    projection: &ResearchProjection,
) -> Result<(), String> {
    let map_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM maps WHERE id = ?1",
            params![map_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if map_exists != 1 {
        return Err("research package target map does not exist".into());
    }

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|error| error.to_string())?;
    let result = (|| -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM edges WHERE map_id = ?1", params![map_id])?;
        conn.execute("DELETE FROM nodes WHERE map_id = ?1", params![map_id])?;
        for node in &projection.nodes {
            conn.execute(
                "INSERT INTO nodes (id, map_id, node_type, content, source, x, y, width, height, strength) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![node.id, map_id, node.node_type, node.content, node.source, node.x, node.y, node.width, node.height, node.strength],
            )?;
        }
        for edge in &projection.edges {
            conn.execute(
                "INSERT INTO edges (id, map_id, source_node_id, target_node_id, edge_type, label) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![edge.id, map_id, edge.source_node_id, edge.target_node_id, edge.edge_type, edge.label],
            )?;
        }
        let canonical_package = serde_json::to_string(&projection.canonical_package)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let adapter_losses = serde_json::to_string(&projection.losses)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        conn.execute(
            "INSERT INTO research_packages
                (map_id, schema_version, package_id, revision_id, canonical_package, adapter_losses, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
             ON CONFLICT(map_id) DO UPDATE SET
                schema_version = excluded.schema_version,
                package_id = excluded.package_id,
                revision_id = excluded.revision_id,
                canonical_package = excluded.canonical_package,
                adapter_losses = excluded.adapter_losses,
                updated_at = CURRENT_TIMESTAMP",
            params![
                map_id,
                projection.schema_version,
                projection.package_id,
                projection.revision_id,
                canonical_package,
                adapter_losses,
            ],
        )?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE maps SET updated_at = ?1 WHERE id = ?2",
            params![now, map_id],
        )?;
        Ok(())
    })();

    match result {
        Ok(()) => conn
            .execute_batch("COMMIT;")
            .map_err(|error| error.to_string()),
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error.to_string())
        }
    }
}

fn load_persisted_research_projection(
    conn: &Connection,
    map_id: &str,
) -> Result<Option<ResearchProjection>, String> {
    let stored = conn.query_row(
        "SELECT canonical_package, adapter_losses FROM research_packages WHERE map_id = ?1",
        params![map_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );
    let (canonical_package, adapter_losses) = match stored {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut projection = import_research_package(&canonical_package, map_id)?;
    projection.losses = serde_json::from_str(&adapter_losses)
        .map_err(|error| format!("persisted research losses are invalid: {error}"))?;
    Ok(Some(projection))
}

#[cfg(test)]
mod tests {
    use super::{
        export_canonical_research_package, inspect_research_package,
        load_persisted_research_projection, persist_research_projection,
    };
    use rusqlite::{params, Connection};

    /// Run both migrations on an in-memory connection, mirroring what init_db() does.
    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn.execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        // migration 2 is always needed on a fresh in-memory db
        conn.execute_batch(include_str!("../migrations/002_add_strength.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/003_research_packages.sql"))
            .unwrap();
        conn
    }

    fn insert_map(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO maps (id, title, created_at, updated_at) VALUES (?1, 'Test Map', '2024-01-01 00:00:00', '2024-01-01 00:00:00')",
            params![id],
        )
        .unwrap();
    }

    fn insert_nodes(conn: &Connection, map_id: &str) -> Vec<String> {
        let node_ids = vec!["node-1".to_string(), "node-2".to_string()];
        for nid in &node_ids {
            conn.execute(
                "INSERT INTO nodes (id, map_id, node_type, content, source, x, y, width, height, strength) \
                 VALUES (?1, ?2, 'claim', 'Content', NULL, 10.0, 20.0, 220.0, 80.0, 3)",
                params![nid, map_id],
            )
            .unwrap();
        }
        node_ids
    }

    fn insert_edge(conn: &Connection, map_id: &str, src: &str, tgt: &str) {
        conn.execute(
            "INSERT INTO edges (id, map_id, source_node_id, target_node_id, edge_type) \
             VALUES ('edge-1', ?1, ?2, ?3, 'supports')",
            params![map_id, src, tgt],
        )
        .unwrap();
    }

    // ─── save_map_state logic ────────────────────────────────────────────────

    /// Replicates the save_map_state transaction logic for testing without Tauri State.
    fn save_map_state_inner(
        conn: &Connection,
        map_id: &str,
        nodes: &[(
            &str,
            &str,
            &str,
            Option<&str>,
            f64,
            f64,
            f64,
            f64,
            Option<i32>,
        )],
        edges: &[(&str, &str, &str, &str, Option<&str>)],
    ) -> Result<(), rusqlite::Error> {
        let now = "2024-06-01 12:00:00";
        conn.execute_batch("BEGIN TRANSACTION;")?;

        conn.execute("DELETE FROM edges WHERE map_id = ?1", params![map_id])?;
        conn.execute("DELETE FROM nodes WHERE map_id = ?1", params![map_id])?;

        for &(id, node_type, content, source, x, y, w, h, strength) in nodes {
            conn.execute(
                "INSERT INTO nodes (id, map_id, node_type, content, source, x, y, width, height, strength) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![id, map_id, node_type, content, source, x, y, w, h, strength],
            )?;
        }

        for &(id, src, tgt, edge_type, label) in edges {
            conn.execute(
                "INSERT INTO edges (id, map_id, source_node_id, target_node_id, edge_type, label) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, map_id, src, tgt, edge_type, label],
            )?;
        }

        conn.execute(
            "UPDATE maps SET updated_at = ?1 WHERE id = ?2",
            params![now, map_id],
        )?;

        conn.execute_batch("COMMIT;")?;
        Ok(())
    }

    #[test]
    fn save_map_with_nodes_and_edges_roundtrip() {
        let conn = setup_db();
        let map_id = "map-abc";
        insert_map(&conn, map_id);

        let nodes = vec![
            (
                "n1",
                "claim",
                "Main claim",
                None,
                0.0,
                0.0,
                220.0,
                80.0,
                Some(5),
            ),
            (
                "n2",
                "evidence",
                "Supporting evidence",
                Some("https://example.com"),
                200.0,
                100.0,
                220.0,
                80.0,
                None,
            ),
        ];
        let edges = vec![("e1", "n1", "n2", "supports", Some("strong support"))];

        save_map_state_inner(&conn, map_id, &nodes, &edges).unwrap();

        let node_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE map_id = ?1",
                params![map_id],
                |r| r.get(0),
            )
            .unwrap();
        let edge_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE map_id = ?1",
                params![map_id],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(node_count, 2);
        assert_eq!(edge_count, 1);

        // Verify content round-trips correctly
        let content: String = conn
            .query_row("SELECT content FROM nodes WHERE id = 'n1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(content, "Main claim");

        let source: Option<String> = conn
            .query_row("SELECT source FROM nodes WHERE id = 'n2'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(source.as_deref(), Some("https://example.com"));

        let label: Option<String> = conn
            .query_row("SELECT label FROM edges WHERE id = 'e1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(label.as_deref(), Some("strong support"));
    }

    #[test]
    fn save_empty_map_persists_map_row() {
        let conn = setup_db();
        let map_id = "map-empty";
        insert_map(&conn, map_id);

        save_map_state_inner(&conn, map_id, &[], &[]).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM maps WHERE id = ?1",
                params![map_id],
                |r| r.get(0),
            )
            .unwrap();
        let node_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE map_id = ?1",
                params![map_id],
                |r| r.get(0),
            )
            .unwrap();
        let edge_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE map_id = ?1",
                params![map_id],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(count, 1, "map row must survive empty save");
        assert_eq!(node_count, 0);
        assert_eq!(edge_count, 0);
    }

    #[test]
    fn save_replaces_existing_nodes_and_edges() {
        let conn = setup_db();
        let map_id = "map-replace";
        insert_map(&conn, map_id);
        let node_ids = insert_nodes(&conn, map_id);
        insert_edge(&conn, map_id, &node_ids[0], &node_ids[1]);

        // Now save with entirely different nodes — old ones must be gone
        let new_nodes = vec![(
            "n-new",
            "rebuttal",
            "New rebuttal",
            None,
            5.0,
            5.0,
            220.0,
            80.0,
            None,
        )];
        save_map_state_inner(&conn, map_id, &new_nodes, &[]).unwrap();

        let ids: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT id FROM nodes WHERE map_id = ?1")
                .unwrap();
            stmt.query_map(params![map_id], |r| r.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(ids, vec!["n-new"]);

        let edge_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE map_id = ?1",
                params![map_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(edge_count, 0, "old edges must be deleted");
    }

    #[test]
    fn save_with_nonexistent_map_id_succeeds_silently() {
        // save_map_state does not validate map existence — UPDATE with no match is a no-op
        let conn = setup_db();
        let result = save_map_state_inner(&conn, "ghost-map", &[], &[]);
        assert!(result.is_ok(), "no error expected for missing map_id");

        // No map row should have been created
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM maps WHERE id = 'ghost-map'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn init_save_load_roundtrip() {
        let conn = setup_db();
        let map_id = "map-roundtrip";
        insert_map(&conn, map_id);

        let nodes = vec![
            (
                "rt-n1",
                "claim",
                "Claim text",
                None,
                0.0,
                0.0,
                220.0,
                80.0,
                Some(4),
            ),
            (
                "rt-n2",
                "evidence",
                "Evidence text",
                Some("source-url"),
                300.0,
                50.0,
                220.0,
                80.0,
                None,
            ),
        ];
        let edges = vec![("rt-e1", "rt-n1", "rt-n2", "supports", None)];
        save_map_state_inner(&conn, map_id, &nodes, &edges).unwrap();

        // Load nodes
        let loaded_nodes: Vec<(String, String, f64, Option<i32>)> = {
            let mut stmt = conn
                .prepare("SELECT id, content, x, strength FROM nodes WHERE map_id = ?1 ORDER BY id")
                .unwrap();
            stmt.query_map(params![map_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
        };

        assert_eq!(loaded_nodes.len(), 2);
        assert_eq!(loaded_nodes[0].0, "rt-n1");
        assert_eq!(loaded_nodes[0].1, "Claim text");
        assert_eq!(loaded_nodes[0].2, 0.0_f64);
        assert_eq!(loaded_nodes[0].3, Some(4));
        assert_eq!(loaded_nodes[1].3, None);

        // Load edges
        let loaded_edges: Vec<(String, String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, source_node_id, target_node_id FROM edges WHERE map_id = ?1")
                .unwrap();
            stmt.query_map(params![map_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };

        assert_eq!(loaded_edges.len(), 1);
        assert_eq!(loaded_edges[0].0, "rt-e1");
        assert_eq!(loaded_edges[0].1, "rt-n1");
        assert_eq!(loaded_edges[0].2, "rt-n2");
    }

    #[test]
    fn research_package_import_persists_projection_and_preserves_losses() {
        const FIXTURE: &str =
            include_str!("../../fixtures/evidence-centered-research/qualified-package-v3.json");
        let conn = setup_db();
        let map_id = "map-research";
        insert_map(&conn, map_id);
        let projection =
            inspect_research_package(FIXTURE.into(), map_id.into()).expect("inspect package");
        persist_research_projection(&conn, map_id, &projection).expect("persist projection");

        let node_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE map_id = ?1",
                params![map_id],
                |row| row.get(0),
            )
            .unwrap();
        let edge_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE map_id = ?1",
                params![map_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(node_count as usize, projection.nodes.len());
        assert_eq!(edge_count as usize, projection.edges.len());
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path.contains("lifecycle_attestation")));

        let exported = export_canonical_research_package(FIXTURE.into(), map_id.into())
            .expect("export canonical package");
        let original: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
        let round_trip: serde_json::Value = serde_json::from_str(&exported).unwrap();
        assert_eq!(original, round_trip);

        let reloaded = load_persisted_research_projection(&conn, map_id)
            .expect("load persisted exchange")
            .expect("persisted exchange exists");
        assert_eq!(reloaded.canonical_package, original);
        assert_eq!(reloaded.losses, projection.losses);
    }

    #[test]
    fn identical_package_ids_persist_in_distinct_maps_without_global_id_collision() {
        const FIXTURE: &str =
            include_str!("../../fixtures/evidence-centered-research/qualified-package-v3.json");
        let conn = setup_db();
        for map_id in ["map-first", "map-second"] {
            insert_map(&conn, map_id);
            let projection =
                inspect_research_package(FIXTURE.into(), map_id.into()).expect("inspect fixture");
            persist_research_projection(&conn, map_id, &projection).expect("persist projection");
        }
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM research_packages", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 2);
    }
}

#[tauri::command]
pub fn export_map_json(db: State<'_, DbPool>, map_id: String) -> Result<String, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    // Load map metadata
    let map = conn
        .query_row(
            "SELECT id, title, description, created_at, updated_at FROM maps WHERE id = ?1",
            params![map_id],
            |row| {
                Ok(Map {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    // Load nodes
    let mut node_stmt = conn
        .prepare("SELECT id, map_id, node_type, content, source, x, y, width, height, strength FROM nodes WHERE map_id = ?1")
        .map_err(|e| e.to_string())?;

    let nodes: Vec<ArgNode> = node_stmt
        .query_map(params![map_id], |row| {
            Ok(ArgNode {
                id: row.get(0)?,
                map_id: row.get(1)?,
                node_type: row.get(2)?,
                content: row.get(3)?,
                source: row.get(4)?,
                x: row.get(5)?,
                y: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                strength: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Load edges
    let mut edge_stmt = conn
        .prepare("SELECT id, map_id, source_node_id, target_node_id, edge_type, label FROM edges WHERE map_id = ?1")
        .map_err(|e| e.to_string())?;

    let edges: Vec<ArgEdge> = edge_stmt
        .query_map(params![map_id], |row| {
            Ok(ArgEdge {
                id: row.get(0)?,
                map_id: row.get(1)?,
                source_node_id: row.get(2)?,
                target_node_id: row.get(3)?,
                edge_type: row.get(4)?,
                label: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let export = serde_json::json!({
        "map": map,
        "nodes": nodes,
        "edges": edges,
    });

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}
