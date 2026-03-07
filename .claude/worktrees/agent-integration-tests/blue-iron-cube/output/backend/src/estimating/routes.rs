use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::new_id};
use super::model::*;
use super::regression;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/estimates", post(create_estimate))
        .route("/api/estimates/model-stats", get(model_stats))
        .route("/api/estimates/retrain", post(retrain))
        .route("/api/estimates/:id", get(get_estimate))
}

async fn create_estimate(
    State(state): State<Arc<AppState>>,
    Json(input): Json<EstimateRequest>,
) -> Result<Json<Estimate>, AppError> {
    let conn = state.db.conn();

    // Load latest model weights from DB
    let model = conn.query_row(
        "SELECT version, weights, intercept, r_squared, sample_size FROM regression_models ORDER BY version DESC LIMIT 1",
        [],
        |row| {
            let weights_json: String = row.get(1)?;
            let weights: Vec<f64> = serde_json::from_str(&weights_json).unwrap_or_default();
            Ok((row.get::<_, i64>(0)?, weights, row.get::<_, f64>(2)?, row.get::<_, f64>(3)?, row.get::<_, i64>(4)?))
        },
    ).map_err(|_| AppError::BadRequest("No trained model found. POST /api/estimates/retrain first.".into()))?;

    let (version, weights, intercept, r_squared, sample_size) = model;

    // Build feature vector
    let lf = input.linear_feet.to_f64().unwrap_or(0.0);
    let depth = input.trench_depth_inches.to_f64().unwrap_or(0.0);
    let mut features = vec![lf, depth];
    let soil_cats = ["clay", "rocky", "sand", "loam"];
    for cat in &soil_cats {
        features.push(if input.soil_type == *cat { 1.0 } else { 0.0 });
    }
    let grade_cats = ["slight", "moderate", "steep"];
    for cat in &grade_cats {
        features.push(if input.yard_grade == *cat { 1.0 } else { 0.0 });
    }
    let access_cats = ["moderate", "difficult", "hand_carry_only"];
    for cat in &access_cats {
        features.push(if input.site_access == *cat { 1.0 } else { 0.0 });
    }
    features.push(if input.includes_sump_pump.unwrap_or(false) { 1.0 } else { 0.0 });
    features.push(if input.includes_downspout_tie_in.unwrap_or(false) { 1.0 } else { 0.0 });
    features.push(input.number_of_catch_basins.unwrap_or(0) as f64);

    let predicted_direct = regression::predict(&weights, intercept, &features);

    // Estimate cost breakdown (rough split based on industry norms)
    let material_pct = 0.45;
    let labor_pct = 0.40;
    let equipment_pct = 0.15;

    let material = predicted_direct * material_pct;
    let labor = predicted_direct * labor_pct;
    let equipment = predicted_direct * equipment_pct;

    // Overhead from recovery rate
    let overhead_rate: f64 = conn.query_row(
        "SELECT COALESCE(
            (SELECT SUM(monthly_budget) FROM overhead_categories) /
            NULLIF((SELECT AVG(COALESCE(actual_revenue, quoted_price)) FROM jobs WHERE status IN ('completed','invoiced')), 0),
            0.15
        )", [], |r| r.get(0),
    ).unwrap_or(0.15);

    let overhead = predicted_direct * overhead_rate;
    let total_cost = predicted_direct + overhead;
    let margin_pct = input.desired_margin_pct.to_f64().unwrap_or(25.0) / 100.0;
    let quoted_price = total_cost / (1.0 - margin_pct);

    // Estimate crew hours (rough: ~1 hour per 4 linear feet for a 2-person crew)
    let crew_hours = lf / 4.0;

    let confidence = if sample_size < 10 {
        format!("Low confidence ({} jobs). Need 20+ for reliable estimates.", sample_size)
    } else if sample_size < 20 {
        format!("Moderate confidence ({} jobs, R²={:.2}). Predictions within ~15%.", sample_size, r_squared)
    } else {
        format!("Good confidence ({} jobs, R²={:.2}). Predictions within ~10%.", sample_size, r_squared)
    };

    let id = new_id("est");
    use rust_decimal::prelude::FromPrimitive;
    let estimate = Estimate {
        id: id.clone(),
        customer_id: input.customer_id,
        linear_feet: input.linear_feet,
        trench_depth_inches: input.trench_depth_inches,
        soil_type: input.soil_type,
        yard_grade: input.yard_grade,
        site_access: input.site_access,
        includes_sump_pump: input.includes_sump_pump.unwrap_or(false),
        includes_downspout_tie_in: input.includes_downspout_tie_in.unwrap_or(false),
        number_of_catch_basins: input.number_of_catch_basins.unwrap_or(0),
        predicted_material_cost: Decimal::from_f64(material).unwrap_or_default(),
        predicted_labor_cost: Decimal::from_f64(labor).unwrap_or_default(),
        predicted_equipment_cost: Decimal::from_f64(equipment).unwrap_or_default(),
        predicted_direct_cost: Decimal::from_f64(predicted_direct).unwrap_or_default(),
        predicted_overhead: Decimal::from_f64(overhead).unwrap_or_default(),
        predicted_total_cost: Decimal::from_f64(total_cost).unwrap_or_default(),
        predicted_crew_hours: Decimal::from_f64(crew_hours).unwrap_or_default(),
        desired_margin_pct: input.desired_margin_pct,
        quoted_price: Decimal::from_f64(quoted_price).unwrap_or_default(),
        model_version: version,
        confidence_note: confidence,
        sample_size,
        created_at: chrono::Utc::now().naive_utc(),
    };

    // Save the estimate
    conn.execute(
        "INSERT INTO estimates (id, customer_id, linear_feet, trench_depth_inches, soil_type,
            yard_grade, site_access, includes_sump_pump, includes_downspout_tie_in,
            number_of_catch_basins, predicted_direct_cost, predicted_overhead,
            predicted_total_cost, predicted_crew_hours, desired_margin_pct, quoted_price,
            model_version, confidence_note, sample_size)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        rusqlite::params![
            id, estimate.customer_id,
            estimate.linear_feet.to_f64().unwrap_or(0.0),
            estimate.trench_depth_inches.to_f64().unwrap_or(0.0),
            estimate.soil_type, estimate.yard_grade, estimate.site_access,
            estimate.includes_sump_pump, estimate.includes_downspout_tie_in,
            estimate.number_of_catch_basins, predicted_direct, overhead, total_cost,
            crew_hours, estimate.desired_margin_pct.to_f64().unwrap_or(0.0),
            quoted_price, version, estimate.confidence_note,
            sample_size,
        ],
    )?;

    Ok(Json(estimate))
}

