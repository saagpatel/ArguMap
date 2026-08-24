use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{ArgEdge, ArgNode};

pub const CONTRACT_SCHEMA_VERSION: &str = "evidence-centered.research-package.v1";
pub const CONTRACT_SCHEMA_SHA256: &str =
    "sha256:ab1702392cdd3c3b0d465f52de5114d5f4aad8e1e47730c10fca53fc7622360c";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AdapterLoss {
    pub path: String,
    pub reason: String,
    pub retained_in_canonical_package: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchProjection {
    pub package_id: String,
    pub revision_id: String,
    pub schema_digest: String,
    pub canonical_package: Value,
    pub nodes: Vec<ArgNode>,
    pub edges: Vec<ArgEdge>,
    pub losses: Vec<AdapterLoss>,
}

pub fn import_research_package(raw: &str, map_id: &str) -> Result<ResearchProjection, String> {
    let package: Value = serde_json::from_str(raw).map_err(|error| error.to_string())?;
    validate_package(&package)?;
    let package_id = text(&package, "package_id")?.to_string();
    let revision_id = text(&package, "revision_id")?.to_string();
    let sources = index(array(&package, "sources")?, "source_id")?;
    let evidence = index(array(&package, "evidence")?, "evidence_id")?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut losses = Vec::new();

    for (position, claim) in array(&package, "claims")?.iter().enumerate() {
        let claim_id = text(claim, "claim_id")?;
        nodes.push(ArgNode {
            id: format!("research-claim:{claim_id}"),
            map_id: map_id.into(),
            node_type: "claim".into(),
            content: text(claim, "statement")?.into(),
            source: None,
            x: 360.0,
            y: position as f64 * 140.0,
            width: 280.0,
            height: 100.0,
            strength: None,
        });
        losses.push(AdapterLoss {
            path: format!("claims/{claim_id}"),
            reason: "ArguMap nodes do not natively encode claim_type or declared_epistemic_status"
                .into(),
            retained_in_canonical_package: true,
        });
    }

    for (position, item) in evidence.values().enumerate() {
        let evidence_id = text(item, "evidence_id")?;
        let source_ref = strings(array(item, "source_refs")?)?
            .into_iter()
            .next()
            .ok_or_else(|| format!("evidence {evidence_id} has no source"))?;
        let source = sources[source_ref.as_str()];
        nodes.push(ArgNode {
            id: format!("research-evidence:{evidence_id}"),
            map_id: map_id.into(),
            node_type: "evidence".into(),
            content: format!(
                "{} [{} / {}]",
                text(item, "evidence_type")?,
                text(item, "status")?,
                text(item, "freshness")?
            ),
            source: Some(text(source, "locator")?.into()),
            x: 0.0,
            y: position as f64 * 120.0,
            width: 280.0,
            height: 100.0,
            strength: None,
        });
        losses.push(AdapterLoss {
            path: format!("evidence/{evidence_id}"),
            reason: "ArguMap evidence nodes render a compact view; method, version, binding, and exclusions remain in the retained canonical package".into(),
            retained_in_canonical_package: true,
        });
    }

    for claim in array(&package, "claims")? {
        let claim_id = text(claim, "claim_id")?;
        for (index, link) in array(claim, "evidence_links")?.iter().enumerate() {
            let evidence_id = text(link, "evidence_ref")?;
            let relationship = text(link, "relationship")?;
            let edge_type = match relationship {
                "supports" => "supports",
                "weakens" => "qualifies",
                "contradicts" => "rebuts",
                _ => return Err("unknown claim-evidence relationship".into()),
            };
            edges.push(ArgEdge {
                id: format!("research-edge:{claim_id}:{evidence_id}:{index}"),
                map_id: map_id.into(),
                source_node_id: format!("research-evidence:{evidence_id}"),
                target_node_id: format!("research-claim:{claim_id}"),
                edge_type: edge_type.into(),
                label: Some(relationship.into()),
            });
        }
    }
    for method in array(&package, "methods")? {
        losses.push(AdapterLoss {
            path: format!("methods/{}", text(method, "method_id")?),
            reason: "research method semantics have no native ArguMap node type".into(),
            retained_in_canonical_package: true,
        });
    }
    for conclusion in array(&package, "conclusions")? {
        losses.push(AdapterLoss {
            path: format!("conclusions/{}", text(conclusion, "conclusion_id")?),
            reason: "authored research conclusions and their qualification state have no native ArguMap node type".into(),
            retained_in_canonical_package: true,
        });
    }

    Ok(ResearchProjection {
        package_id,
        revision_id,
        schema_digest: CONTRACT_SCHEMA_SHA256.into(),
        canonical_package: package,
        nodes,
        edges,
        losses,
    })
}

pub fn export_research_package(projection: &ResearchProjection) -> Result<String, String> {
    serde_json::to_string(&canonicalize(&projection.canonical_package))
        .map_err(|error| error.to_string())
}

fn validate_package(package: &Value) -> Result<(), String> {
    if text(package, "schema_version")? != CONTRACT_SCHEMA_VERSION {
        return Err("unsupported evidence-centered research schema".into());
    }
    if !matches!(text(package, "privacy_tier")?, "P0" | "P1") {
        return Err("research package privacy tier must be P0 or P1".into());
    }
    if text(package, "privacy_tier")? == "P1"
        && package.get("reviewed").and_then(Value::as_bool) != Some(true)
    {
        return Err("P1 research packages require reviewed=true".into());
    }
    let sources = index(array(package, "sources")?, "source_id")?;
    let methods = index(array(package, "methods")?, "method_id")?;
    let evidence = index(array(package, "evidence")?, "evidence_id")?;
    let claims = index(array(package, "claims")?, "claim_id")?;
    if sources.is_empty() || methods.is_empty() || claims.is_empty() {
        return Err("sources, methods, and claims must be non-empty".into());
    }
    for item in evidence.values() {
        if !methods.contains_key(text(item, "method_ref")?) {
            return Err(format!(
                "evidence {} has unknown method",
                text(item, "evidence_id")?
            ));
        }
        for source_ref in strings(array(item, "source_refs")?)? {
            if !sources.contains_key(source_ref.as_str()) {
                return Err(format!(
                    "evidence {} has unknown source",
                    text(item, "evidence_id")?
                ));
            }
        }
        if text(item, "status")? == "available"
            && item.get("result_binding").is_none_or(Value::is_null)
        {
            return Err(format!(
                "available evidence {} lacks result binding",
                text(item, "evidence_id")?
            ));
        }
    }
    for claim in claims.values() {
        for link in array(claim, "evidence_links")? {
            if !evidence.contains_key(text(link, "evidence_ref")?) {
                return Err(format!(
                    "claim {} has unknown evidence",
                    text(claim, "claim_id")?
                ));
            }
        }
    }
    Ok(())
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}"))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing array field {key}"))
}

