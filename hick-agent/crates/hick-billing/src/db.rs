//! Database helpers — schema creation and query functions.

use sqlx::PgPool;
use uuid::Uuid;

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let sql = include_str!("../migrations/001_initial.sql");
    sqlx::raw_sql(sql).execute(pool).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

pub struct User {
    pub id: Uuid,
    pub github_id: i64,
    pub github_login: String,
    pub github_email: Option<String>,
    pub stripe_customer_id: Option<String>,
}

/// Upsert a GitHub user.  Returns the user row.
pub async fn upsert_user(
    pool: &PgPool,
    github_id: i64,
    github_login: &str,
    github_email: Option<&str>,
) -> anyhow::Result<User> {
    let row = sqlx::query!(
        r#"
        INSERT INTO users (github_id, github_login, github_email)
        VALUES ($1, $2, $3)
        ON CONFLICT (github_id) DO UPDATE
            SET github_login  = EXCLUDED.github_login,
                github_email  = EXCLUDED.github_email,
                updated_at    = now()
        RETURNING id, github_id, github_login, github_email, stripe_customer_id
        "#,
        github_id,
        github_login,
        github_email,
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: row.id,
        github_id: row.github_id,
        github_login: row.github_login,
        github_email: row.github_email,
        stripe_customer_id: row.stripe_customer_id,
    })
}

pub async fn set_stripe_customer(
    pool: &PgPool,
    user_id: Uuid,
    customer_id: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE users SET stripe_customer_id = $1, updated_at = now() WHERE id = $2",
        customer_id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Usage events
// ---------------------------------------------------------------------------

pub struct UsageEventRow {
    pub machine_id: String,
    pub call_timestamp: chrono::DateTime<chrono::Utc>,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub estimated_cost_usd: f64,
    pub fee_usd: f64,
}

pub async fn insert_usage_events(
    pool: &PgPool,
    user_id: Option<Uuid>,
    events: Vec<UsageEventRow>,
) -> anyhow::Result<()> {
    for ev in events {
        sqlx::query!(
            r#"
            INSERT INTO usage_events (
                user_id, machine_id, call_timestamp, provider, model,
                input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
                estimated_cost_usd, fee_usd
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            user_id,
            ev.machine_id,
            ev.call_timestamp,
            ev.provider,
            ev.model,
            ev.input_tokens,
            ev.output_tokens,
            ev.cache_creation_tokens,
            ev.cache_read_tokens,
            ev.estimated_cost_usd,
            ev.fee_usd,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Current-month totals for a user.
pub struct MonthlyTotals {
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_fee_usd: f64,
}

pub async fn monthly_totals(pool: &PgPool, user_id: Uuid) -> anyhow::Result<MonthlyTotals> {
    let row = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(input_tokens + cache_read_tokens + cache_creation_tokens), 0)::bigint AS total_input,
            COALESCE(SUM(output_tokens), 0)::bigint                                            AS total_output,
            COALESCE(SUM(fee_usd), 0.0)::double precision                                      AS total_fee
        FROM usage_events
        WHERE user_id = $1
          AND date_trunc('month', call_timestamp) = date_trunc('month', now())
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(MonthlyTotals {
        total_input_tokens: row.total_input.unwrap_or(0),
        total_output_tokens: row.total_output.unwrap_or(0),
        total_fee_usd: row.total_fee.unwrap_or(0.0),
    })
}
