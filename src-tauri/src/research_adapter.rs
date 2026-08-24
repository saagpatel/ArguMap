use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{ArgEdge, ArgNode};

pub const CONTRACT_SCHEMA_VERSION_V1: &str = "evidence-centered.research-package.v1";
pub const CONTRACT_SCHEMA_SHA256_V1: &str =
    "sha256:ab1702392cdd3c3b0d465f52de5114d5f4aad8e1e47730c10fca53fc7622360c";
pub const CONTRACT_SCHEMA_VERSION_V2: &str = "evidence-centered.research-package.v2";
pub const CONTRACT_SCHEMA_SHA256_V2: &str =
    "sha256:4cff2030f2ccfb64937d8db5453f16510b30bc1db48a882d161a8b6944ae3ceb";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AdapterLoss {
    pub path: String,
    pub reason: String,
    pub retained_in_canonical_package: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchProjection {
    pub schema_version: String,
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
    let schema_version = text(&package, "schema_version")?.to_string();
    let namespace = format!("{map_id}:{package_id}:{revision_id}");
    let sources = index(array(&package, "sources")?, "source_id")?;
    let evidence = index(optional_array(&package, "evidence")?, "evidence_id")?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut losses = Vec::new();

    for (position, claim) in array(&package, "claims")?.iter().enumerate() {
        let claim_id = text(claim, "claim_id")?;
        nodes.push(ArgNode {
            id: projected_id(&namespace, "claim", claim_id),
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
            id: projected_id(&namespace, "evidence", evidence_id),
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
        for (index, link) in optional_array(claim, "evidence_links")?.iter().enumerate() {
            let evidence_id = text(link, "evidence_ref")?;
            let relationship = text(link, "relationship")?;
            let edge_type = match relationship {
                "supports" => "supports",
                "weakens" => "qualifies",
                "contradicts" => "rebuts",
                _ => return Err("unknown claim-evidence relationship".into()),
            };
            edges.push(ArgEdge {
                id: projected_id(
                    &namespace,
                    "edge",
                    &format!("{claim_id}:{evidence_id}:{index}"),
                ),
                map_id: map_id.into(),
                source_node_id: projected_id(&namespace, "evidence", evidence_id),
                target_node_id: projected_id(&namespace, "claim", claim_id),
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
        if schema_version == CONTRACT_SCHEMA_VERSION_V2 {
            losses.push(AdapterLoss {
                path: format!(
                    "methods/{}/population_binding",
                    text(method, "method_id")?
                ),
                reason: "estimand, population, sampling-frame, and missingness semantics have no native ArguMap node type".into(),
                retained_in_canonical_package: true,
            });
        }
    }
    for conclusion in optional_array(&package, "conclusions")? {
        losses.push(AdapterLoss {
            path: format!("conclusions/{}", text(conclusion, "conclusion_id")?),
            reason: "authored research conclusions and their qualification state have no native ArguMap node type".into(),
            retained_in_canonical_package: true,
        });
    }
    if schema_version == CONTRACT_SCHEMA_VERSION_V2 {
        for source in array(&package, "sources")? {
            losses.push(AdapterLoss {
                path: format!(
                    "sources/{}/lifecycle_attestation",
                    text(source, "source_id")?
                ),
                reason: "signed lifecycle and authority-trust semantics are retained but have no native ArguMap node type; ArguMap does not promote embedded trust declarations".into(),
                retained_in_canonical_package: true,
            });
        }
    }

    Ok(ResearchProjection {
        schema_version: schema_version.clone(),
        package_id,
        revision_id,
        schema_digest: match schema_version.as_str() {
            CONTRACT_SCHEMA_VERSION_V1 => CONTRACT_SCHEMA_SHA256_V1.into(),
            CONTRACT_SCHEMA_VERSION_V2 => CONTRACT_SCHEMA_SHA256_V2.into(),
            _ => return Err("unsupported evidence-centered research schema".into()),
        },
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
    let schema_version = text(package, "schema_version")?;
    if !matches!(
        schema_version,
        CONTRACT_SCHEMA_VERSION_V1 | CONTRACT_SCHEMA_VERSION_V2
    ) {
        return Err("unsupported evidence-centered research schema".into());
    }
    validate_declared_schema(package, schema_version)?;
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
    let evidence = index(optional_array(package, "evidence")?, "evidence_id")?;
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
        for link in optional_array(claim, "evidence_links")? {
            if !evidence.contains_key(text(link, "evidence_ref")?) {
                return Err(format!(
                    "claim {} has unknown evidence",
                    text(claim, "claim_id")?
                ));
            }
        }
    }
    if schema_version == CONTRACT_SCHEMA_VERSION_V2 {
        let authorities = index(array(package, "lifecycle_authorities")?, "authority_id")?;
        let attestations = index(array(package, "lifecycle_attestations")?, "attestation_id")?;
        let mut referenced = BTreeSet::new();
        for (source_id, source) in &sources {
            let attestation_ref = text(source, "lifecycle_attestation_ref")?;
            let attestation = attestations
                .get(attestation_ref)
                .ok_or_else(|| format!("source {source_id} has unknown lifecycle attestation"))?;
            if text(attestation, "source_ref")? != source_id
                || text(attestation, "asserted_state")? != text(source, "state")?
                || text(attestation, "asserted_freshness")? != text(source, "freshness")?
                || text(attestation, "version_id")? != text(source, "version_id")?
                || text(attestation, "content_digest")? != text(source, "content_digest")?
            {
                return Err(format!(
                    "attestation {attestation_ref} does not bind source state"
                ));
            }
            if !authorities.contains_key(text(attestation, "authority_ref")?) {
                return Err(format!(
                    "attestation {attestation_ref} has unknown authority"
                ));
            }
            referenced.insert(attestation_ref.to_string());
        }
        if referenced != attestations.keys().cloned().collect() {
            return Err("every lifecycle attestation must bind exactly one source".into());
        }
        for (method_id, method) in &methods {
            let population = method
                .get("population_binding")
                .ok_or_else(|| format!("method {method_id} lacks population binding"))?;
            let mut missing: BTreeSet<String> = [
                "estimand",
                "target_population",
                "analysis_population",
                "sampling_frame",
                "sampling_method",
            ]
            .into_iter()
            .filter(|field| population.get(*field).is_none_or(Value::is_null))
            .map(str::to_string)
            .collect();
            if text(population, "missingness_mechanism")? == "unknown" {
                missing.insert("missingness_mechanism".into());
            }
            let declared: BTreeSet<String> = strings(array(population, "unknown_fields")?)?
                .into_iter()
                .collect();
            if missing != declared {
                return Err(format!(
                    "method {method_id} population unknown_fields do not match missing fields"
                ));
            }
        }
    }
    Ok(())
}

fn validate_declared_schema(package: &Value, schema_version: &str) -> Result<(), String> {
    let raw_schema = match schema_version {
        CONTRACT_SCHEMA_VERSION_V1 => {
            include_str!("../../contracts/evidence-centered-research-package-v1.schema.json")
        }
        CONTRACT_SCHEMA_VERSION_V2 => {
            include_str!("../../contracts/evidence-centered-research-package-v2.schema.json")
        }
        _ => return Err("unsupported evidence-centered research schema".into()),
    };
    let schema: Value = serde_json::from_str(raw_schema)
        .map_err(|error| format!("bundled research schema is invalid: {error}"))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|error| format!("bundled research schema did not compile: {error}"))?;
    validator
        .validate(package)
        .map_err(|error| format!("research package violates {schema_version}: {error}"))
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

fn optional_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    match value.get(key) {
        None => Ok(&[]),
        Some(value) => value
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| format!("field {key} must be an array")),
    }
}

fn projected_id(namespace: &str, kind: &str, native_id: &str) -> String {
    format!("research:{namespace}:{kind}:{native_id}")
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
    const FIXTURE_V3: &str =
        include_str!("../../fixtures/evidence-centered-research/qualified-package-v3.json");
    const FIXTURE_V4: &str =
        include_str!("../../fixtures/evidence-centered-research/qualified-package-v4.json");

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
            edge.edge_type == "rebuts"
                && edge.target_node_id
                    == projected_id(
                        "map-ecrp:ecrp-adversarial-fixture:rev-2",
                        "claim",
                        "claim-contested",
                    )
        }));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "claims/claim-contested"));
        assert!(projection.nodes.iter().any(|node| {
            node.id
                == projected_id(
                    "map-ecrp:ecrp-adversarial-fixture:rev-2",
                    "evidence",
                    "evidence-retracted",
                )
        }));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "conclusions/conclusion-unsupported"));
    }

    #[test]
    fn v2_package_round_trips_with_explicit_lifecycle_and_population_loss() {
        let projection =
            import_research_package(FIXTURE_V3, "map-ecrp-v3").expect("import package v2 fixture");
        assert_eq!(projection.schema_version, CONTRACT_SCHEMA_VERSION_V2);
        assert_eq!(projection.schema_digest, CONTRACT_SCHEMA_SHA256_V2);
        let original: Value = serde_json::from_str(FIXTURE_V3).expect("parse fixture");
        let exported: Value = serde_json::from_str(
            &export_research_package(&projection).expect("export package v2 fixture"),
        )
        .expect("parse export");
        assert_eq!(canonicalize(&original), canonicalize(&exported));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "sources/source-unknown-authority/lifecycle_attestation"));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "methods/method-observation/population_binding"));
        assert!(projection
            .losses
            .iter()
            .all(|loss| loss.retained_in_canonical_package));
    }

    #[test]
    fn v4_package_round_trips_with_complete_identity_retained_as_explicit_loss() {
        let projection = import_research_package(FIXTURE_V4, "map-ecrp-v4")
            .expect("import complete-identity v4 fixture");
        assert_eq!(projection.schema_version, CONTRACT_SCHEMA_VERSION_V2);
        assert_eq!(projection.schema_digest, CONTRACT_SCHEMA_SHA256_V2);

        let original: Value = serde_json::from_str(FIXTURE_V4).expect("parse v4 fixture");
        let exported: Value =
            serde_json::from_str(&export_research_package(&projection).expect("export v4 fixture"))
                .expect("parse v4 export");
        assert_eq!(canonicalize(&original), canonicalize(&exported));
        assert_eq!(projection.revision_id, "rev-4");
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "sources/source-current/lifecycle_attestation"));
        assert!(projection
            .losses
            .iter()
            .any(|loss| loss.path == "methods/method-observation/population_binding"));
        assert!(projection
            .losses
            .iter()
            .all(|loss| loss.retained_in_canonical_package));
    }

    #[test]
    fn projected_ids_are_namespaced_per_map_and_package_revision() {
        let first = import_research_package(FIXTURE, "map-first").expect("first projection");
        let second = import_research_package(FIXTURE, "map-second").expect("second projection");
        let first_ids: BTreeSet<_> = first.nodes.iter().map(|node| node.id.as_str()).collect();
        let second_ids: BTreeSet<_> = second.nodes.iter().map(|node| node.id.as_str()).collect();
        assert!(first_ids.is_disjoint(&second_ids));
        assert!(first.edges.iter().all(|edge| {
            first_ids.contains(edge.source_node_id.as_str())
                && first_ids.contains(edge.target_node_id.as_str())
        }));
    }

    #[test]
    fn declared_schema_rejects_invalid_nested_values() {
        let mut package: Value = serde_json::from_str(FIXTURE).expect("parse fixture");
        package["methods"][0]["power_status"] = Value::String("certainly_powered".into());
        let error = import_research_package(&package.to_string(), "map-invalid")
            .expect_err("schema-invalid package must fail");
        assert!(error.contains("violates evidence-centered.research-package.v1"));
    }

    #[test]
    fn omitted_optional_collections_are_empty() {
        let mut package: Value = serde_json::from_str(FIXTURE).expect("parse fixture");
        package.as_object_mut().unwrap().remove("evidence");
        package.as_object_mut().unwrap().remove("conclusions");
        for claim in package["claims"].as_array_mut().unwrap() {
            claim.as_object_mut().unwrap().remove("evidence_links");
        }
        let projection = import_research_package(&package.to_string(), "map-claims-only")
            .expect("schema-valid claims-only package");
        assert_eq!(
            projection.nodes.len(),
            package["claims"].as_array().unwrap().len()
        );
        assert!(projection.edges.is_empty());
    }
}
