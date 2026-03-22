use crate::db::DbPool;
use crate::models::{ArgEdge, ArgNode, EdgePayload, Map, NodePayload};
use rusqlite::params;
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
        .prepare("SELECT id, map_id, node_type, content, source, x, y, width, height FROM nodes WHERE map_id = ?1")
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
            "INSERT INTO nodes (id, map_id, node_type, content, source, x, y, width, height) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![node.id, map_id, node.node_type, node.content, node.source, node.x, node.y, node.width, node.height],
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
        .prepare("SELECT id, map_id, node_type, content, source, x, y, width, height FROM nodes WHERE map_id = ?1")
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
