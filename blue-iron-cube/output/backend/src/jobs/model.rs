use chrono::NaiveDate;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status progression: lead -> scheduled -> in_progress -> completed -> invoiced
/// A job can also be partially_completed (work stopped before 100%) or cancelled.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Lead,
    Scheduled,
    InProgress,
    PartiallyCompleted,
    Completed,
    Invoiced,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lead => "lead",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::PartiallyCompleted => "partially_completed",
            Self::Completed => "completed",
            Self::Invoiced => "invoiced",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lead" => Some(Self::Lead),
            "scheduled" => Some(Self::Scheduled),
            "in_progress" => Some(Self::InProgress),
            "partially_completed" => Some(Self::PartiallyCompleted),
            "completed" => Some(Self::Completed),
            "invoiced" => Some(Self::Invoiced),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LineItemCategory {
    Material,
    Labor,
    Equipment,
    Overhead,
}

/// A french drain installation job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub customer_id: String,
    pub status: JobStatus,

    // Site characteristics — these drive cost variability
    pub site_address: String,
    pub soil_type: String,        // clay, sand, loam, rocky, mixed
    pub yard_grade: String,       // flat, slight, moderate, steep
    pub site_access: String,      // easy, moderate, difficult, hand_carry_only
    pub site_notes: Option<String>,

    // Scope
    pub linear_feet_planned: Option<Decimal>,
    pub linear_feet_actual: Option<Decimal>,
    pub trench_depth_inches: Option<Decimal>,
    pub includes_sump_pump: bool,
    pub includes_downspout_tie_in: bool,
    pub number_of_catch_basins: i32,

    // Schedule
    pub scheduled_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub estimated_crew_hours: Option<Decimal>,
    pub actual_crew_hours: Option<Decimal>,

    // Conditions on the day
    pub weather_conditions: Option<String>,
    pub ground_was_wet: Option<bool>,

    // Completion tracking
    pub completion_pct: i32,              // 0-100
    pub phase_assessment: bool,
    pub phase_excavation: bool,
    pub phase_pipe_install: bool,
    pub phase_backfill: bool,
    pub phase_restoration: bool,
    pub phase_walkthrough: bool,
    pub cancellation_reason: Option<String>,

    // Financials (computed from line items and payments)
    pub quoted_price: Option<Decimal>,
    pub actual_revenue: Option<Decimal>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// An individual cost line on a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLineItem {
    pub id: String,
    pub job_id: String,
    pub category: LineItemCategory,
    pub description: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_cost: Decimal,
    pub total_cost: Decimal,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Cost summary computed from line items and payments.
#[derive(Debug, Serialize)]
pub struct JobCostBreakdown {
    pub job_id: String,
    pub material_cost: Decimal,
    pub labor_cost: Decimal,
    pub equipment_cost: Decimal,
    pub processing_fees: Decimal,   // credit card fees absorbed
    pub overhead_allocated: Decimal,
    pub direct_cost: Decimal,       // material + labor + equipment
    pub true_cost: Decimal,         // direct + processing_fees + overhead
    pub revenue: Decimal,
    pub gross_margin: Decimal,      // revenue - direct
    pub true_margin: Decimal,       // revenue - true cost
    pub true_margin_pct: Decimal,   // true_margin / revenue
    pub completion_pct: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateJob {
    pub customer_id: String,
    pub site_address: String,
    pub soil_type: String,
    pub yard_grade: String,
    pub site_access: String,
    pub site_notes: Option<String>,
    pub linear_feet_planned: Option<Decimal>,
    pub trench_depth_inches: Option<Decimal>,
    pub includes_sump_pump: Option<bool>,
    pub includes_downspout_tie_in: Option<bool>,
    pub number_of_catch_basins: Option<i32>,
    pub scheduled_date: Option<NaiveDate>,
    pub estimated_crew_hours: Option<Decimal>,
    pub quoted_price: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLineItem {
    pub category: LineItemCategory,
    pub description: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_cost: Decimal,
    pub notes: Option<String>,
}
