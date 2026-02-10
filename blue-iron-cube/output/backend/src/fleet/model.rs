use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A company vehicle (truck, trailer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub vehicle_type: String,     // truck, trailer, van
    pub year: i32,
    pub make: String,
    pub model: String,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub status: String,           // active, maintenance, retired
    pub monthly_payment: Option<Decimal>,
    pub insurance_monthly: Option<Decimal>,
    pub created_at: NaiveDateTime,
}

/// A piece of equipment (excavator, compactor, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub id: String,
    pub equipment_type: String,   // mini_excavator, plate_compactor, trencher, sod_cutter, pump, laser_level
    pub make: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub owned_or_rented: String,  // owned, rented
    pub daily_rate: Option<Decimal>, // cost per day if rented, or depreciation rate if owned
    pub status: String,           // active, maintenance, retired
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub id: String,
    pub asset_type: String,       // vehicle or equipment
    pub asset_id: String,
    pub date: NaiveDate,
    pub maintenance_type: String, // oil_change, tires, tracks, hydraulic, repair, inspection
    pub cost: Decimal,
    pub vendor: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelLog {
    pub id: String,
    pub vehicle_id: String,
    pub date: NaiveDate,
    pub gallons: Decimal,
    pub cost_per_gallon: Decimal,
    pub total_cost: Decimal,
    pub odometer: Option<i64>,
    pub created_at: NaiveDateTime,
}

/// Total cost of ownership summary for a fleet asset.
#[derive(Debug, Serialize)]
pub struct AssetCostSummary {
    pub asset_id: String,
    pub maintenance_total: Decimal,
    pub fuel_total: Decimal,
    pub payment_total: Decimal,
    pub insurance_total: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateVehicle {
    pub vehicle_type: String,
    pub year: i32,
    pub make: String,
    pub model: String,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub monthly_payment: Option<Decimal>,
    pub insurance_monthly: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEquipment {
    pub equipment_type: String,
    pub make: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub owned_or_rented: String,
    pub daily_rate: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenance {
    pub date: NaiveDate,
    pub maintenance_type: String,
    pub cost: Decimal,
    pub vendor: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFuelLog {
    pub date: NaiveDate,
    pub gallons: Decimal,
    pub cost_per_gallon: Decimal,
    pub odometer: Option<i64>,
}
