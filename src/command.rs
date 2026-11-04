//! Coordinator → simulator robot commands (`command.schema.json`).

use crate::common::{GridCoord, GridRect, Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub command_id: Id,
    pub robot_id: Id,
    /// `message_id` of the approval request authorising this action: approved
    /// by Earth, or past its deadline with fallback `proceed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<Id>,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    MoveTo {
        target: GridCoord,
    },
    Scan {
        area: GridRect,
    },
    Dig {
        area: GridRect,
        target_elevation_m: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spoil_to: Option<GridCoord>,
    },
    Grade {
        area: GridRect,
        target_elevation_m: f64,
    },
    Haul {
        from: GridCoord,
        to: GridCoord,
        mass_kg: f64,
    },
    Compact {
        area: GridRect,
    },
    Sinter {
        area: GridRect,
    },
    Repair {
        target_robot: Id,
    },
    Tow {
        target_robot: Id,
        to: GridCoord,
    },
    Cannibalize {
        target_robot: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        beneficiary_robot: Option<Id>,
    },
    /// Drive to the depot and be serviced: clears every fault and resets wear.
    Service {
        /// The depot's `depot_id`.
        depot: Id,
    },
    Charge {
        /// A station_id or the robot_id of a mobile charger.
        source: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until_wh: Option<f64>,
    },
    Relay {
        position: GridCoord,
    },
    Wait {
        duration_s: f64,
    },
    Cancel {
        target_command: Id,
    },
}

impl Action {
    /// The `type` tag, which doubles as the policy action name.
    pub fn name(&self) -> &'static str {
        match self {
            Action::MoveTo { .. } => "move_to",
            Action::Scan { .. } => "scan",
            Action::Dig { .. } => "dig",
            Action::Grade { .. } => "grade",
            Action::Haul { .. } => "haul",
            Action::Compact { .. } => "compact",
            Action::Sinter { .. } => "sinter",
            Action::Repair { .. } => "repair",
            Action::Tow { .. } => "tow",
            Action::Cannibalize { .. } => "cannibalize",
            Action::Service { .. } => "service",
            Action::Charge { .. } => "charge",
            Action::Relay { .. } => "relay",
            Action::Wait { .. } => "wait",
            Action::Cancel { .. } => "cancel",
        }
    }
}
