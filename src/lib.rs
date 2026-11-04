//! Rust types for the Arcadia interface spec.
//!
//! The JSON Schemas in `schemas/` are the source of truth. These types mirror
//! them structurally but are lenient: they do not enforce patterns, ranges or
//! unknown-field rejection. Enable the `validate` feature for strict checking
//! against the embedded schemas. The behaviour behind the messages (decision
//! points, command preconditions and effects, faults, energy, comms,
//! authority and run metrics) is specified in `docs/semantics.md`.
//!
//! Units are SI throughout: metres, seconds, kilograms, watt-hours, watts.

pub mod capability;
pub mod command;
pub mod common;
pub mod earth;
pub mod event;
pub mod event_log;
pub mod messages;
pub mod policy;
pub mod robot_state;
pub mod schemas;
pub mod site_map;
#[cfg(feature = "validate")]
pub mod validate;
pub mod wire;

pub use capability::*;
pub use command::*;
pub use common::*;
pub use earth::*;
pub use event::*;
pub use event_log::*;
pub use messages::*;
pub use policy::*;
pub use robot_state::*;
pub use site_map::*;

/// Protocol version implemented by this crate.
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Two protocol versions are compatible when MAJOR.MINOR match.
pub fn is_compatible(a: &str, b: &str) -> bool {
    fn major_minor(v: &str) -> Option<(&str, &str)> {
        let mut parts = v.split('.');
        let major = parts.next()?;
        let minor = parts.next()?;
        parts.next()?;
        Some((major, minor))
    }
    matches!((major_minor(a), major_minor(b)), (Some(x), Some(y)) if x == y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_is_major_minor() {
        assert!(is_compatible("0.1.0", "0.1.7"));
        assert!(!is_compatible("0.1.0", "0.2.0"));
        assert!(!is_compatible("0.1.0", "1.1.0"));
        assert!(!is_compatible("0.1", "0.1.0"));
        // From 1.0 the rule is unchanged: a minor release may add fields that
        // the previous minor's strict schemas reject.
        assert!(is_compatible("1.0.0", "1.0.3"));
        assert!(!is_compatible("1.0.0", "1.1.0"));
        assert!(!is_compatible("0.2.0", "1.0.0"));
    }

    #[test]
    fn crate_version_matches_protocol_version() {
        assert_eq!(env!("CARGO_PKG_VERSION"), PROTOCOL_VERSION);
    }
}
