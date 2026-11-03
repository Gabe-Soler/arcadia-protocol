//! Run event log, one record per JSONL line (`event_log.schema.json`).

use crate::command::Command;
use crate::common::{Id, SimTime, SoftwareInfo};
use crate::earth::{EarthToSite, SiteToEarth};
use crate::event::Event;
use crate::messages::{EndReason, Scenario};
use crate::robot_state::RobotState;
use crate::site_map::{CellObservation, SiteMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogRecord {
    pub t: SimTime,
    #[serde(flatten)]
    pub entry: LogEntry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LogEntry {
    RunStart {
        protocol_version: String,
        simulator: SoftwareInfo,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        coordinator: Option<SoftwareInfo>,
        run_id: Id,
        seed: u64,
        scenario: Box<Scenario>,
        true_site: Box<SiteMap>,
    },
    /// Ground truth, free of telemetry noise.
    Truth {
        robots: Vec<RobotState>,
        changed_cells: Vec<CellObservation>,
    },
    Observed {
        event: Event,
    },
    Commanded {
        command: Command,
    },
    EarthUplink {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        deliver_t: Option<SimTime>,
        message: EarthToSite,
    },
    EarthDownlink {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        deliver_t: Option<SimTime>,
        message: SiteToEarth,
    },
    RunEnd {
        reason: EndReason,
    },
}
