use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// An estimate generated for a potential job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub id: String,
    pub customer_id: Option<String>,

    // Input features
    pub linear_feet: Decimal,
    pub trench_depth_inches: Decimal,
    pub soil_type: String,
    pub yard_grade: String,
    pub site_access: String,
    pub includes_sump_pump: bool,
    pub includes_downspout_tie_in: bool,
    pub number_of_catch_basins: i32,

    // Predictions
    pub predicted_material_cost: Decimal,
    pub predicted_labor_cost: Decimal,
    pub predicted_equipment_cost: Decimal,
    pub predicted_direct_cost: Decimal,
    pub predicted_overhead: Decimal,
    pub predicted_total_cost: Decimal,
    pub predicted_crew_hours: Decimal,

    // Quote
    pub desired_margin_pct: Decimal,
    pub quoted_price: Decimal,

    // Model metadata
    pub model_version: i64,
    pub confidence_note: String,
    pub sample_size: i64,

    pub created_at: NaiveDateTime,
}

/// Snapshot of a trained regression model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub version: i64,
    pub trained_at: NaiveDateTime,
    pub sample_size: i64,
    pub r_squared: f64,
    pub mean_absolute_error: f64,
    pub feature_names: Vec<String>,
    pub feature_weights: Vec<f64>,
    pub intercept: f64,
}

#[derive(Debug, Deserialize)]
pub struct EstimateRequest {
    pub customer_id: Option<String>,
    pub linear_feet: Decimal,
    pub trench_depth_inches: Decimal,
    pub soil_type: String,
    pub yard_grade: String,
    pub site_access: String,
    pub includes_sump_pump: Option<bool>,
    pub includes_downspout_tie_in: Option<bool>,
    pub number_of_catch_basins: Option<i32>,
    /// Target profit margin as a percentage (e.g., 25 for 25%)
    pub desired_margin_pct: Decimal,
}
