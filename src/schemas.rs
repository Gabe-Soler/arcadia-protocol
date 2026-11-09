//! The JSON Schemas, embedded at compile time.

/// Base URI shared by every schema `$id` in this version.
pub const BASE_URI: &str =
    "https://raw.githubusercontent.com/Gabe-Soler/arcadia-protocol/v1.1.0/schemas/";

macro_rules! schemas {
    ($($file:literal),* $(,)?) => {
        /// `(file name, schema JSON)` for every schema.
        pub const ALL: &[(&str, &str)] = &[
            $(($file, include_str!(concat!("../schemas/", $file)))),*
        ];
    };
}

schemas!(
    "common.schema.json",
    "site_map.schema.json",
    "capability.schema.json",
    "robot_state.schema.json",
    "command.schema.json",
    "event.schema.json",
    "earth.schema.json",
    "policy.schema.json",
    "messages.schema.json",
    "event_log.schema.json",
);

/// Full URI of a definition, e.g. `def_uri("messages.schema.json", "Decision")`.
pub fn def_uri(file: &str, def: &str) -> String {
    format!("{BASE_URI}{file}#/$defs/{def}")
}
