use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArgNode {
    pub id: String,
    pub map_id: String,
    pub node_type: String,
    pub content: String,
    pub source: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub strength: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArgEdge {
    pub id: String,
    pub map_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodePayload {
    pub id: String,
    pub node_type: String,
    pub content: String,
    pub source: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub strength: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgePayload {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: String,
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // ─── NodePayload serde round-trips ───────────────────────────────────────

    fn make_node_payload(
        node_type: &str,
        strength: Option<i32>,
        source: Option<&str>,
    ) -> NodePayload {
        NodePayload {
            id: "node-1".to_string(),
            node_type: node_type.to_string(),
            content: "Test content".to_string(),
            source: source.map(str::to_string),
            x: 10.5,
            y: 20.0,
            width: 220.0,
            height: 80.0,
            strength,
        }
    }

    fn make_edge_payload(edge_type: &str, label: Option<&str>) -> EdgePayload {
        EdgePayload {
            id: "edge-1".to_string(),
            source_node_id: "node-a".to_string(),
            target_node_id: "node-b".to_string(),
            edge_type: edge_type.to_string(),
            label: label.map(str::to_string),
        }
    }

    #[test]
    fn node_payload_claim_roundtrip() {
        let original = make_node_payload("claim", Some(5), None);
        let json = serde_json::to_string(&original).expect("serialize");
        // NodePayload is Deserialize-only, but we can deserialize from the JSON we produced
        let restored: NodePayload = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.node_type, "claim");
        assert_eq!(restored.content, original.content);
        assert_eq!(restored.source, None);
        assert_eq!(restored.x, 10.5);
        assert_eq!(restored.y, 20.0);
        assert_eq!(restored.width, 220.0);
        assert_eq!(restored.height, 80.0);
        assert_eq!(restored.strength, Some(5));
    }

    #[test]
    fn node_payload_evidence_with_source_roundtrip() {
        let original = make_node_payload("evidence", None, Some("https://example.com"));
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: NodePayload = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.node_type, "evidence");
        assert_eq!(restored.source.as_deref(), Some("https://example.com"));
        assert_eq!(restored.strength, None);
    }

    #[test]
    fn node_payload_rebuttal_roundtrip() {
        let original = make_node_payload("rebuttal", Some(3), None);
        let json = serde_json::to_string(&original).unwrap();
        let restored: NodePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.node_type, "rebuttal");
        assert_eq!(restored.strength, Some(3));
    }

    #[test]
    fn node_payload_counter_rebuttal_roundtrip() {
        let original = make_node_payload("counter_rebuttal", None, None);
        let json = serde_json::to_string(&original).unwrap();
        let restored: NodePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.node_type, "counter_rebuttal");
    }

    #[test]
    fn all_node_types_survive_roundtrip() {
        for node_type in &["claim", "evidence", "rebuttal", "counter_rebuttal"] {
            let payload = make_node_payload(node_type, None, None);
            let json = serde_json::to_string(&payload).unwrap();
            let restored: NodePayload = serde_json::from_str(&json).unwrap();
            assert_eq!(
                &restored.node_type, node_type,
                "node_type mismatch for {node_type}"
            );
        }
    }

    // ─── EdgePayload serde round-trips ───────────────────────────────────────

    #[test]
    fn edge_payload_supports_roundtrip() {
        let original = make_edge_payload("supports", Some("key support"));
        let json = serde_json::to_string(&original).unwrap();
        let restored: EdgePayload = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, "edge-1");
        assert_eq!(restored.source_node_id, "node-a");
        assert_eq!(restored.target_node_id, "node-b");
        assert_eq!(restored.edge_type, "supports");
        assert_eq!(restored.label.as_deref(), Some("key support"));
    }

    #[test]
    fn edge_payload_rebuts_no_label_roundtrip() {
        let original = make_edge_payload("rebuts", None);
        let json = serde_json::to_string(&original).unwrap();
        let restored: EdgePayload = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.edge_type, "rebuts");
        assert_eq!(restored.label, None);
    }

    #[test]
    fn all_edge_types_survive_roundtrip() {
        for edge_type in &["supports", "rebuts", "qualifies", "depends_on"] {
            let payload = make_edge_payload(edge_type, None);
            let json = serde_json::to_string(&payload).unwrap();
            let restored: EdgePayload = serde_json::from_str(&json).unwrap();
            assert_eq!(
                &restored.edge_type, edge_type,
                "edge_type mismatch for {edge_type}"
            );
        }
    }

    // ─── ArgNode / ArgEdge / Map serde round-trips ───────────────────────────

    #[test]
    fn arg_node_serialize_deserialize() {
        let node = ArgNode {
            id: "n1".to_string(),
            map_id: "m1".to_string(),
            node_type: "claim".to_string(),
            content: "Some claim".to_string(),
            source: Some("ref".to_string()),
            x: 5.0,
            y: 10.0,
            width: 200.0,
            height: 75.0,
            strength: Some(2),
        };
        let json = serde_json::to_string(&node).unwrap();
        let restored: ArgNode = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, node.id);
        assert_eq!(restored.map_id, node.map_id);
        assert_eq!(restored.node_type, node.node_type);
        assert_eq!(restored.content, node.content);
        assert_eq!(restored.source, node.source);
        assert_eq!(restored.strength, node.strength);
    }

    #[test]
    fn arg_edge_serialize_deserialize() {
        let edge = ArgEdge {
            id: "e1".to_string(),
            map_id: "m1".to_string(),
            source_node_id: "n1".to_string(),
            target_node_id: "n2".to_string(),
            edge_type: "qualifies".to_string(),
            label: None,
        };
        let json = serde_json::to_string(&edge).unwrap();
        let restored: ArgEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, edge.id);
        assert_eq!(restored.edge_type, "qualifies");
        assert_eq!(restored.label, None);
    }

    #[test]
    fn map_serialize_deserialize() {
        let map = Map {
            id: "m1".to_string(),
            title: "My Map".to_string(),
            description: Some("A description".to_string()),
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: "2024-06-01 12:00:00".to_string(),
        };
        let json = serde_json::to_string(&map).unwrap();
        let restored: Map = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, map.id);
        assert_eq!(restored.title, map.title);
        assert_eq!(restored.description, map.description);
        assert_eq!(restored.updated_at, map.updated_at);
    }

    #[test]
    fn map_without_description_roundtrip() {
        let map = Map {
            id: "m2".to_string(),
            title: "No Desc".to_string(),
            description: None,
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: "2024-01-01 00:00:00".to_string(),
        };
        let json = serde_json::to_string(&map).unwrap();
        let restored: Map = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.description, None);
    }
}
