use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

/// All monetary values are stored as Decimal to avoid floating-point errors.
pub type Money = Decimal;

/// Standard ID prefix format: "entity-uuid7"
pub fn new_id(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::now_v7())
}

/// Read a Decimal from a rusqlite row column (stored as REAL in SQLite).
pub fn get_decimal(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Decimal> {
    let val: f64 = row.get(idx)?;
    Ok(Decimal::from_f64(val).unwrap_or_default())
}

/// Read an optional Decimal from a rusqlite row column.
pub fn get_decimal_opt(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Option<Decimal>> {
    let val: Option<f64> = row.get(idx)?;
    Ok(val.map(|v| Decimal::from_f64(v).unwrap_or_default()))
}

/// Soil types encountered during excavation. Affects labor time and equipment needs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SoilType {
    Clay,
    Sand,
    Loam,
    Rocky,
    Mixed,
}

/// Yard grade affects difficulty and drainage design.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum YardGrade {
    Flat,
    Slight,
    Moderate,
    Steep,
}

/// How accessible is the job site for equipment?
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SiteAccess {
    Easy,
    Moderate,
    Difficult,
    HandCarryOnly,
}
