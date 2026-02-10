use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Check,
    CreditCard,
    DebitCard,
    Ach,
    Financing,
    Other,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Check => "check",
            Self::CreditCard => "credit_card",
            Self::DebitCard => "debit_card",
            Self::Ach => "ach",
            Self::Financing => "financing",
            Self::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "cash" => Self::Cash,
            "check" => Self::Check,
            "credit_card" => Self::CreditCard,
            "debit_card" => Self::DebitCard,
            "ach" => Self::Ach,
            "financing" => Self::Financing,
            _ => Self::Other,
        }
    }

    /// Does this method incur a processing fee?
    pub fn has_processing_fee(&self) -> bool {
        matches!(self, Self::CreditCard | Self::DebitCard)
    }
}

/// A payment received against a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub job_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub method: String,
    /// The processing fee percentage charged by the processor (e.g., 2.9)
    pub processing_fee_pct: Decimal,
    /// Computed: amount * fee_pct / 100
    pub processing_fee_amount: Decimal,
    /// Was the fee passed to the customer (surcharge) or absorbed by the business?
    pub fee_recovered: bool,
    /// check number, last-4 of card, transaction ref, etc.
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

/// A refund issued against a job (and optionally a specific payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refund {
    pub id: String,
    pub job_id: String,
    pub payment_id: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub reason: String,
    pub method: String,
    /// Was the original processing fee refunded by the processor?
    pub processor_refunded_fee: bool,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Payment summary for a job.
#[derive(Debug, Serialize)]
pub struct JobPaymentSummary {
    pub job_id: String,
    pub quoted_price: Decimal,
    pub total_payments: Decimal,
    pub total_refunds: Decimal,
    pub net_collected: Decimal,
    pub balance_due: Decimal,
    pub total_processing_fees: Decimal,
    pub processing_fees_recovered: Decimal,
    pub processing_fees_absorbed: Decimal,
    pub payment_count: i64,
    pub is_paid_in_full: bool,
    pub payments: Vec<Payment>,
    pub refunds: Vec<Refund>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayment {
    pub job_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub method: String,
    /// Override fee pct. If omitted, uses default for the payment method.
    pub processing_fee_pct: Option<Decimal>,
    /// If true, the fee is added on top (customer pays amount + fee).
    /// If false (default), fee is absorbed by the business.
    pub fee_recovered: Option<bool>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRefund {
    pub job_id: String,
    pub payment_id: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub reason: String,
    pub method: Option<String>,
    pub processor_refunded_fee: Option<bool>,
    pub notes: Option<String>,
}
