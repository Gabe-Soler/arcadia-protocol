//! Drift guard between the JSON Schemas, the example fixtures and the Rust types.
//!
//! For every `examples/valid/<Def>/*.json`:
//!   1. it validates against `<Def>` in its schema,
//!   2. it deserializes into the matching Rust type,
//!   3. re-serializing gives the same JSON (numerically) and still validates.
//!
//! For every `examples/invalid/<Def>/*.json`: the schema rejects it.
//!
//! For every `examples/later/<Def>/*.json`, a message as a simulator on a
//! later minor version might send it: this version's strict schema rejects
//! it, and the Rust type still reads it (`docs/semantics.md` §1.6).

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
    ("Constants", "messages.schema.json", round_trip::<Constants>),
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
    for kind in ["valid", "invalid", "later"] {
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

/// A minor release only adds, so what 1.0 sent is still valid: a 1.0 `hello`
/// has no `constants`, and 1.0 run metrics have no `coverage`.
#[test]
fn messages_from_1_0_stay_valid() {
    let hello = fixtures("valid", "SimToCoordinator")
        .into_iter()
        .find(|(name, _)| name.ends_with("/hello_1_0.json"))
        .expect("the 1.0 hello fixture")
        .1;
    assert_eq!(hello["protocol_version"], "1.0.0");
    let SimToCoordinator::Hello(hello) = serde_json::from_value(hello).unwrap() else {
        panic!("not a hello");
    };
    assert_eq!(hello.scenario.constants, None);

    let metrics = fixtures("valid", "RunMetrics")
        .into_iter()
        .find(|(name, _)| name.ends_with("/time_limit.json"))
        .expect("the 1.0 run metrics fixture")
        .1;
    let metrics: RunMetrics = serde_json::from_value(metrics).unwrap();
    assert_eq!(metrics.coverage, None);
    assert_eq!(metrics.conditions.between_passes_s, None);
}

/// Messages from a later minor version are not valid under this version's
/// strict schemas, but the Rust types read them: unknown fields are ignored.
#[test]
fn later_minor_fixtures_are_rejected_by_the_schema_and_read_by_the_types() {
    let mut failures = Vec::new();
    let mut seen = 0;
    for (def, file, rt) in DEFS {
        let v = validator(file, def).unwrap();
        for (name, json) in fixtures("later", def) {
            seen += 1;
            if v.is_valid(&json) {
                failures.push(format!(
                    "{name}: valid under this version, so it tests nothing"
                ));
            }
            if let Err(e) = rt(&json) {
                failures.push(format!("{name}: Rust type rejected it: {e}"));
            }
        }
    }
    assert!(seen > 0, "no later-minor fixtures");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// What a later minor version adds to an enum reads as that enum's `Unknown`.
#[test]
fn later_minor_values_read_as_unknown() {
    let later = |file: &str| -> SimToCoordinator {
        let (_, json) = fixtures("later", "SimToCoordinator")
            .into_iter()
            .find(|(name, _)| name.ends_with(file))
            .unwrap_or_else(|| panic!("no fixture {file}"));
        serde_json::from_value(json).unwrap()
    };

    let SimToCoordinator::Hello(hello) = later("/hello.json") else {
        panic!("not a hello");
    };
    assert!(is_compatible(&hello.protocol_version, PROTOCOL_VERSION));
    let scenario = &hello.scenario;
    assert_eq!(scenario.power.mode, PowerMode::Unknown);
    assert_eq!(scenario.power.solar_peak_w, 100_000.0);
    assert_eq!(scenario.policy.level3.default_fallback, Fallback::Unknown);
    assert_eq!(scenario.site.cells[0].cell.surface, Some(Surface::Unknown));
    let driller = &scenario.robots[0];
    assert_eq!(driller.capabilities[0].name, CapabilityName::Unknown);
    assert!(driller.capability(CapabilityName::Haul).is_some());
    let constants = scenario.constants.as_ref().expect("constants");
    assert_eq!(constants.earth_response_mean_s, 14_400.0);

    let SimToCoordinator::Observation(obs) = later("/observation.json") else {
        panic!("not an observation");
    };
    let kinds: Vec<&EventKind> = obs.events.iter().map(|e| &e.kind).collect();
    assert_eq!(kinds.len(), 8);
    assert_eq!(*kinds[0], EventKind::Unknown);
    assert_eq!(obs.events[0].t, 250_400.0);
    assert!(matches!(
        kinds[1],
        EventKind::Comms {
            link: Link::Down,
            cause: CommsCause::Unknown,
            expected_restore_t: None,
            window_end_t: None,
            ..
        }
    ));
    assert!(matches!(
        kinds[2],
        EventKind::CommandRejected {
            reason: RejectReason::Unknown,
            ..
        }
    ));
    assert!(matches!(
        kinds[3],
        EventKind::CommandFailed {
            reason: FailReason::Unknown,
            ..
        }
    ));
    assert!(matches!(
        kinds[4],
        EventKind::RobotFault {
            cause: FaultCause::Unknown,
            severity: Severity::Unknown,
            ..
        }
    ));
    assert!(matches!(
        kinds[5],
        EventKind::Weather {
            condition: WeatherCondition::Unknown,
            ..
        }
    ));
    let EventKind::EarthMessage { message, .. } = kinds[6] else {
        panic!("not an earth_message");
    };
    assert_eq!(message.message_id, "earth-0031");
    assert_eq!(message.body, Uplink::Unknown);
    let EventKind::EarthMessage { message, .. } = kinds[7] else {
        panic!("not an earth_message");
    };
    assert!(matches!(
        &message.body,
        Uplink::Goal {
            objective: Objective::Unknown,
            priority: 5,
            ..
        }
    ));
    let robot = &obs.robots[0];
    assert_eq!(robot.status, RobotStatus::Unknown);
    assert_eq!(robot.faults[0].severity, Severity::Unknown);
    assert_eq!(robot.tank_wh, None);

    let SimToCoordinator::End(end) = later("/end.json") else {
        panic!("not an end");
    };
    assert_eq!(end.reason, EndReason::Unknown);

    let (_, record) = fixtures("later", "LogRecord").remove(0);
    let record: LogRecord = serde_json::from_value(record).unwrap();
    assert_eq!(record.t, 251_000.0);
    assert_eq!(record.entry, LogEntry::Unknown);
}

/// The enums a coordinator sends stay closed: a simulator rejects a command
/// or a message to Earth of a type it does not know.
#[test]
fn what_a_coordinator_sends_has_no_unknown() {
    let command = serde_json::json!({
        "type": "drill", "command_id": "cmd-1", "robot_id": "dr-1", "depth_m": 2.0
    });
    assert!(serde_json::from_value::<Command>(command).is_err());
    let downlink = serde_json::json!({
        "type": "request_resupply", "message_id": "site-1", "summary": "Send parts."
    });
    assert!(serde_json::from_value::<SiteToEarth>(downlink).is_err());
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
