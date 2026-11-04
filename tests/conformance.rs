//! Drift guard between the JSON Schemas, the example fixtures and the Rust types.
//!
//! For every `examples/valid/<Def>/*.json`:
//!   1. it validates against `<Def>` in its schema,
//!   2. it deserializes into the matching Rust type,
//!   3. re-serializing gives the same JSON (numerically) and still validates.
//!
//! For every `examples/invalid/<Def>/*.json`: the schema rejects it.

use arcadia_protocol::schemas::ALL;
use arcadia_protocol::validate::validator;
use arcadia_protocol::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

type RoundTrip = fn(&Value) -> Result<Value, String>;

fn round_trip<T: DeserializeOwned + Serialize>(v: &Value) -> Result<Value, String> {
    let typed: T = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    serde_json::to_value(&typed).map_err(|e| e.to_string())
}

/// Every definition with fixtures: (fixture dir / def name, schema file, Rust round-trip).
const DEFS: &[(&str, &str, RoundTrip)] = &[
    ("SiteMap", "site_map.schema.json", round_trip::<SiteMap>),
    (
        "RobotSpec",
        "capability.schema.json",
        round_trip::<RobotSpec>,
    ),
    (
        "PowerSystem",
        "capability.schema.json",
        round_trip::<PowerSystem>,
    ),
    (
        "BaseStation",
        "capability.schema.json",
        round_trip::<BaseStation>,
    ),
    (
        "RobotState",
        "robot_state.schema.json",
        round_trip::<RobotState>,
    ),
    (
        "ActiveFault",
        "robot_state.schema.json",
        round_trip::<ActiveFault>,
    ),
    ("Policy", "policy.schema.json", round_trip::<Policy>),
    ("Command", "command.schema.json", round_trip::<Command>),
    ("Event", "event.schema.json", round_trip::<Event>),
    (
        "EarthToSite",
        "earth.schema.json",
        round_trip::<EarthToSite>,
    ),
    (
        "SiteToEarth",
        "earth.schema.json",
        round_trip::<SiteToEarth>,
    ),
    (
        "SimToCoordinator",
        "messages.schema.json",
        round_trip::<SimToCoordinator>,
    ),
    (
        "CoordinatorToSim",
        "messages.schema.json",
        round_trip::<CoordinatorToSim>,
    ),
    (
        "LogRecord",
        "event_log.schema.json",
        round_trip::<LogRecord>,
    ),
    (
        "SiteTruth",
        "event_log.schema.json",
        round_trip::<SiteTruth>,
    ),
    (
        "RunMetrics",
        "event_log.schema.json",
        round_trip::<RunMetrics>,
    ),
];

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn fixtures(kind: &str, def: &str) -> Vec<(String, Value)> {
    let dir = examples().join(kind).join(def);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<_> = entries
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let text = fs::read_to_string(&p).unwrap();
            let v = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("{} is not JSON: {e}", p.display()));
            (
                format!("{kind}/{def}/{}", p.file_name().unwrap().to_string_lossy()),
                v,
            )
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// JSON equality that treats numbers by value (so `1` == `1.0`).
fn same_json(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same_json(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| same_json(v, w)))
        }
        _ => a == b,
    }
}

#[test]
fn schemas_are_valid_2020_12_and_ids_match_file_names() {
    for (name, text) in ALL {
        let schema: Value = serde_json::from_str(text).unwrap();
        jsonschema::meta::validate(&schema)
            .unwrap_or_else(|e| panic!("{name} fails the meta-schema: {e}"));
        assert_eq!(
            schema["$id"].as_str().unwrap(),
            format!("{}{name}", schemas::BASE_URI),
            "$id of {name}"
        );
    }
}

#[test]
fn every_schema_file_is_embedded() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let mut on_disk: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".schema.json"))
        .collect();
    let mut embedded: Vec<String> = ALL.iter().map(|(n, _)| n.to_string()).collect();
    on_disk.sort();
    embedded.sort();
    assert_eq!(on_disk, embedded);
}

#[test]
fn every_fixture_dir_is_covered() {
    for kind in ["valid", "invalid"] {
        for entry in fs::read_dir(examples().join(kind)).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            assert!(
                DEFS.iter().any(|(d, _, _)| *d == name),
                "examples/{kind}/{name} has no entry in DEFS"
            );
        }
    }
    for (def, _, _) in DEFS {
        assert!(
            !fixtures("valid", def).is_empty(),
            "no valid fixtures for {def}"
        );
    }
}

#[test]
fn valid_fixtures_validate_and_round_trip() {
    let mut failures = Vec::new();
    for (def, file, rt) in DEFS {
        let v = validator(file, def).unwrap();
        for (name, json) in fixtures("valid", def) {
            if let Err(e) = v.validate(&json) {
                failures.push(format!("{name}: schema rejected it: {e}"));
                continue;
            }
            match rt(&json) {
                Err(e) => failures.push(format!("{name}: Rust type rejected it: {e}")),
                Ok(back) => {
                    if !same_json(&json, &back) {
                        failures.push(format!(
                            "{name}: Rust round-trip changed the JSON\n  before: {json}\n  after:  {back}"
                        ));
                    } else if let Err(e) = v.validate(&back) {
                        failures.push(format!("{name}: re-serialized JSON is invalid: {e}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn invalid_fixtures_are_rejected() {
    let mut accepted = Vec::new();
    for (def, file, _) in DEFS {
        let v = validator(file, def).unwrap();
        for (name, json) in fixtures("invalid", def) {
            if v.is_valid(&json) {
                accepted.push(name);
            }
        }
    }
    assert!(accepted.is_empty(), "schema accepted: {accepted:?}");
}

/// `Action::name()` doubles as the policy action name, so it must equal the
/// wire `type` tag. Every command in the schema also needs a valid fixture.
#[test]
fn every_command_type_has_a_fixture_and_matches_its_action_name() {
    let (_, text) = ALL
        .iter()
        .find(|(name, _)| *name == "command.schema.json")
        .unwrap();
    let schema: Value = serde_json::from_str(text).unwrap();
    let defs = &schema["$defs"];
    let in_schema: BTreeSet<String> = defs["Command"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let def = r["$ref"].as_str().unwrap().trim_start_matches("#/$defs/");
            defs[def]["properties"]["type"]["const"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let mut in_fixtures = BTreeSet::new();
    for (name, json) in fixtures("valid", "Command") {
        let command: Command = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json["type"], command.action.name(), "{name}");
        in_fixtures.insert(command.action.name().to_string());
    }
    assert_eq!(in_schema, in_fixtures);
}

#[test]
fn component_vocabulary_matches_the_schema() {
    let v = validator("event.schema.json", "Component").unwrap();
    for c in COMPONENTS {
        assert!(v.is_valid(&Value::from(c)), "{c}");
    }
}
