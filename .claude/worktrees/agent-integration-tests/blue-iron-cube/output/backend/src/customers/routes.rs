use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use crate::{AppState, error::AppError, types::new_id};
use super::model::{CreateCustomer, Customer, UpdateCustomer};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/customers", get(list_customers).post(create_customer))
        .route("/api/customers/:id", get(get_customer).put(update_customer))
}

async fn list_customers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Customer>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, phone, email, address, city, state, zip,
                referral_source, referred_by, notes, created_at, updated_at
         FROM customers ORDER BY name"
    )?;
    let customers = stmt.query_map([], |row| {
        Ok(Customer {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            address: row.get(4)?,
            city: row.get(5)?,
            state: row.get(6)?,
            zip: row.get(7)?,
            referral_source: row.get(8)?,
            referred_by: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(customers))
}

async fn create_customer(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateCustomer>,
) -> Result<Json<Customer>, AppError> {
    let id = new_id("cust");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO customers (id, name, phone, email, address, city, state, zip,
                                referral_source, referred_by, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            id, input.name, input.phone, input.email,
            input.address, input.city, input.state, input.zip,
            input.referral_source, input.referred_by, input.notes,
        ],
    )?;
    let customer = get_customer_by_id(&conn, &id)?;
    Ok(Json(customer))
}

async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Customer>, AppError> {
    let conn = state.db.conn();
    let customer = get_customer_by_id(&conn, &id)?;
    Ok(Json(customer))
}

async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCustomer>,
) -> Result<Json<Customer>, AppError> {
    let conn = state.db.conn();
    // Verify exists
    get_customer_by_id(&conn, &id)?;
    conn.execute(
        "UPDATE customers SET
            name = COALESCE(?2, name),
            phone = COALESCE(?3, phone),
            email = COALESCE(?4, email),
            address = COALESCE(?5, address),
            city = COALESCE(?6, city),
            state = COALESCE(?7, state),
            zip = COALESCE(?8, zip),
            referral_source = COALESCE(?9, referral_source),
            referred_by = COALESCE(?10, referred_by),
            notes = COALESCE(?11, notes),
            updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![
            id, input.name, input.phone, input.email,
            input.address, input.city, input.state, input.zip,
            input.referral_source, input.referred_by, input.notes,
        ],
    )?;
    let customer = get_customer_by_id(&conn, &id)?;
    Ok(Json(customer))
}

fn get_customer_by_id(conn: &rusqlite::Connection, id: &str) -> Result<Customer, AppError> {
    conn.query_row(
        "SELECT id, name, phone, email, address, city, state, zip,
                referral_source, referred_by, notes, created_at, updated_at
         FROM customers WHERE id = ?1",
        [id],
        |row| Ok(Customer {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            address: row.get(4)?,
            city: row.get(5)?,
            state: row.get(6)?,
            zip: row.get(7)?,
            referral_source: row.get(8)?,
            referred_by: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("Customer {} not found", id)))
}
