//! Shared primitives (`common.schema.json`).

use serde::{Deserialize, Serialize};

/// Opaque identifier (`^[A-Za-z0-9_.:-]{1,64}$`).
pub type Id = String;
/// Seconds since mission start.
pub type SimTime = f64;
/// Seconds.
pub type Duration = f64;

/// Length of one sol in seconds.
pub const SOL_S: f64 = 88_775.0;

/// Integer cell coordinate; x east, y north, origin at the south-west corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridCoord {
    pub x: u32,
    pub y: u32,
}

/// Inclusive rectangle of cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridRect {
    pub min: GridCoord,
    pub max: GridCoord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub name: String,
    pub version: String,
}
