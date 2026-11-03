//! Dynamic robot state (`robot_state.schema.json`).

use crate::common::{GridCoord, Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobotStatus {
    Idle,
    Moving,
    Working,
    Charging,
    Stuck,
    Failed,
    BeingTowed,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthTelemetry {
    pub wheel_slip: f64,
    pub motor_current_ratio: f64,
    pub dust_coverage: f64,
    pub temperature_c: f64,
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
    pub in_relay_coverage: bool,
}
