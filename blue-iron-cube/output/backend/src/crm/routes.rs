use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Leads
        .route("/api/crm/leads", get(list_leads).post(create_lead))
        .route("/api/crm/leads/:id", get(get_lead).put(update_lead))
        .route("/api/crm/leads/pipeline", get(pipeline_summary))
        // Interactions
        .route("/api/crm/interactions", post(log_interaction))
        .route("/api/crm/customers/:id/interactions", get(customer_interactions))
        // Follow-ups
        .route("/api/crm/follow-ups", get(pending_follow_ups).post(create_follow_up))
        .route("/api/crm/follow-ups/:id/complete", post(complete_follow_up))
        // Customer value
        .route("/api/crm/customers/:id/lifetime-value", get(customer_lifetime_value))
        .route("/api/crm/conversion-rate", get(conversion_rate))
}

#[derive(Deserialize)]
struct LeadFilter {
    status: Option<String>,
}

async fn list_leads(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<LeadFilter>,
) -> Result<Json<Vec<Lead>>, AppError> {
    let conn = state.db.conn();
    let leads = if let Some(status) = &filter.status {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, status, source, estimated_value, job_id,
                    assigned_to, lost_reason, lost_to_competitor, created_at, updated_at
             FROM leads WHERE status = ?1 ORDER BY updated_at DESC"
        )?;
        stmt.query_map([status], map_lead)?.collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, status, source, estimated_value, job_id,
                    assigned_to, lost_reason, lost_to_competitor, created_at, updated_at
             FROM leads ORDER BY updated_at DESC"
        )?;
        stmt.query_map([], map_lead)?.collect::<Result<Vec<_>, _>>()?
    };
    Ok(Json(leads))
}

fn map_lead(row: &rusqlite::Row) -> rusqlite::Result<Lead> {
    Ok(Lead {
        id: row.get(0)?,
        customer_id: row.get(1)?,
        status: row.get(2)?,
        source: row.get(3)?,
        estimated_value: get_decimal_opt(row, 4)?,
        job_id: row.get(5)?,
        assigned_to: row.get(6)?,
        lost_reason: row.get(7)?,
        lost_to_competitor: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

async fn create_lead(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateLead>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("lead");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO leads (id, customer_id, status, source, estimated_value, assigned_to)
         VALUES (?1, ?2, 'new_lead', ?3, ?4, ?5)",
        rusqlite::params![id, input.customer_id, input.source, input.estimated_value.map(|d| d.to_f64().unwrap_or(0.0)), input.assigned_to],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "status": "new_lead" })))
}

async fn get_lead(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    let lead = conn.query_row(
        "SELECT l.id, l.customer_id, c.name, l.status, l.source, l.estimated_value,
                l.job_id, l.assigned_to, l.lost_reason, l.lost_to_competitor,
                l.created_at, l.updated_at
         FROM leads l JOIN customers c ON l.customer_id = c.id
         WHERE l.id = ?1",
        [&id],
        |row| Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "customer_id": row.get::<_, String>(1)?,
            "customer_name": row.get::<_, String>(2)?,
            "status": row.get::<_, String>(3)?,
            "source": row.get::<_, String>(4)?,
            "estimated_value": row.get::<_, Option<f64>>(5)?,
            "job_id": row.get::<_, Option<String>>(6)?,
            "assigned_to": row.get::<_, Option<String>>(7)?,
            "lost_reason": row.get::<_, Option<String>>(8)?,
            "lost_to_competitor": row.get::<_, Option<String>>(9)?,
            "created_at": row.get::<_, String>(10)?,
            "updated_at": row.get::<_, String>(11)?,
        })),
    ).map_err(|_| AppError::NotFound(format!("Lead {} not found", id)))?;
    Ok(Json(lead))
}

async fn update_lead(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateLead>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    conn.execute(
        "UPDATE leads SET
            status = COALESCE(?2, status),
            estimated_value = COALESCE(?3, estimated_value),
            job_id = COALESCE(?4, job_id),
            assigned_to = COALESCE(?5, assigned_to),
            lost_reason = COALESCE(?6, lost_reason),
            lost_to_competitor = COALESCE(?7, lost_to_competitor),
            updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![
            id, input.status, input.estimated_value.map(|d| d.to_f64().unwrap_or(0.0)), input.job_id,
            input.assigned_to, input.lost_reason, input.lost_to_competitor,
        ],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

/// Pipeline summary: count of leads per stage.
async fn pipeline_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT status, COUNT(*), COALESCE(SUM(estimated_value), 0)
         FROM leads GROUP BY status ORDER BY
         CASE status
            WHEN 'new_lead' THEN 1
            WHEN 'contacted' THEN 2
            WHEN 'site_visit_scheduled' THEN 3
            WHEN 'site_visit_completed' THEN 4
            WHEN 'estimate_sent' THEN 5
            WHEN 'negotiating' THEN 6
            WHEN 'won' THEN 7
            WHEN 'lost' THEN 8
         END"
    )?;
    let stages: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "status": row.get::<_, String>(0)?,
            "count": row.get::<_, i64>(1)?,
            "total_value": row.get::<_, f64>(2)?,
        }))
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(serde_json::json!({ "stages": stages })))
}

