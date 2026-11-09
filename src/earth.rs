//! Earth ↔ site messages (`earth.schema.json`). These are the messages that
//! experience light-time delay and blackouts; robot commands do not.

use crate::common::{GridCoord, GridRect, Id, SimTime};
use crate::policy::Policy;
use serde::{Deserialize, Serialize};

/// Earth → site.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarthToSite {
    pub message_id: Id,
    #[serde(flatten)]
    pub body: Uplink,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Uplink {
    Goal {
        goal_id: Id,
        /// Higher runs first.
        priority: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        deadline_t: Option<SimTime>,
        objective: Objective,
    },
    Approve {
        request_id: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    Veto {
        request_id: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    Reprioritize {
        priorities: Vec<GoalPriority>,
    },
    PolicyUpdate {
        policy: Policy,
    },
    /// A message type from a later minor version. A coordinator ignores it.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Objective {
    LandingPad {
        pad: LandingPadSpec,
    },
    /// An objective from a later minor version: a goal this version cannot
    /// pursue.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LandingPadSpec {
    pub area: GridRect,
    pub target_elevation_m: f64,
    pub surface_tolerance_m: f64,
    pub max_slope_deg: f64,
    pub stabilize: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub berm: Option<BermSpec>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BermSpec {
    pub cells: Vec<GridCoord>,
    pub height_m: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalPriority {
    pub goal_id: Id,
    pub priority: i64,
}

/// Site → Earth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteToEarth {
    pub message_id: Id,
    #[serde(flatten)]
    pub body: Downlink,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Downlink {
    /// Level-2 action already taken.
    Notify {
        action: String,
        summary: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        commands: Option<Vec<Id>>,
    },
    /// Ask before a level-3/4 action.
    ApprovalRequest {
        action: String,
        summary: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decide_by_t: Option<SimTime>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fallback: Option<Fallback>,
    },
    StatusReport {
        summary: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        progress: Option<Vec<GoalProgress>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fallback {
    Proceed,
    Abort,
    /// A fallback from a later minor version, as a policy may carry. A
    /// coordinator never sends it.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalProgress {
    pub goal_id: Id,
    pub fraction_complete: f64,
}
