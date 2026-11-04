//! Capabilities and static robot/infrastructure specs (`capability.schema.json`):
//! robots, charge stations, depots, the power system and the comms base station.

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

/// Maintenance depot; a robot on its cell can be serviced there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Depot {
    pub depot_id: Id,
    pub position: GridCoord,
}

/// Power architecture of the site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerMode {
    /// Fission baseline plus solar and storage (the default).
    Hybrid,
    /// Solar and storage only.
    SolarOnly,
    /// Mostly fission.
    NuclearHeavy,
}

/// The site power grid. Supply is fission plus solar; demand is the essential
/// load plus station charging; storage absorbs the difference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerSystem {
    pub mode: PowerMode,
    /// Constant fission output, W.
    pub fission_w: f64,
    /// Solar output at noon under a clear sky, W.
    pub solar_peak_w: f64,
    /// Capacity of the site battery bank, Wh.
    pub storage_capacity_wh: f64,
    /// Always-on site load, served before any charging, W.
    pub essential_load_w: f64,
}

/// The site comms hub. Robots within `range_m` of it, or of an active relay
/// that is itself in coverage, can receive commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseStation {
    pub position: GridCoord,
    /// Direct coverage radius in metres, measured between cell centres.
    pub range_m: f64,
}
