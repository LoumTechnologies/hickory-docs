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
        .route("/api/inventory", get(list_items).post(add_item))
        .route("/api/inventory/low-stock", get(low_stock))
        .route("/api/inventory/:id/receive", post(receive_stock))
        .route("/api/inventory/:id/use", post(use_stock))
}

async fn list_items(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, category, unit, quantity_on_hand, reorder_point,
                unit_cost, supplier, supplier_sku, notes, created_at, updated_at
         FROM inventory_items ORDER BY category, name"
    )?;
    let items = stmt.query_map([], |row| {
        Ok(InventoryItem {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            unit: row.get(3)?,
            quantity_on_hand: get_decimal(row, 4)?,
            reorder_point: get_decimal(row, 5)?,
            unit_cost: get_decimal(row, 6)?,
            supplier: row.get(7)?,
            supplier_sku: row.get(8)?,
            notes: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

async fn add_item(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateItem>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("inv");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO inventory_items (id, name, category, unit, quantity_on_hand, reorder_point, unit_cost, supplier, supplier_sku, notes)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, input.name, input.category, input.unit, input.reorder_point.to_f64().unwrap_or(0.0), input.unit_cost.to_f64().unwrap_or(0.0), input.supplier, input.supplier_sku, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn low_stock(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, category, unit, quantity_on_hand, reorder_point,
                unit_cost, supplier, supplier_sku, notes, created_at, updated_at
         FROM inventory_items
         WHERE quantity_on_hand <= reorder_point
         ORDER BY (quantity_on_hand - reorder_point)"
    )?;
    let items = stmt.query_map([], |row| {
        Ok(InventoryItem {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            unit: row.get(3)?,
            quantity_on_hand: get_decimal(row, 4)?,
            reorder_point: get_decimal(row, 5)?,
            unit_cost: get_decimal(row, 6)?,
            supplier: row.get(7)?,
            supplier_sku: row.get(8)?,
            notes: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

async fn receive_stock(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(input): Json<ReceiveStock>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("txn");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO inventory_transactions (id, item_id, transaction_type, quantity, unit_cost, notes)
         VALUES (?1, ?2, 'purchase', ?3, ?4, ?5)",
        rusqlite::params![id, item_id, input.quantity.to_f64().unwrap_or(0.0), input.unit_cost.to_f64().unwrap_or(0.0), input.notes],
    )?;
    conn.execute(
        "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand + ?2, unit_cost = ?3, updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![item_id, input.quantity.to_f64().unwrap_or(0.0), input.unit_cost.to_f64().unwrap_or(0.0)],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn use_stock(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(input): Json<UseStock>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("txn");
    let conn = state.db.conn();

    // Record the transaction
    conn.execute(
        "INSERT INTO inventory_transactions (id, item_id, transaction_type, quantity, job_id, notes)
         VALUES (?1, ?2, 'usage', ?3, ?4, ?5)",
        rusqlite::params![id, item_id, -input.quantity.to_f64().unwrap_or(0.0), input.job_id, input.notes],
    )?;
    conn.execute(
        "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand - ?2, updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![item_id, input.quantity.to_f64().unwrap_or(0.0)],
    )?;

    // Auto-create material line item on the job
    let (name, unit_cost, unit): (String, f64, String) = conn.query_row(
        "SELECT name, unit_cost, unit FROM inventory_items WHERE id = ?1",
        [&item_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let qty_f64 = input.quantity.to_f64().unwrap_or(0.0);
    let total: f64 = qty_f64 * unit_cost;
    let li_id = new_id("li");
    conn.execute(
        "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost)
         VALUES (?1, ?2, 'material', ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![li_id, input.job_id, name, qty_f64, unit, unit_cost, total],
    )?;

    Ok(Json(serde_json::json!({ "id": id, "material_cost": total })))
}
