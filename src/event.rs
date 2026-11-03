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
        component: String,
        severity: Severity,
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
    Degraded,
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