async fn get_estimate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    let row = conn.query_row(
        "SELECT id, customer_id, linear_feet, quoted_price, confidence_note, created_at
         FROM estimates WHERE id = ?1",
        [&id],
        |row| Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "customer_id": row.get::<_, Option<String>>(1)?,
            "linear_feet": row.get::<_, f64>(2)?,
            "quoted_price": row.get::<_, f64>(3)?,
            "confidence_note": row.get::<_, String>(4)?,
            "created_at": row.get::<_, String>(5)?,
        })),
    ).map_err(|_| AppError::NotFound(format!("Estimate {} not found", id)))?;
    Ok(Json(row))
}

async fn model_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ModelStats>, AppError> {
    let conn = state.db.conn();
    let stats = conn.query_row(
        "SELECT version, trained_at, sample_size, r_squared, mae, weights, intercept
         FROM regression_models ORDER BY version DESC LIMIT 1",
        [],
        |row| {
            let weights_json: String = row.get(5)?;
            let weights: Vec<f64> = serde_json::from_str(&weights_json).unwrap_or_default();
            Ok(ModelStats {
                version: row.get(0)?,
                trained_at: row.get(1)?,
                sample_size: row.get(2)?,
                r_squared: row.get(3)?,
                mean_absolute_error: row.get(4)?,
                feature_names: regression::FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
                feature_weights: weights,
                intercept: row.get(6)?,
            })
        },
    ).map_err(|_| AppError::NotFound("No trained model found. POST /api/estimates/retrain first.".into()))?;
    Ok(Json(stats))
}

async fn retrain(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ModelStats>, AppError> {
    let conn = state.db.conn();
    let (weights, intercept, r_squared, sample_size) = regression::train(&conn)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Compute MAE on training data
    let (x, y, _) = regression::build_training_data(&conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mae: f64 = x.outer_iter().zip(y.iter()).map(|(row, target)| {
        let pred = regression::predict(&weights, intercept, row.as_slice().unwrap());
        (pred - target).abs()
    }).sum::<f64>() / sample_size as f64;

    let weights_json = serde_json::to_string(&weights).unwrap();
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM regression_models", [], |r| r.get(0),
    )?;

    conn.execute(
        "INSERT INTO regression_models (version, sample_size, r_squared, mae, weights, intercept)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![version, sample_size, r_squared, mae, weights_json, intercept],
    )?;

    Ok(Json(ModelStats {
        version,
        trained_at: chrono::Utc::now().naive_utc(),
        sample_size,
        r_squared,
        mean_absolute_error: mae,
        feature_names: regression::FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
        feature_weights: weights,
        intercept,
    }))
}