fn strings(values: &[Value]) -> Result<Vec<String>, String> {
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "expected string array".into())
        })
        .collect()
}

fn index<'a>(values: &'a [Value], id_field: &str) -> Result<BTreeMap<String, &'a Value>, String> {
    let mut result = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for value in values {
        let id = text(value, id_field)?.to_string();
        if !seen.insert(id.clone()) {
            return Err(format!("duplicate {id_field}"));
        }
        result.insert(id, value);
    }
    Ok(result)
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        Value::Object(values) => {
            let ordered: BTreeMap<_, _> = values
                .iter()
                .map(|(key, value)| (key.clone(), canonicalize(value)))
                .collect();
            serde_json::to_value(ordered).expect("BTreeMap serialization is infallible")
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../fixtures/evidence-centered-research/qualified-package-v2.json");

    #[test]
    fn shared_package_round_trips_without_loss() {
        let projection = import_research_package(FIXTURE, "map-ecrp").expect("import fixture");
        let original: Value = serde_json::from_str(FIXTURE).expect("parse fixture");
        let exported: Value =
            serde_json::from_str(&export_research_package(&projection).expect("export fixture"))
                .expect("parse export");
        assert_eq!(canonicalize(&original), canonicalize(&exported));
        assert!(projection
            .losses
            .iter()
            .all(|loss| loss.retained_in_canonical_package));
    }

    #[test]
    fn contradictions_and_projection_loss_are_visible() {
        let projection = import_research_package(FIXTURE, "map-ecrp").expect("import fixture");
        assert!(projection.edges.iter().any(|edge| {
            edge.edge_type == "rebuts" && edge.target_node_id == "research-claim:claim-contested"
        }));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "claims/claim-contested"));
        assert!(projection
            .nodes
            .iter()
            .any(|node| node.id == "research-evidence:evidence-retracted"));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "conclusions/conclusion-unsupported"));
    }
}
