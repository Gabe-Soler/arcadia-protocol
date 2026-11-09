//! Run event log, one record per JSONL line (`event_log.schema.json`).

use crate::command::Command;
use crate::common::{Duration, Id, SimTime, SoftwareInfo};
use crate::earth::{EarthToSite, SiteToEarth};
use crate::event::{Event, Link};
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
        site: SiteTruth,
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
        metrics: RunMetrics,
    },
    /// A record kind from a later minor version. A reader skips it.
    #[serde(other)]
    Unknown,
}

/// True state of the site power grid, sky and Earth link at a truth record's `t`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteTruth {
    pub storage_wh: f64,
    pub solar_w: f64,
    pub fission_w: f64,
    /// Essential load plus the charging power stations are delivering.
    pub demand_w: f64,
    pub optical_depth: f64,
    /// Ambient air temperature.
    pub temperature_c: f64,
    pub link: Link,
    pub one_way_delay_s: Duration,
}

/// Outcome of a run, computed from ground truth. `docs/semantics.md` defines
/// every field.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunMetrics {
    /// The run ended `goals_complete`.
    pub completed: bool,
    /// `(end t - start_t) / sol_length_s`.
    pub sols: f64,
    pub decisions: u64,
    pub commands: u64,
    pub commands_rejected: u64,
    pub quality: QualityMetrics,
    pub energy: EnergyMetrics,
    pub fleet: FleetMetrics,
    pub autonomy: AutonomyMetrics,
    pub conditions: ConditionMetrics,
    /// Since 1.1: a simulator on 1.1 or later always writes it, and a 1.0
    /// log has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageMetrics>,
}

/// How often site coverage got in the way (since 1.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageMetrics {
    /// Commands rejected `out_of_coverage`.
    pub out_of_coverage_rejections: u64,
    /// Lost-link drives that robots started on their own.
    pub lost_link_returns: u64,
}

/// The pad at the end of the run, against the goal's spec.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub cells_total: u64,
    pub cells_in_spec: u64,
    pub rms_error_m: f64,
    pub max_abs_error_m: f64,
    pub max_slope_deg: f64,
    pub berm_complete: bool,
    pub verified: bool,
}

/// Energy use and the closest approach to a power deficit.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnergyMetrics {
    pub robot_energy_wh: f64,
    pub grid_energy_wh: f64,
    pub min_storage_wh: f64,
    pub min_storage_fraction: f64,
    pub deficit_s: Duration,
}

/// Robot time and robots lost to faults.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FleetMetrics {
    pub faults: u64,
    pub robot_hours_lost: f64,
    pub robots_lost: u64,
}

/// Traffic to Earth and time spent waiting on it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AutonomyMetrics {
    pub downlink_messages: u64,
    pub approval_requests: u64,
    pub notifications: u64,
    pub status_reports: u64,
    pub approval_wait_s: Duration,
    pub policy_violations: u64,
}

/// How much adversity the run contained.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConditionMetrics {
    pub storm_s: Duration,
    pub blackout_s: Duration,
    pub outage_s: Duration,
    /// Time with the link down between pass windows. Since 1.1: a simulator
    /// on 1.1 or later always writes it, and a 1.0 log has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub between_passes_s: Option<Duration>,
}
