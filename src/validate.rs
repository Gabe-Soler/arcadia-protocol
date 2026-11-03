//! Strict validation against the embedded JSON Schemas (feature `validate`).
//!
//! ```ignore
//! let v = arcadia_protocol::validate::validator("messages.schema.json", "Decision")?;
//! v.validate(&json_value)?;
//! ```

use crate::schemas::{ALL, BASE_URI, def_uri};
use jsonschema::Registry;
use serde_json::Value;

/// Compiled validator (re-exported so callers need no direct `jsonschema` dependency).
pub use jsonschema::Validator;

/// Build a validator for one definition, e.g. `("command.schema.json", "Command")`.
/// All schemas are registered up front and no network access is attempted.
pub fn validator(file: &str, def: &str) -> Result<Validator, String> {
    let mut builder = Registry::new();
    for (name, text) in ALL {
        let schema: Value =
            serde_json::from_str(text).map_err(|e| format!("{name} is not valid JSON: {e}"))?;
        builder = builder
            .add(format!("{BASE_URI}{name}"), schema)
            .map_err(|e| format!("{name}: {e}"))?;
    }
    let registry = builder.prepare().map_err(|e| e.to_string())?;
    jsonschema::options()
        .offline()
        .with_registry(&registry)
        .build(&serde_json::json!({ "$ref": def_uri(file, def) }))
        .map_err(|e| format!("{file}#{def}: {e}"))
}
