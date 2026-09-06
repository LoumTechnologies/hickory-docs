//! The domain the generator reads, and the only thing anybody edits here.

use describe_derive::Describe;

pub mod describe;

/// An order placed by a customer.
#[derive(Debug, Clone, Describe)]
pub struct Order {
    pub id: u64,
    pub sku: String,
    pub quantity: u32,
}

/// A person the orders belong to.
#[derive(Debug, Clone, Describe)]
pub struct Customer {
    pub name: String,
    pub email: Option<String>,
}

/// Not marked, and so not described — which is the half of the demonstration
/// that would pass silently if it were missing.
#[derive(Debug, Clone)]
pub struct Ledger {
    pub balance_cents: i64,
}
