use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/fleet/vehicles", get(list_vehicles).post(add_vehicle))
        .route("/api/fleet/equipment", get(list_equipment).post(add_equipment))
        .route("/api/fleet/vehicles/:id/maintenance", post(log_vehicle_maintenance))
        .route("/api/fleet/equipment/:id/maintenance", post(log_equipment_maintenance))
        .route("/api/fleet/vehicles/:id/fuel", post(log_fuel))
        .route("/api/fleet/vehicles/:id/costs", get(vehicle_costs))
}

async fn list_vehicles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Vehicle>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, vehicle_type, year, make, model, license_plate, vin, status,
                monthly_payment, insurance_monthly, created_at
         FROM vehicles ORDER BY year DESC"
    )?;
    let vehicles = stmt.query_map([], |row| {
        Ok(Vehicle {
            id: row.get(0)?,
            vehicle_type: row.get(1)?,
            year: row.get(2)?,
            make: row.get(3)?,
            model: row.get(4)?,
            license_plate: row.get(5)?,
            vin: row.get(6)?,
            status: row.get(7)?,
            monthly_payment: get_decimal_opt(row, 8)?,
            insurance_monthly: get_decimal_opt(row, 9)?,
            created_at: row.get(10)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(vehicles))
}

async fn list_equipment(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Equipment>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, equipment_type, make, model, serial_number, owned_or_rented,
                daily_rate, status, created_at
         FROM equipment ORDER BY equipment_type"
    )?;
    let items = stmt.query_map([], |row| {
        Ok(Equipment {
            id: row.get(0)?,
            equipment_type: row.get(1)?,
            make: row.get(2)?,
            model: row.get(3)?,
            serial_number: row.get(4)?,
            owned_or_rented: row.get(5)?,
            daily_rate: get_decimal_opt(row, 6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

async fn add_vehicle(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateVehicle>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("veh");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO vehicles (id, vehicle_type, year, make, model, license_plate, vin, monthly_payment, insurance_monthly)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, input.vehicle_type, input.year, input.make, input.model,
            input.license_plate, input.vin,
            input.monthly_payment.map(|d| d.to_f64().unwrap_or(0.0)),
            input.insurance_monthly.map(|d| d.to_f64().unwrap_or(0.0))],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn add_equipment(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateEquipment>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("equip");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO equipment (id, equipment_type, make, model, serial_number, owned_or_rented, daily_rate)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, input.equipment_type, input.make, input.model,
            input.serial_number, input.owned_or_rented, input.daily_rate.map(|d| d.to_f64().unwrap_or(0.0))],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_vehicle_maintenance(
    State(state): State<Arc<AppState>>,
    Path(vehicle_id): Path<String>,
    Json(input): Json<CreateMaintenance>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("maint");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO maintenance_records (id, asset_type, asset_id, date, maintenance_type, cost, vendor, notes)
         VALUES (?1, 'vehicle', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, vehicle_id, input.date, input.maintenance_type, input.cost.to_f64().unwrap_or(0.0), input.vendor, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_equipment_maintenance(
    State(state): State<Arc<AppState>>,
    Path(equipment_id): Path<String>,
    Json(input): Json<CreateMaintenance>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("maint");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO maintenance_records (id, asset_type, asset_id, date, maintenance_type, cost, vendor, notes)
         VALUES (?1, 'equipment', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, equipment_id, input.date, input.maintenance_type, input.cost.to_f64().unwrap_or(0.0), input.vendor, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_fuel(
    State(state): State<Arc<AppState>>,
    Path(vehicle_id): Path<String>,
    Json(input): Json<CreateFuelLog>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("fuel");
    let total = input.gallons * input.cost_per_gallon;
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO fuel_logs (id, vehicle_id, date, gallons, cost_per_gallon, total_cost, odometer)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, vehicle_id, input.date, input.gallons.to_f64().unwrap_or(0.0), input.cost_per_gallon.to_f64().unwrap_or(0.0), total.to_f64().unwrap_or(0.0), input.odometer],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "total_cost": total })))
}

async fn vehicle_costs(
    State(state): State<Arc<AppState>>,
    Path(vehicle_id): Path<String>,
) -> Result<Json<AssetCostSummary>, AppError> {
    let conn = state.db.conn();
    let maintenance: f64 = conn.query_row(
        "SELECT COALESCE(SUM(cost), 0) FROM maintenance_records WHERE asset_id = ?1",
        [&vehicle_id], |r| r.get(0),
    )?;
    let fuel: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_cost), 0) FROM fuel_logs WHERE vehicle_id = ?1",
        [&vehicle_id], |r| r.get(0),
    )?;
    // Payments and insurance are monthly; sum over months since creation
    let (payment, insurance): (f64, f64) = conn.query_row(
        "SELECT COALESCE(monthly_payment, 0) * CAST((julianday('now') - julianday(created_at)) / 30 AS INTEGER),
                COALESCE(insurance_monthly, 0) * CAST((julianday('now') - julianday(created_at)) / 30 AS INTEGER)
         FROM vehicles WHERE id = ?1",
        [&vehicle_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| AppError::NotFound(format!("Vehicle {} not found", vehicle_id)))?;

    use rust_decimal::prelude::FromPrimitive;
    let m = rust_decimal::Decimal::from_f64(maintenance).unwrap_or_default();
    let f = rust_decimal::Decimal::from_f64(fuel).unwrap_or_default();
    let p = rust_decimal::Decimal::from_f64(payment).unwrap_or_default();
    let i = rust_decimal::Decimal::from_f64(insurance).unwrap_or_default();

    Ok(Json(AssetCostSummary {
        asset_id: vehicle_id,
        maintenance_total: m,
        fuel_total: f,
        payment_total: p,
        insurance_total: i,
        total_cost: m + f + p + i,
    }))
}
