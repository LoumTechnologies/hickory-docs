use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use crate::{AppState, error::AppError, types::new_id};
use super::model::*;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/sops", get(list_procedures).post(create_procedure))
        .route("/api/sops/:id", get(get_procedure).put(update_procedure))
}

async fn list_procedures(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Procedure>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, category, version, description, estimated_duration_minutes,
                required_roles, required_equipment, required_materials, safety_notes,
                active, created_at, updated_at
         FROM procedures WHERE active = 1 ORDER BY category, title"
    )?;
    let procs = stmt.query_map([], |row| {
        Ok(Procedure {
            id: row.get(0)?,
            title: row.get(1)?,
            category: row.get(2)?,
            version: row.get(3)?,
            description: row.get(4)?,
            estimated_duration_minutes: row.get(5)?,
            required_roles: row.get(6)?,
            required_equipment: row.get(7)?,
            required_materials: row.get(8)?,
            safety_notes: row.get(9)?,
            active: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(procs))
}

async fn create_procedure(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateProcedure>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("sop");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO procedures (id, title, category, version, description, estimated_duration_minutes,
                                 required_roles, required_equipment, required_materials, safety_notes)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            id, input.title, input.category, input.description,
            input.estimated_duration_minutes, input.required_roles,
            input.required_equipment, input.required_materials, input.safety_notes,
        ],
    )?;

    for (i, step) in input.steps.iter().enumerate() {
        let step_id = new_id("step");
        conn.execute(
            "INSERT INTO procedure_steps (id, procedure_id, step_number, instruction, is_checkpoint, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![step_id, id, (i + 1) as i32, step.instruction, step.is_checkpoint.unwrap_or(false), step.notes],
        )?;
    }

    Ok(Json(serde_json::json!({ "id": id, "version": 1, "steps_created": input.steps.len() })))
}

async fn get_procedure(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ProcedureWithSteps>, AppError> {
    let conn = state.db.conn();
    let procedure = conn.query_row(
        "SELECT id, title, category, version, description, estimated_duration_minutes,
                required_roles, required_equipment, required_materials, safety_notes,
                active, created_at, updated_at
         FROM procedures WHERE id = ?1",
        [&id],
        |row| Ok(Procedure {
            id: row.get(0)?,
            title: row.get(1)?,
            category: row.get(2)?,
            version: row.get(3)?,
            description: row.get(4)?,
            estimated_duration_minutes: row.get(5)?,
            required_roles: row.get(6)?,
            required_equipment: row.get(7)?,
            required_materials: row.get(8)?,
            safety_notes: row.get(9)?,
            active: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("Procedure {} not found", id)))?;

    let mut stmt = conn.prepare(
        "SELECT id, procedure_id, step_number, instruction, is_checkpoint, notes
         FROM procedure_steps WHERE procedure_id = ?1 ORDER BY step_number"
    )?;
    let steps = stmt.query_map([&id], |row| {
        Ok(ProcedureStep {
            id: row.get(0)?,
            procedure_id: row.get(1)?,
            step_number: row.get(2)?,
            instruction: row.get(3)?,
            is_checkpoint: row.get(4)?,
            notes: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ProcedureWithSteps { procedure, steps }))
}

async fn update_procedure(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(input): Json<CreateProcedure>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();

    // Bump version, replace steps
    let current_version: i32 = conn.query_row(
        "SELECT version FROM procedures WHERE id = ?1",
        [&id], |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("Procedure {} not found", id)))?;

    let new_version = current_version + 1;
    conn.execute(
        "UPDATE procedures SET title=?2, category=?3, version=?4, description=?5,
                estimated_duration_minutes=?6, required_roles=?7, required_equipment=?8,
                required_materials=?9, safety_notes=?10, updated_at=datetime('now')
         WHERE id = ?1",
        rusqlite::params![
            id, input.title, input.category, new_version, input.description,
            input.estimated_duration_minutes, input.required_roles,
            input.required_equipment, input.required_materials, input.safety_notes,
        ],
    )?;

    // Replace steps
    conn.execute("DELETE FROM procedure_steps WHERE procedure_id = ?1", [&id])?;
    for (i, step) in input.steps.iter().enumerate() {
        let step_id = new_id("step");
        conn.execute(
            "INSERT INTO procedure_steps (id, procedure_id, step_number, instruction, is_checkpoint, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![step_id, id, (i + 1) as i32, step.instruction, step.is_checkpoint.unwrap_or(false), step.notes],
        )?;
    }

    Ok(Json(serde_json::json!({ "id": id, "version": new_version })))
}
