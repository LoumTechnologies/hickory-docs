use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/jobs", get(list_jobs).post(create_job))
        .route("/api/jobs/:id", get(get_job))
        .route("/api/jobs/:id/line-items", post(add_line_item))
        .route("/api/jobs/:id/costs", get(cost_breakdown))
        .route("/api/jobs/:id/complete", post(complete_job))
        .route("/api/jobs/:id/phase", post(update_phase))
        .route("/api/jobs/:id/cancel", post(cancel_job))
}

async fn list_jobs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Job>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, customer_id, status, site_address, soil_type, yard_grade,
                site_access, site_notes, linear_feet_planned, linear_feet_actual,
                trench_depth_inches, includes_sump_pump, includes_downspout_tie_in,
                number_of_catch_basins, scheduled_date, start_date, end_date,
                estimated_crew_hours, actual_crew_hours, weather_conditions,
                ground_was_wet, completion_pct, phase_assessment, phase_excavation,
                phase_pipe_install, phase_backfill, phase_restoration, phase_walkthrough,
                cancellation_reason, quoted_price, actual_revenue, created_at, updated_at
         FROM jobs ORDER BY created_at DESC"
    )?;
    let jobs = stmt.query_map([], |row| {
        Ok(Job {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            status: JobStatus::from_str(&row.get::<_, String>(2)?).unwrap_or(JobStatus::Lead),
            site_address: row.get(3)?,
            soil_type: row.get(4)?,
            yard_grade: row.get(5)?,
            site_access: row.get(6)?,
            site_notes: row.get(7)?,
            linear_feet_planned: get_decimal_opt(row, 8)?,
            linear_feet_actual: get_decimal_opt(row, 9)?,
            trench_depth_inches: get_decimal_opt(row, 10)?,
            includes_sump_pump: row.get(11)?,
            includes_downspout_tie_in: row.get(12)?,
            number_of_catch_basins: row.get(13)?,
            scheduled_date: row.get(14)?,
            start_date: row.get(15)?,
            end_date: row.get(16)?,
            estimated_crew_hours: get_decimal_opt(row, 17)?,
            actual_crew_hours: get_decimal_opt(row, 18)?,
            weather_conditions: row.get(19)?,
            ground_was_wet: row.get(20)?,
            completion_pct: row.get(21)?,
            phase_assessment: row.get(22)?,
            phase_excavation: row.get(23)?,
            phase_pipe_install: row.get(24)?,
            phase_backfill: row.get(25)?,
            phase_restoration: row.get(26)?,
            phase_walkthrough: row.get(27)?,
            cancellation_reason: row.get(28)?,
            quoted_price: get_decimal_opt(row, 29)?,
            actual_revenue: get_decimal_opt(row, 30)?,
            created_at: row.get(31)?,
            updated_at: row.get(32)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(jobs))
}

async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateJob>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("job");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO jobs (id, customer_id, status, site_address, soil_type, yard_grade,
                           site_access, site_notes, linear_feet_planned, trench_depth_inches,
                           includes_sump_pump, includes_downspout_tie_in, number_of_catch_basins,
                           scheduled_date, estimated_crew_hours, quoted_price)
         VALUES (?1, ?2, 'lead', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![
            id, input.customer_id, input.site_address, input.soil_type,
            input.yard_grade, input.site_access, input.site_notes,
            input.linear_feet_planned.map(|d| d.to_f64().unwrap_or(0.0)),
            input.trench_depth_inches.map(|d| d.to_f64().unwrap_or(0.0)),
            input.includes_sump_pump.unwrap_or(false),
            input.includes_downspout_tie_in.unwrap_or(false),
            input.number_of_catch_basins.unwrap_or(0),
            input.scheduled_date,
            input.estimated_crew_hours.map(|d| d.to_f64().unwrap_or(0.0)),
            input.quoted_price.map(|d| d.to_f64().unwrap_or(0.0)),
        ],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    // Get job + line items in one response
    let job_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM jobs WHERE id = ?1", [&id], |r| r.get(0)
    )?;
    if !job_exists {
        return Err(AppError::NotFound(format!("Job {} not found", id)));
    }

    let mut stmt = conn.prepare(
        "SELECT id, category, description, quantity, unit, unit_cost, total_cost, notes, created_at
         FROM job_line_items WHERE job_id = ?1 ORDER BY created_at"
    )?;
    let line_items: Vec<serde_json::Value> = stmt.query_map([&id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "category": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2)?,
            "quantity": row.get::<_, f64>(3)?,
            "unit": row.get::<_, String>(4)?,
            "unit_cost": row.get::<_, f64>(5)?,
            "total_cost": row.get::<_, f64>(6)?,
            "notes": row.get::<_, Option<String>>(7)?,
        }))
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(Json(serde_json::json!({
        "id": id,
        "line_items": line_items,
    })))
}

