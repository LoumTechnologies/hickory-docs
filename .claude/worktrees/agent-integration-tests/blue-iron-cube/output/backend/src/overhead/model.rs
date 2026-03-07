use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A category of overhead expense.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverheadCategory {
    pub id: String,
    pub name: String,
    /// How often this cost recurs: monthly, annual, per_job, one_time
    pub frequency: String,
    /// Expected monthly amount (for budgeting). Annual costs are divided by 12.
    pub monthly_budget: Decimal,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

/// An actual overhead expense incurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverheadEntry {
    pub id: String,
    pub category_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Result of allocating overhead to jobs for a period.
#[derive(Debug, Serialize)]
pub struct AllocationResult {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_overhead: Decimal,
    pub jobs_in_period: i64,
    pub allocation_method: String,
    pub allocations: Vec<JobAllocation>,
    pub recovery_rate: Decimal,
}

#[derive(Debug, Serialize)]
pub struct JobAllocation {
    pub job_id: String,
    pub allocated_amount: Decimal,
}

/// Overall overhead recovery health.
#[derive(Debug, Serialize)]
pub struct RecoveryRateReport {
    /// Total overhead incurred (all time)
    pub total_overhead_incurred: Decimal,
    /// Total overhead allocated to jobs
    pub total_overhead_allocated: Decimal,
    /// allocated / incurred * 100
    pub recovery_rate_pct: Decimal,
    /// Monthly budget total
    pub monthly_budget: Decimal,
    /// Overhead per job (average)
    pub avg_overhead_per_job: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub frequency: String,
    pub monthly_budget: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntry {
    pub category_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AllocateRequest {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    /// "equal", "revenue_weighted", or "labor_hour_weighted"
    pub method: String,
}