async fn log_interaction(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateInteraction>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("ix");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO interactions (id, customer_id, lead_id, date, interaction_type,
                                   direction, summary, duration_minutes)
         VALUES (?1, ?2, ?3, datetime('now'), ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id, input.customer_id, input.lead_id, input.interaction_type,
            input.direction, input.summary, input.duration_minutes,
        ],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn customer_interactions(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<Interaction>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, customer_id, lead_id, date, interaction_type, direction,
                summary, duration_minutes, created_at
         FROM interactions WHERE customer_id = ?1 ORDER BY date DESC"
    )?;
    let interactions = stmt.query_map([&customer_id], |row| {
        Ok(Interaction {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            lead_id: row.get(2)?,
            date: row.get(3)?,
            interaction_type: row.get(4)?,
            direction: row.get(5)?,
            summary: row.get(6)?,
            duration_minutes: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(interactions))
}

async fn pending_follow_ups(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT f.id, f.customer_id, c.name, c.phone, f.lead_id, f.due_date,
                f.action, f.notes, f.created_at
         FROM follow_ups f JOIN customers c ON f.customer_id = c.id
         WHERE f.completed = 0
         ORDER BY f.due_date"
    )?;
    let follow_ups = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "customer_id": row.get::<_, String>(1)?,
            "customer_name": row.get::<_, String>(2)?,
            "customer_phone": row.get::<_, String>(3)?,
            "lead_id": row.get::<_, Option<String>>(4)?,
            "due_date": row.get::<_, String>(5)?,
            "action": row.get::<_, String>(6)?,
            "notes": row.get::<_, Option<String>>(7)?,
        }))
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(Json(follow_ups))
}

async fn create_follow_up(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateFollowUp>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = new_id("fu");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO follow_ups (id, customer_id, lead_id, due_date, action, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, input.customer_id, input.lead_id, input.due_date, input.action, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn complete_follow_up(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    conn.execute(
        "UPDATE follow_ups SET completed = 1, completed_at = datetime('now') WHERE id = ?1",
        [&id],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "completed": true })))
}

async fn customer_lifetime_value(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<String>,
) -> Result<Json<CustomerLifetimeValue>, AppError> {
    let conn = state.db.conn();

    let name: String = conn.query_row(
        "SELECT name FROM customers WHERE id = ?1",
        [&customer_id], |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("Customer {} not found", customer_id)))?;

    let total_jobs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM jobs WHERE customer_id = ?1", [&customer_id], |r| r.get(0),
    )?;
    let total_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(actual_revenue, quoted_price, 0)), 0) FROM jobs WHERE customer_id = ?1",
        [&customer_id], |r| r.get(0),
    )?;
    let first_job: Option<String> = conn.query_row(
        "SELECT MIN(created_at) FROM jobs WHERE customer_id = ?1",
        [&customer_id], |r| r.get(0),
    ).ok();
    let last_job: Option<String> = conn.query_row(
        "SELECT MAX(created_at) FROM jobs WHERE customer_id = ?1",
        [&customer_id], |r| r.get(0),
    ).ok();

    // Count customers who listed this customer as their referrer
    let referral_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM customers WHERE referred_by = ?1", [&customer_id], |r| r.get(0),
    )?;
    let referral_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(j.actual_revenue, j.quoted_price, 0)), 0)
         FROM jobs j JOIN customers c ON j.customer_id = c.id
         WHERE c.referred_by = ?1",
        [&customer_id], |r| r.get(0),
    )?;

    use rust_decimal::prelude::FromPrimitive;
    let avg = if total_jobs > 0 { total_revenue / total_jobs as f64 } else { 0.0 };

    Ok(Json(CustomerLifetimeValue {
        customer_id,
        customer_name: name,
        total_jobs,
        total_revenue: rust_decimal::Decimal::from_f64(total_revenue).unwrap_or_default(),
        total_referrals: referral_count,
        referral_revenue: rust_decimal::Decimal::from_f64(referral_revenue).unwrap_or_default(),
        first_job_date: first_job,
        last_job_date: last_job,
        avg_job_value: rust_decimal::Decimal::from_f64(avg).unwrap_or_default(),
    }))
}

/// Lead conversion rate: won / (won + lost).
async fn conversion_rate(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = state.db.conn();
    let won: i64 = conn.query_row(
        "SELECT COUNT(*) FROM leads WHERE status = 'won'", [], |r| r.get(0),
    )?;
    let lost: i64 = conn.query_row(
        "SELECT COUNT(*) FROM leads WHERE status = 'lost'", [], |r| r.get(0),
    )?;
    let total_closed = won + lost;
    let rate = if total_closed > 0 { won as f64 / total_closed as f64 * 100.0 } else { 0.0 };

    // Conversion by source
    let mut stmt = conn.prepare(
        "SELECT source,
                SUM(CASE WHEN status = 'won' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'lost' THEN 1 ELSE 0 END),
                COUNT(*)
         FROM leads GROUP BY source"
    )?;
    let by_source: Vec<serde_json::Value> = stmt.query_map([], |row| {
        let s_won: i64 = row.get(1)?;
        let s_lost: i64 = row.get(2)?;
        let s_total: i64 = row.get(3)?;
        let s_closed = s_won + s_lost;
        Ok(serde_json::json!({
            "source": row.get::<_, String>(0)?,
            "won": s_won,
            "lost": s_lost,
            "open": s_total - s_closed,
            "conversion_rate_pct": if s_closed > 0 { s_won as f64 / s_closed as f64 * 100.0 } else { 0.0 },
        }))
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(Json(serde_json::json!({
        "total_won": won,
        "total_lost": lost,
        "overall_conversion_rate_pct": rate,
        "by_source": by_source,
    })))
}
