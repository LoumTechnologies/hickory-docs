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
        .route("/api/payments", post(record_payment))
        .route("/api/payments/:id", get(get_payment))
        .route("/api/refunds", post(record_refund))
        .route("/api/jobs/:job_id/payments", get(job_payment_summary))
        .route("/api/payments/processing-fees/summary", get(processing_fee_summary))
}

async fn record_payment(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreatePayment>,
) -> Result<Json<Payment>, AppError> {
    let id = new_id("pay");
    let method = PaymentMethod::from_str(&input.method);
    let fee_recovered = input.fee_recovered.unwrap_or(false);

    // Determine processing fee
    let default_fee_pct: Decimal = if method.has_processing_fee() {
        // Read default from config, fallback to 3.0%
        Decimal::from(3)
    } else {
        Decimal::ZERO
    };
    let fee_pct = input.processing_fee_pct.unwrap_or(default_fee_pct);
    let fee_amount = input.amount * fee_pct / Decimal::from(100);

    let conn = state.db.conn();

    // Verify job exists
    let job_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM jobs WHERE id = ?1", [&input.job_id], |r| r.get(0),
    )?;
    if !job_exists {
        return Err(AppError::NotFound(format!("Job {} not found", input.job_id)));
    }

    conn.execute(
        "INSERT INTO payments (id, job_id, date, amount, method, processing_fee_pct,
                               processing_fee_amount, fee_recovered, reference, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            id, input.job_id, input.date, input.amount.to_f64().unwrap_or(0.0), method.as_str(),
            fee_pct.to_f64().unwrap_or(0.0), fee_amount.to_f64().unwrap_or(0.0),
            fee_recovered, input.reference, input.notes,
        ],
    )?;

    // If fee is absorbed (not recovered), create a processing_fee line item on the job
    if fee_amount > Decimal::ZERO && !fee_recovered {
        let li_id = new_id("li");
        conn.execute(
            "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost, notes)
             VALUES (?1, ?2, 'processing_fee', ?3, 1, 'fee', ?4, ?4, ?5)",
            rusqlite::params![
                li_id, input.job_id,
                format!("CC processing fee ({}%)", fee_pct),
                fee_amount.to_f64().unwrap_or(0.0),
                format!("Payment {}", id),
            ],
        )?;
    }

    // Update job's actual_revenue (sum of all payments minus refunds)
    conn.execute(
        "UPDATE jobs SET actual_revenue = (
            SELECT COALESCE(SUM(amount), 0) FROM payments WHERE job_id = ?1
         ) - (
            SELECT COALESCE(SUM(amount), 0) FROM refunds WHERE job_id = ?1
         ), updated_at = datetime('now')
         WHERE id = ?1",
        [&input.job_id],
    )?;

    let payment = Payment {
        id,
        job_id: input.job_id,
        date: input.date,
        amount: input.amount,
        method: method.as_str().to_string(),
        processing_fee_pct: fee_pct,
        processing_fee_amount: fee_amount,
        fee_recovered,
        reference: input.reference,
        notes: input.notes,
        created_at: chrono::Utc::now().naive_utc(),
    };

    Ok(Json(payment))
}

async fn get_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Payment>, AppError> {
    let conn = state.db.conn();
    let payment = conn.query_row(
        "SELECT id, job_id, date, amount, method, processing_fee_pct,
                processing_fee_amount, fee_recovered, reference, notes, created_at
         FROM payments WHERE id = ?1",
        [&id],
        |row| Ok(Payment {
            id: row.get(0)?,
            job_id: row.get(1)?,
            date: row.get(2)?,
            amount: get_decimal(row, 3)?,
            method: row.get(4)?,
            processing_fee_pct: get_decimal(row, 5)?,
            processing_fee_amount: get_decimal(row, 6)?,
            fee_recovered: row.get(7)?,
            reference: row.get(8)?,
            notes: row.get(9)?,
            created_at: row.get(10)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("Payment {} not found", id)))?;
    Ok(Json(payment))
}

async fn record_refund(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateRefund>,
) -> Result<Json<Refund>, AppError> {
    let id = new_id("ref");
    let method = input.method.as_deref().unwrap_or("original_method");
    let processor_refunded = input.processor_refunded_fee.unwrap_or(false);
    let conn = state.db.conn();

    conn.execute(
        "INSERT INTO refunds (id, job_id, payment_id, date, amount, reason, method,
                              processor_refunded_fee, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            id, input.job_id, input.payment_id, input.date, input.amount.to_f64().unwrap_or(0.0),
            input.reason, method, processor_refunded, input.notes,
        ],
    )?;

    // Update job's actual_revenue
    conn.execute(
        "UPDATE jobs SET actual_revenue = (
            SELECT COALESCE(SUM(amount), 0) FROM payments WHERE job_id = ?1
         ) - (
            SELECT COALESCE(SUM(amount), 0) FROM refunds WHERE job_id = ?1
         ), updated_at = datetime('now')
         WHERE id = ?1",
        [&input.job_id],
    )?;

    let refund = Refund {
        id,
        job_id: input.job_id,
        payment_id: input.payment_id,
        date: input.date,
        amount: input.amount,
        reason: input.reason,
        method: method.to_string(),
        processor_refunded_fee: processor_refunded,
        notes: input.notes,
        created_at: chrono::Utc::now().naive_utc(),
    };

    Ok(Json(refund))
}

