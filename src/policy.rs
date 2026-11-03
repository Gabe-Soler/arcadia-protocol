//! Authority policy (`policy.schema.json`).

use crate::common::Duration;
use crate::earth::Fallback;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 1 autonomous, 2 notify, 3 approve-if-reachable, 4 hold for Earth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct AuthorityLevel(u8);

impl AuthorityLevel {
    pub const AUTONOMOUS: Self = Self(1);
    pub const NOTIFY: Self = Self(2);
    pub const APPROVE_IF_REACHABLE: Self = Self(3);
    pub const HOLD_FOR_EARTH: Self = Self(4);

    pub fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for AuthorityLevel {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if (1..=4).contains(&v) {
            Ok(Self(v))
        } else {
            Err(format!("authority level must be 1-4, got {v}"))
        }
    }
}

impl From<AuthorityLevel> for u8 {
    fn from(l: AuthorityLevel) -> u8 {
        l.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Policy {
    pub policy_version: u32,
    pub default_level: AuthorityLevel,
    /// Action name → level. Unlisted actions use `default_level`.
    pub levels: BTreeMap<String, AuthorityLevel>,
    pub level3: Level3Rule,
}

impl Policy {
    pub fn level_for(&self, action: &str) -> AuthorityLevel {
        self.levels
            .get(action)
            .copied()
            .unwrap_or(self.default_level)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Level3Rule {
    pub max_wait_s: Duration,
    pub default_fallback: Fallback,
}
