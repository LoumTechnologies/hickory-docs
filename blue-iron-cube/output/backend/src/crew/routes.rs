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
        .route("/api/crew", get(list_crew).post(add_member))
        .route("/api/crew/:id/time-entries", get(list_time_entries).post(log_time))
}

async fn list_crew(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CrewMember>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, role, hourly_rate, phone, certifications, hire_date, active, created_at
         FROM crew_members ORDER BY name"
    )?;
    let members = stmt.query_map([], |row| {
        Ok(CrewMember {
            id: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            hourly_rate: get_decimal(row, 3)?,
            phone: row.get(4)?,
            certifications: row.get(5)?,
            hire_date: row.get(6)?,
            active: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(members))
}

async fn add_member(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateCrewMember>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("crew");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO crew_members (id, name, role, hourly_rate, phone, certifications, hire_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, input.name, input.role, input.hourly_rate.to_f64().unwrap_or(0.0), input.phone, input.certifications, input.hire_date],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_time(
    State(state): State<Arc<AppState>>,
    Path(crew_id): Path<String>,
    Json(input): Json<CreateTimeEntry>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("time");
    let overtime = input.hours_overtime.unwrap_or(Decimal::ZERO);
    let conn = state.db.conn();

    // Look up hourly rate to auto-create a labor line item on the job
    let hourly_rate: Decimal = conn.query_row(
        "SELECT hourly_rate FROM crew_members WHERE id = ?1",
        [&crew_id], |r| get_decimal(r, 0),
    ).map_err(|_| AppError::NotFound(format!("Crew member {} not found", crew_id)))?;

    let name: String = conn.query_row(
        "SELECT name FROM crew_members WHERE id = ?1",
        [&crew_id], |r| r.get(0),
    )?;

    conn.execute(
        "INSERT INTO time_entries (id, crew_member_id, job_id, date, hours_regular, hours_overtime, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, crew_id, input.job_id, input.date, input.hours_regular.to_f64().unwrap_or(0.0), overtime.to_f64().unwrap_or(0.0), input.notes],
    )?;

    // Auto-create labor line item on the job
    let labor_cost = (input.hours_regular * hourly_rate) + (overtime * hourly_rate * Decimal::from(150) / Decimal::from(100));
    let total_hours = input.hours_regular + overtime;
    let li_id = new_id("li");
    conn.execute(
        "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost)
         VALUES (?1, ?2, 'labor', ?3, ?4, 'hours', ?5, ?6)",
        rusqlite::params![li_id, input.job_id, format!("Labor: {}", name), total_hours.to_f64().unwrap_or(0.0), hourly_rate.to_f64().unwrap_or(0.0), labor_cost.to_f64().unwrap_or(0.0)],
    )?;

    Ok(Json(serde_json::json!({ "id": id, "labor_cost": labor_cost })))
}

async fn list_time_entries(
    State(state): State<Arc<AppState>>,
    Path(crew_id): Path<String>,
) -> Result<Json<Vec<TimeEntry>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, crew_member_id, job_id, date, hours_regular, hours_overtime, notes, created_at
         FROM time_entries WHERE crew_member_id = ?1 ORDER BY date DESC"
    )?;
    let entries = stmt.query_map([&crew_id], |row| {
        Ok(TimeEntry {
            id: row.get(0)?,
            crew_member_id: row.get(1)?,
            job_id: row.get(2)?,
            date: row.get(3)?,
            hours_regular: get_decimal(row, 4)?,
            hours_overtime: get_decimal(row, 5)?,
            notes: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(entries))
}
