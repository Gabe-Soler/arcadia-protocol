//! Simulator → coordinator events (`event.schema.json`).
//!
//! Every enum here has an `Unknown` variant for values added by a later
//! minor version (`docs/semantics.md` §1.6).

use crate::common::{Duration, Id, SimTime};
use crate::earth::EarthToSite;
use crate::site_map::CellObservation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub t: SimTime,
    #[serde(flatten)]
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    CommandAccepted {
        command_id: Id,
        robot_id: Id,
        estimated_duration_s: Duration,
    },
    CommandRejected {
        command_id: Id,
        robot_id: Id,
        reason: RejectReason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    CommandCompleted {
        command_id: Id,
        robot_id: Id,
        energy_used_wh: f64,
    },
    CommandFailed {
        command_id: Id,
        robot_id: Id,
        reason: FailReason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        energy_used_wh: f64,
    },
    ScanResult {
        command_id: Id,
        robot_id: Id,
        cells: Vec<CellObservation>,
    },
    RobotFault {
        robot_id: Id,
        cause: FaultCause,
        /// One of [`crate::COMPONENTS`] in ArcadiaBench v1.
        component: String,
        severity: Severity,
        /// Whether a field repair can clear it. Always true for degraded faults.
        field_repairable: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    Weather {
        condition: WeatherCondition,
        optical_depth: f64,
        temperature_c: f64,
    },
    Power {
        solar_w: f64,
        fission_w: f64,
        demand_w: f64,
        storage_wh: f64,
        storage_capacity_wh: f64,
        daylight: bool,
    },
    Comms {
        link: Link,
        cause: CommsCause,
        one_way_delay_s: Duration,
        /// While the link is down: when it is next scheduled to come up.
        /// Absent when nobody can know, as in an unplanned outage.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_restore_t: Option<SimTime>,
        /// While the link is up in a pass window: when it is next scheduled
        /// to go down. Since 1.1.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window_end_t: Option<SimTime>,
    },
    EarthMessage {
        sent_t: SimTime,
        message: EarthToSite,
    },
    /// An event type this version does not know. A coordinator ignores it.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    UnknownRobot,
    Busy,
    MissingCapability,
    InvalidTarget,
    OutOfCoverage,
    RobotDisabled,
    /// The action's authority level needs an Earth approval not yet obtained.
    ApprovalRequired,
    /// Earth vetoed the cited approval request.
    Vetoed,
    /// A reason this version does not know. The command was still rejected.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailReason {
    Fault,
    Stuck,
    InsufficientEnergy,
    OutOfCoverage,
    Blocked,
    Cancelled,
    TargetLost,
    /// A reason this version does not know. The command still failed.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultCause {
    Wear,
    Random,
    Environmental,
    /// A cause this version does not know.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The robot keeps working with a component-specific penalty.
    Degraded,
    /// The robot stops and is `failed` until the fault is cleared.
    Disabled,
    /// A severity this version does not know.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition {
    Clear,
    DustStorm,
    /// A condition this version does not know.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Link {
    Up,
    Down,
    /// A link state this version does not know.
    #[serde(other)]
    Unknown,
}

/// Why the Earth link is in its state. `Nominal` while it is up; while it is
/// down, the first of `Conjunction`, `Outage` and `BetweenPasses` that holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommsCause {
    Nominal,
    /// Solar conjunction blackout.
    Conjunction,
    /// Unplanned outage.
    Outage,
    /// The gap between two pass windows. Since 1.1.
    BetweenPasses,
    /// A cause this version does not know. `link` still says whether the
    /// link is up.
    #[serde(other)]
    Unknown,
}
