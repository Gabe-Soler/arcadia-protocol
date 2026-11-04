//! Simulator → coordinator events (`event.schema.json`).

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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_restore_t: Option<SimTime>,
    },
    EarthMessage {
        sent_t: SimTime,
        message: EarthToSite,
    },
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultCause {
    Wear,
    Random,
    Environmental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The robot keeps working with a component-specific penalty.
    Degraded,
    /// The robot stops and is `failed` until the fault is cleared.
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition {
    Clear,
    DustStorm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Link {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommsCause {
    Nominal,
    Conjunction,
    Outage,
}
