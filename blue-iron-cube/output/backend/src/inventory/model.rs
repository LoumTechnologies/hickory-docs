use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub category: String,       // pipe, gravel, fabric, fitting, pump, restoration
    pub unit: String,            // feet, tons, sq_ft, each, bags
    pub quantity_on_hand: Decimal,
    pub reorder_point: Decimal,
    pub unit_cost: Decimal,      // last known cost per unit
    pub supplier: Option<String>,
    pub supplier_sku: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Purchase,    // stock received from supplier
    Usage,       // material used on a job
    Adjustment,  // manual correction (damage, count correction)
    Return,      // returned to supplier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransaction {
    pub id: String,
    pub item_id: String,
    pub transaction_type: String,
    pub quantity: Decimal,       // positive for in, negative for out
    pub unit_cost: Option<Decimal>,
    pub job_id: Option<String>,  // set when type=usage
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub name: String,
    pub category: String,
    pub unit: String,
    pub reorder_point: Decimal,
    pub unit_cost: Decimal,
    pub supplier: Option<String>,
    pub supplier_sku: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiveStock {
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UseStock {
    pub quantity: Decimal,
    pub job_id: String,
    pub notes: Option<String>,
}
