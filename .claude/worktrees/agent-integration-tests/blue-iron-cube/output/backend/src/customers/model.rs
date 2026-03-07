use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A customer who has or may have french drain work done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    /// How did they find us? "referral", "google", "yard_sign", "repeat", etc.
    pub referral_source: Option<String>,
    /// If referred, who referred them? (customer ID or free text)
    pub referred_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomer {
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub referral_source: Option<String>,
    pub referred_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomer {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub referral_source: Option<String>,
    pub referred_by: Option<String>,
    pub notes: Option<String>,
}