async fn job_payment_summary(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<JobPaymentSummary>, AppError> {
    let conn = state.db.conn();

    let quoted: Decimal = conn.query_row(
        "SELECT COALESCE(quoted_price, 0) FROM jobs WHERE id = ?1",
        [&job_id], |r| get_decimal(r, 0),
    ).map_err(|_| AppError::NotFound(format!("Job {} not found", job_id)))?;

    // Fetch payments
    let mut stmt = conn.prepare(
        "SELECT id, job_id, date, amount, method, processing_fee_pct,
                processing_fee_amount, fee_recovered, reference, notes, created_at
         FROM payments WHERE job_id = ?1 ORDER BY date"
    )?;
    let payments: Vec<Payment> = stmt.query_map([&job_id], |row| {
        Ok(Payment {
            id: row.get(0)?,
            job_id: row.get(1)?,
            date: row.get(2)?,
            amount: get_decimal(row, 3)?,
            method: row.get(4)?,
            processing_fee_pct: get_decimal(row, 5)?,
            processing_fee_amount: get_decimal(row, 6)?,
            fee_recovered: row.get(7)?,
            reference: row.get(8)?,
            notes: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // Fetch refunds
    let mut stmt = conn.prepare(
        "SELECT id, job_id, payment_id, date, amount, reason, method,
                processor_refunded_fee, notes, created_at
         FROM refunds WHERE job_id = ?1 ORDER BY date"
    )?;
    let refunds: Vec<Refund> = stmt.query_map([&job_id], |row| {
        Ok(Refund {
            id: row.get(0)?,
            job_id: row.get(1)?,
            payment_id: row.get(2)?,
            date: row.get(3)?,
            amount: get_decimal(row, 4)?,
            reason: row.get(5)?,
            method: row.get(6)?,
            processor_refunded_fee: row.get(7)?,
            notes: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    let total_payments: Decimal = payments.iter().map(|p| p.amount).sum();
    let total_refunds: Decimal = refunds.iter().map(|r| r.amount).sum();
    let net = total_payments - total_refunds;
    let balance = quoted - net;
    let total_fees: Decimal = payments.iter().map(|p| p.processing_fee_amount).sum();
    let fees_recovered: Decimal = payments.iter()
        .filter(|p| p.fee_recovered)
        .map(|p| p.processing_fee_amount).sum();
    let fees_absorbed = total_fees - fees_recovered;

    Ok(Json(JobPaymentSummary {
        job_id,
        quoted_price: quoted,
        total_payments,
        total_refunds,
        net_collected: net,
        balance_due: balance,
        total_processing_fees: total_fees,
        processing_fees_recovered: fees_recovered,
        processing_fees_absorbed: fees_absorbed,
        payment_count: payments.len() as i64,
        is_paid_in_full: balance <= Decimal::ZERO,
        payments,
        refunds,
    }))
}

/// Summary of processing fees across all jobs — how much has been lost to processors.
async fn processing_fee_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();

    let total_fees: f64 = conn.query_row(
        "SELECT COALESCE(SUM(processing_fee_amount), 0) FROM payments", [], |r| r.get(0),
    )?;
    let fees_recovered: f64 = conn.query_row(
        "SELECT COALESCE(SUM(processing_fee_amount), 0) FROM payments WHERE fee_recovered = 1",
        [], |r| r.get(0),
    )?;
    let fees_absorbed = total_fees - fees_recovered;
    let total_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM payments", [], |r| r.get(0),
    )?;
    let effective_fee_pct = if total_revenue > 0.0 { fees_absorbed / total_revenue * 100.0 } else { 0.0 };

    let card_payments: i64 = conn.query_row(
        "SELECT COUNT(*) FROM payments WHERE method IN ('credit_card', 'debit_card')",
        [], |r| r.get(0),
    )?;
    let non_card_payments: i64 = conn.query_row(
        "SELECT COUNT(*) FROM payments WHERE method NOT IN ('credit_card', 'debit_card')",
        [], |r| r.get(0),
    )?;

    Ok(Json(serde_json::json!({
        "total_processing_fees": total_fees,
        "fees_recovered_from_customers": fees_recovered,
        "fees_absorbed_by_business": fees_absorbed,
        "total_revenue_collected": total_revenue,
        "effective_fee_rate_pct": effective_fee_pct,
        "card_payment_count": card_payments,
        "non_card_payment_count": non_card_payments,
        "savings_from_non_card": format!(
            "Non-card payments saved approximately ${:.2} in processing fees",
            non_card_payments as f64 * (total_revenue / (card_payments + non_card_payments).max(1) as f64) * 0.03
        ),
    })))
}
