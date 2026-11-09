//! Simulator ↔ coordinator loop envelope (`messages.schema.json`).

use crate::capability::{BaseStation, ChargeStation, Depot, PowerSystem, RobotSpec};
use crate::command::Command;
use crate::common::{Id, SimTime, SoftwareInfo};
use crate::earth::SiteToEarth;
use crate::event::Event;
use crate::policy::Policy;
use crate::robot_state::RobotState;
use crate::site_map::SiteMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimToCoordinator {
    /// Boxed: the scenario is large and sent once.
    Hello(Box<Hello>),
    Observation(Observation),
    End(End),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoordinatorToSim {
    Welcome(Welcome),
    Decision(Decision),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    pub protocol_version: String,
    pub simulator: SoftwareInfo,
    pub run_id: Id,
    pub seed: u64,
    pub scenario: Scenario,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub scenario_id: Id,
    pub start_t: SimTime,
    pub time_limit_t: SimTime,
    pub sol_length_s: f64,
    /// Prior (orbital) knowledge, not ground truth.
    pub site: SiteMap,
    pub robots: Vec<RobotSpec>,
    pub stations: Vec<ChargeStation>,
    pub depots: Vec<Depot>,
    pub power: PowerSystem,
    pub base_station: BaseStation,
    pub policy: Policy,
    /// The simulator constants a coordinator plans with. A simulator on 1.1
    /// or later always sends them; a 1.0 scenario has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constants: Option<Constants>,
}

/// Simulator constants that a coordinator needs to plan (`Scenario.constants`,
/// since 1.1). `docs/semantics.md` uses each where it applies, and its
/// appendix lists them. Work is counted in the repair capability's unit, a
/// health fraction, and is done at the executor's repair `rate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constants {
    /// `grade` only accepts a cell within this distance of its target, m.
    pub grade_window_m: f64,
    /// Mean distance a cell settles when compacted, m.
    pub compaction_settlement_m: f64,
    /// The shortest any work step but loading and dumping can take, s.
    pub min_work_duration_s: f64,
    /// Density of boulders, kg/m³. A boulder is a sphere of rock.
    pub rock_density_kg_m3: f64,
    /// Repair work per degraded fault.
    pub repair_work_degraded: f64,
    /// Repair work per disabled, field-repairable fault.
    pub repair_work_disabled: f64,
    /// Repair work to free a stuck robot.
    pub repair_work_stuck: f64,
    /// Repair work per unit of wear the repair removes.
    pub repair_work_per_wear_removed: f64,
    /// Fraction of each component's wear a repair removes.
    pub repair_wear_removed_fraction: f64,
    /// Work to strip a robot for parts.
    pub strip_work: f64,
    /// Work to install cannibalized parts.
    pub install_work: f64,
    /// Speed of a tow as a fraction of the tower's `max_speed_m_s`.
    pub tow_speed_factor: f64,
    /// Drive energy of a tow as a multiple of the tower's `drive_energy_wh_per_m`.
    pub tow_energy_factor: f64,
    /// How long a depot service takes, s.
    pub depot_service_s: f64,
    /// Robots a depot services at once. The rest wait, first come, first served.
    pub depot_capacity: u32,
    /// Ambient temperature below which robot heaters run, °C.
    pub heater_threshold_c: f64,
    /// Heater draw as a multiple of the robot's `idle_power_w`, added to idle drain.
    pub heater_power_factor: f64,
    /// Radio power a robot draws while holding as a relay, on top of idle drain, W.
    pub relay_radio_w: f64,
    /// How long an idle robot stays out of coverage before it drives back
    /// toward the base station on its own, s.
    pub lost_link_timeout_s: f64,
    /// Mission control's mean time to answer an approval request once it
    /// arrives at Earth, s.
    pub earth_response_mean_s: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Welcome {
    pub protocol_version: String,
    pub coordinator: SoftwareInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub seq: u64,
    pub t: SimTime,
    pub events: Vec<Event>,
    pub robots: Vec<RobotState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    /// Echoes the observation's seq.
    pub seq: u64,
    pub commands: Vec<Command>,
    pub to_earth: Vec<SiteToEarth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake_at: Option<SimTime>,
}

impl Decision {
    /// A decision that does nothing.
    pub fn empty(seq: u64) -> Self {
        Self {
            seq,
            commands: Vec::new(),
            to_earth: Vec::new(),
            wake_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct End {
    pub t: SimTime,
    pub reason: EndReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    GoalsComplete,
    TimeLimit,
    FleetLost,
    CoordinatorError,
    SimulatorError,
    /// A reason from a later minor version. The run has ended all the same.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    VersionMismatch,
    InvalidMessage,
    UnexpectedMessage,
    Internal,
    /// A code from a later minor version.
    #[serde(other)]
    Unknown,
}
