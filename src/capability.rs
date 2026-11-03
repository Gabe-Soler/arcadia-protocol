//! Capabilities and static robot/infrastructure specs (`capability.schema.json`).

use crate::common::{GridCoord, Id};
use serde::{Deserialize, Serialize};

/// Work units: scan m², dig m³, grade m², haul kg, compact m², sinter m²,
/// repair health-fraction, charge Wh, relay m (coverage radius, no rate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityName {
    Scan,
    Dig,
    Grade,
    Haul,
    Compact,
    Sinter,
    Repair,
    Charge,
    Relay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitySpec {
    pub name: CapabilityName,
    /// Work units per second (0 for relay).
    pub rate: f64,
    pub energy_wh_per_unit: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotSpec {
    pub robot_id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub mass_kg: f64,
    pub battery_capacity_wh: f64,
    pub max_speed_m_s: f64,
    pub drive_energy_wh_per_m: f64,
    pub idle_power_w: f64,
    pub capabilities: Vec<CapabilitySpec>,
}

impl RobotSpec {
    pub fn capability(&self, name: CapabilityName) -> Option<&CapabilitySpec> {
        self.capabilities.iter().find(|c| c.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargeStation {
    pub station_id: Id,
    pub position: GridCoord,
    pub max_power_w: f64,
    pub slots: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Depot {
    pub depot_id: Id,
    pub position: GridCoord,
}
