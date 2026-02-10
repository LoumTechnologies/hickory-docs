use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CrewRole {
    Foreman,
    Operator,        // runs mini excavator
    Laborer,
    Apprentice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub hourly_rate: Decimal,
    pub phone: Option<String>,
    pub certifications: Option<String>,  // comma-separated
    pub hire_date: NaiveDate,
    pub active: bool,
    pub created_at: NaiveDateTime,
}

/// A block of time worked by one crew member on one job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    pub crew_member_id: String,
    pub job_id: String,
    pub date: NaiveDate,
    pub hours_regular: Decimal,
    pub hours_overtime: Decimal,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCrewMember {
    pub name: String,
    pub role: String,
    pub hourly_rate: Decimal,
    pub phone: Option<String>,
    pub certifications: Option<String>,
    pub hire_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct CreateTimeEntry {
    pub job_id: String,
    pub date: NaiveDate,
    pub hours_regular: Decimal,
    pub hours_overtime: Option<Decimal>,
    pub notes: Option<String>,
}