async fn add_line_item(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    Json(input): Json<CreateLineItem>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("li");
    let total = input.quantity * input.unit_cost;
    let category = match input.category {
        LineItemCategory::Material => "material",
        LineItemCategory::Labor => "labor",
        LineItemCategory::Equipment => "equipment",
        LineItemCategory::Overhead => "overhead",
    };
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, job_id, category, input.description,
            input.quantity.to_f64().unwrap_or(0.0), input.unit,
            input.unit_cost.to_f64().unwrap_or(0.0), total.to_f64().unwrap_or(0.0),
            input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "total_cost": total })))
}

async fn cost_breakdown(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<JobCostBreakdown>, AppError> {
    let conn = state.db.conn();
    let (revenue, completion_pct): (Decimal, i32) = conn.query_row(
        "SELECT COALESCE(actual_revenue, quoted_price, 0), completion_pct FROM jobs WHERE id = ?1",
        [&id], |r| Ok((get_decimal(r, 0)?, r.get(1)?)),
    ).map_err(|_| AppError::NotFound(format!("Job {} not found", id)))?;

    let sum_category = |cat: &str| -> Result<Decimal, AppError> {
        Ok(conn.query_row(
            "SELECT COALESCE(SUM(total_cost), 0) FROM job_line_items WHERE job_id = ?1 AND category = ?2",
            rusqlite::params![&id, cat], |r| get_decimal(r, 0),
        )?)
    };

    let material = sum_category("material")?;
    let labor = sum_category("labor")?;
    let equipment = sum_category("equipment")?;
    let processing_fees = sum_category("processing_fee")?;
    let overhead = sum_category("overhead")?;
    let direct = material + labor + equipment;
    let true_cost = direct + processing_fees + overhead;
    let gross_margin = revenue - direct;
    let true_margin = revenue - true_cost;
    let true_margin_pct = if revenue > Decimal::ZERO {
        true_margin / revenue * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    Ok(Json(JobCostBreakdown {
        job_id: id,
        material_cost: material,
        labor_cost: labor,
        equipment_cost: equipment,
        processing_fees,
        overhead_allocated: overhead,
        direct_cost: direct,
        true_cost,
        revenue,
        gross_margin,
        true_margin,
        true_margin_pct,
        completion_pct,
    }))
}

async fn complete_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    conn.execute(
        "UPDATE jobs SET status = 'completed', completion_pct = 100, end_date = date('now'),
                phase_assessment = 1, phase_excavation = 1, phase_pipe_install = 1,
                phase_backfill = 1, phase_restoration = 1, phase_walkthrough = 1,
                updated_at = datetime('now')
         WHERE id = ?1",
        [&id],
    )?;
    Ok(Json(serde_json::json!({ "status": "completed", "completion_pct": 100 })))
}

#[derive(Debug, serde::Deserialize)]
struct PhaseUpdate {
    phase: String,
    completed: bool,
}

/// Mark individual phases complete/incomplete. Completion percentage auto-computes
/// from the number of completed phases (each phase = ~17%).
async fn update_phase(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(input): Json<PhaseUpdate>,
) -> Result<Json<serde_json::Value>, AppError> {
    let col = match input.phase.as_str() {
        "assessment" => "phase_assessment",
        "excavation" => "phase_excavation",
        "pipe_install" => "phase_pipe_install",
        "backfill" => "phase_backfill",
        "restoration" => "phase_restoration",
        "walkthrough" => "phase_walkthrough",
        _ => return Err(AppError::BadRequest(format!("Unknown phase: {}", input.phase))),
    };
    let conn = state.db.conn();
    // Update the phase
    conn.execute(
        &format!("UPDATE jobs SET {} = ?2, updated_at = datetime('now') WHERE id = ?1", col),
        rusqlite::params![id, input.completed],
    )?;
    // Recompute completion_pct from phases
    let pct: i32 = conn.query_row(
        "SELECT (CAST(phase_assessment AS INTEGER) + CAST(phase_excavation AS INTEGER) +
                 CAST(phase_pipe_install AS INTEGER) + CAST(phase_backfill AS INTEGER) +
                 CAST(phase_restoration AS INTEGER) + CAST(phase_walkthrough AS INTEGER)) * 100 / 6
         FROM jobs WHERE id = ?1",
        [&id], |r| r.get(0),
    )?;
    // Auto-set status based on completion
    let status = if pct == 100 { "completed" } else if pct > 0 { "in_progress" } else { "scheduled" };
    conn.execute(
        "UPDATE jobs SET completion_pct = ?2, status = ?3, updated_at = datetime('now') WHERE id = ?1",
        rusqlite::params![id, pct, status],
    )?;
    Ok(Json(serde_json::json!({ "completion_pct": pct, "status": status })))
}

#[derive(Debug, serde::Deserialize)]
struct CancelRequest {
    reason: String,
}

async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(input): Json<CancelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    let current_pct: i32 = conn.query_row(
        "SELECT completion_pct FROM jobs WHERE id = ?1",
        [&id], |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("Job {} not found", id)))?;

    let status = if current_pct > 0 { "partially_completed" } else { "cancelled" };
    conn.execute(
        "UPDATE jobs SET status = ?2, cancellation_reason = ?3, end_date = date('now'),
                updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![id, status, input.reason],
    )?;
    Ok(Json(serde_json::json!({ "status": status, "completion_pct": current_pct })))
}
