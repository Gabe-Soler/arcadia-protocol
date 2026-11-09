//! Terrain grid (`site_map.schema.json`).

use crate::common::{GridCoord, Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteMap {
    pub site_id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub width: u32,
    pub height: u32,
    pub cell_size_m: f64,
    /// Sparse; absent coordinates are unknown.
    pub cells: Vec<CellObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellObservation {
    pub at: GridCoord,
    pub cell: Cell,
}

/// Every field is optional: absent means unknown to the holder of the map.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regolith: Option<Regolith>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boulders: Option<Vec<Boulder>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<Surface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ice_depth_m: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Regolith {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bulk_density_kg_m3: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cohesion_kpa: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Boulder {
    pub diameter_m: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_kg: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Natural,
    Cleared,
    Graded,
    Compacted,
    Stabilized,
    Berm,
    Stockpile,
    /// A surface state from a later minor version.
    #[serde(other)]
    Unknown,
}
