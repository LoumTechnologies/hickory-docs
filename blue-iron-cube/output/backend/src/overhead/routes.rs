use std::sync::Arc;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/overhead/categories", get(list_categories).post(add_category))
        .route("/api/overhead/entries", post(add_entry))
        .route("/api/overhead/recovery-rate", get(recovery_rate))
        .route("/api/overhead/allocate", post(allocate))
}

async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<OverheadCategory>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, frequency, monthly_budget, notes, created_at
         FROM overhead_categories ORDER BY name"
    )?;
    let cats = stmt.query_map([], |row| {
        Ok(OverheadCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            frequency: row.get(2)?,
            monthly_budget: get_decimal(row, 3)?,
            notes: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(cats))
}

async fn add_category(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateCategory>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("ohcat");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO overhead_categories (id, name, frequency, monthly_budget, notes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, input.name, input.frequency, input.monthly_budget.to_f64().unwrap_or(0.0), input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn add_entry(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateEntry>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("oh");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO overhead_entries (id, category_id, date, amount, description)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, input.category_id, input.date, input.amount.to_f64().unwrap_or(0.0), input.description],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn recovery_rate(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RecoveryRateReport>, AppError> {
    let conn = state.db.conn();

    let total_incurred: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM overhead_entries", [], |r| get_decimal(r, 0),
    )?;
    let total_allocated: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(total_cost), 0) FROM job_line_items WHERE category = 'overhead'",
        [], |r| get_decimal(r, 0),
    )?;
    let monthly_budget: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(monthly_budget), 0) FROM overhead_categories", [], |r| get_decimal(r, 0),
    )?;
    let job_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT job_id) FROM job_line_items WHERE category = 'overhead'",
        [], |r| r.get(0),
    )?;

    let recovery_pct = if total_incurred > Decimal::ZERO {
        total_allocated / total_incurred * Decimal::from(100)
    } else {
        Decimal::ZERO
    };
    let avg_per_job = if job_count > 0 {
        total_allocated / Decimal::from(job_count)
    } else {
        Decimal::ZERO
    };

    Ok(Json(RecoveryRateReport {
        total_overhead_incurred: total_incurred,
        total_overhead_allocated: total_allocated,
        recovery_rate_pct: recovery_pct,
        monthly_budget,
        avg_overhead_per_job: avg_per_job,
    }))
}

async fn allocate(
    State(state): State<Arc<AppState>>,
    Json(input): Json<AllocateRequest>,
) -> Result<Json<AllocationResult>, AppError> {
    let conn = state.db.conn();

    // Sum overhead entries in the period
    let total_overhead: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM overhead_entries WHERE date BETWEEN ?1 AND ?2",
        rusqlite::params![input.period_start, input.period_end], |r| get_decimal(r, 0),
    )?;

    // Find completed jobs in the period
    let mut stmt = conn.prepare(
        "SELECT id FROM jobs WHERE status IN ('completed', 'invoiced') AND end_date BETWEEN ?1 AND ?2"
    )?;
    let job_ids: Vec<String> = stmt.query_map(
        rusqlite::params![input.period_start, input.period_end],
        |row| row.get(0),
    )?.collect::<Result<Vec<_>, _>>()?;

    let job_count = job_ids.len() as i64;
    if job_count == 0 {
        return Ok(Json(AllocationResult {
            period_start: input.period_start,
            period_end: input.period_end,
            total_overhead,
            jobs_in_period: 0,
            allocation_method: input.method.clone(),
            allocations: vec![],
            recovery_rate: Decimal::ZERO,
        }));
    }

    // Simple equal-split allocation (other methods follow same pattern)
    let per_job = total_overhead / Decimal::from(job_count);
    let mut allocations = Vec::new();

    for job_id in &job_ids {
        let li_id = new_id("li");
        conn.execute(
            "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost)
             VALUES (?1, ?2, 'overhead', 'Overhead allocation', 1, 'allocation', ?3, ?3)",
            rusqlite::params![li_id, job_id, per_job.to_f64().unwrap_or(0.0)],
        )?;
        allocations.push(JobAllocation {
            job_id: job_id.clone(),
            allocated_amount: per_job,
        });
    }

    let allocated = per_job * Decimal::from(job_count);
    let recovery = if total_overhead > Decimal::ZERO {
        allocated / total_overhead * Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    Ok(Json(AllocationResult {
        period_start: input.period_start,
        period_end: input.period_end,
        total_overhead,
        jobs_in_period: job_count,
        allocation_method: input.method,
        allocations,
        recovery_rate: recovery,
    }))
}
