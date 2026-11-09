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
//!
//! # Reading a later minor version
//!
//! From 1.1, a coordinator must be able to read the messages of a simulator
//! on a later minor version (`docs/semantics.md` §1.6). These types do that:
//! they ignore fields they do not know, and every enum a simulator sends has
//! an `Unknown` variant that takes any value this version does not define.
//! The enums a coordinator sends ([`Action`], [`Downlink`]) have none: a
//! simulator rejects what it does not know.

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
pub const PROTOCOL_VERSION: &str = "1.1.0";

/// Whether a simulator and a coordinator on these two protocol versions can
/// run a session together (`docs/semantics.md` §1.6).
///
/// From 1.1, versions with the same MAJOR are compatible: a coordinator
/// ignores what it does not know in the simulator's messages, and sends only
/// what the simulator's version defines. Versions before 1.1 have no such
/// rule, so each is compatible only with its own MAJOR.MINOR.
pub fn is_compatible(a: &str, b: &str) -> bool {
    match (major_minor(a), major_minor(b)) {
        (Some(x), Some(y)) => {
            x == y || (x.0 == y.0 && reads_later_minors(x) && reads_later_minors(y))
        }
        _ => false,
    }
}

/// MAJOR and MINOR of a `MAJOR.MINOR.PATCH` version.
fn major_minor(v: &str) -> Option<(u64, u64)> {
    let mut parts = v.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    parts.next()?;
    Some((major, minor))
}

/// Versions from 1.1 on, whose readers accept a later minor's messages.
fn reads_later_minors((major, minor): (u64, u64)) -> bool {
    major > 1 || (major == 1 && minor >= 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn before_1_1_compatibility_is_major_minor() {
        assert!(is_compatible("0.1.0", "0.1.7"));
        assert!(!is_compatible("0.1.0", "0.2.0"));
        assert!(!is_compatible("0.1.0", "1.1.0"));
        assert!(is_compatible("1.0.0", "1.0.3"));
        assert!(!is_compatible("0.2.0", "1.0.0"));
        // 1.0's strict readers cannot read what 1.1 adds.
        assert!(!is_compatible("1.0.0", "1.1.0"));
        assert!(!is_compatible("1.2.0", "1.0.4"));
    }

    #[test]
    fn from_1_1_compatibility_is_major() {
        assert!(is_compatible("1.1.0", "1.1.3"));
        assert!(is_compatible("1.1.0", "1.2.0"));
        assert!(is_compatible("1.7.2", "1.1.0"));
        assert!(is_compatible("2.0.0", "2.3.1"));
        assert!(!is_compatible("1.1.0", "2.1.0"));
        assert!(!is_compatible("2.0.0", "1.4.0"));
        assert!(is_compatible(PROTOCOL_VERSION, PROTOCOL_VERSION));
    }

    #[test]
    fn malformed_versions_are_incompatible() {
        assert!(!is_compatible("0.1", "0.1.0"));
        assert!(!is_compatible("1.1", "1.1"));
        assert!(!is_compatible("v1.1.0", "1.1.0"));
        assert!(!is_compatible("1.x.0", "1.x.0"));
        assert!(!is_compatible("", "1.1.0"));
    }

    #[test]
    fn crate_version_matches_protocol_version() {
        assert_eq!(env!("CARGO_PKG_VERSION"), PROTOCOL_VERSION);
    }
}
