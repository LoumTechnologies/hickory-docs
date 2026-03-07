use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Pipeline status for a lead. Progression:
/// new_lead -> contacted -> site_visit_scheduled -> site_visit_completed
///          -> estimate_sent -> negotiating -> won | lost
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LeadStatus {
    NewLead,
    Contacted,
    SiteVisitScheduled,
    SiteVisitCompleted,
    EstimateSent,
    Negotiating,
    Won,
    Lost,
}

impl LeadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NewLead => "new_lead",
            Self::Contacted => "contacted",
            Self::SiteVisitScheduled => "site_visit_scheduled",
            Self::SiteVisitCompleted => "site_visit_completed",
            Self::EstimateSent => "estimate_sent",
            Self::Negotiating => "negotiating",
            Self::Won => "won",
            Self::Lost => "lost",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "new_lead" => Self::NewLead,
            "contacted" => Self::Contacted,
            "site_visit_scheduled" => Self::SiteVisitScheduled,
            "site_visit_completed" => Self::SiteVisitCompleted,
            "estimate_sent" => Self::EstimateSent,
            "negotiating" => Self::Negotiating,
            "won" => Self::Won,
            "lost" => Self::Lost,
            _ => Self::NewLead,
        }
    }
}

/// A lead ties a customer to a pipeline stage. One customer can have multiple
/// leads over time (e.g., they come back for a second project).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub source: String,           // google, referral, yard_sign, repeat, home_show, etc.
    pub estimated_value: Option<Decimal>,
    pub job_id: Option<String>,   // set when lead converts to a job
    pub assigned_to: Option<String>,
    pub lost_reason: Option<String>,
    pub lost_to_competitor: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// A logged interaction with a customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub customer_id: String,
    pub lead_id: Option<String>,
    pub date: NaiveDateTime,
    /// call, email, text, in_person, site_visit, voicemail, social_media
    pub interaction_type: String,
    /// inbound (customer initiated) or outbound (we initiated)
    pub direction: String,
    pub summary: String,
    pub duration_minutes: Option<i32>,
    pub created_at: NaiveDateTime,
}

/// A scheduled follow-up action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUp {
    pub id: String,
    pub customer_id: String,
    pub lead_id: Option<String>,
    pub due_date: NaiveDate,
    pub action: String,           // call, email, send_estimate, schedule_visit, etc.
    pub notes: Option<String>,
    pub completed: bool,
    pub completed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

/// Customer lifetime value summary.
#[derive(Debug, Serialize)]
pub struct CustomerLifetimeValue {
    pub customer_id: String,
    pub customer_name: String,
    pub total_jobs: i64,
    pub total_revenue: Decimal,
    pub total_referrals: i64,
    pub referral_revenue: Decimal,
    pub first_job_date: Option<String>,
    pub last_job_date: Option<String>,
    pub avg_job_value: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateLead {
    pub customer_id: String,
    pub source: String,
    pub estimated_value: Option<Decimal>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLead {
    pub status: Option<String>,
    pub estimated_value: Option<Decimal>,
    pub job_id: Option<String>,
    pub assigned_to: Option<String>,
    pub lost_reason: Option<String>,
    pub lost_to_competitor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInteraction {
    pub customer_id: String,
    pub lead_id: Option<String>,
    pub interaction_type: String,
    pub direction: String,
    pub summary: String,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowUp {
    pub customer_id: String,
    pub lead_id: Option<String>,
    pub due_date: NaiveDate,
    pub action: String,
    pub notes: Option<String>,
}
