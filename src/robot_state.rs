//! Dynamic robot state (`robot_state.schema.json`).

use crate::common::{GridCoord, Id, SimTime};
use crate::event::{FaultCause, Severity};
use serde::{Deserialize, Serialize};

/// Component names ArcadiaBench v1 uses in faults. The schema accepts any
/// name matching `^[a-z][a-z0-9_]{0,31}$`, so later versions can add more.
pub const COMPONENTS: [&str; 6] = [
    "wheels",
    "drive_motor",
    "joints",
    "tool",
    "sensors",
    "battery",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobotStatus {
    Idle,
    Moving,
    Working,
    Charging,
    /// Immobilized in soft regolith.
    Stuck,
    /// Has an active disabled fault.
    Failed,
    BeingTowed,
    /// Battery empty.
    Offline,
    /// Permanently gone (cannibalized); never commandable again.
    Lost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthTelemetry {
    pub wheel_slip: f64,
    pub motor_current_ratio: f64,
    pub dust_coverage: f64,
    pub temperature_c: f64,
}

/// A fault the robot currently has; at most one per component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveFault {
    /// One of [`COMPONENTS`] in ArcadiaBench v1.
    pub component: String,
    pub severity: Severity,
    pub cause: FaultCause,
    /// When the fault occurred.
    pub since_t: SimTime,
    /// Whether a field repair can clear it. Always true for degraded faults.
    pub field_repairable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotState {
    pub robot_id: Id,
    pub position: GridCoord,
    pub status: RobotStatus,
    pub battery_wh: f64,
    pub payload_kg: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_command: Option<Id>,
    pub health: HealthTelemetry,
    /// Active faults ordered by component name; empty when healthy.
    pub faults: Vec<ActiveFault>,
    pub in_relay_coverage: bool,
    /// When this state was measured: the observation time, or the time of
    /// last contact for a robot out of coverage.
    pub telemetry_t: SimTime,
}
