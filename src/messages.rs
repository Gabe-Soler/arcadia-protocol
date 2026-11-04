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
}
