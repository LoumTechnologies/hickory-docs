use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A standard operating procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: String,
    pub title: String,
    /// Category: site_assessment, excavation, installation, restoration, equipment,
    /// safety, admin, customer
    pub category: String,
    pub version: i32,
    pub description: Option<String>,
    pub estimated_duration_minutes: Option<i32>,
    pub required_roles: Option<String>,       // comma-separated roles
    pub required_equipment: Option<String>,    // comma-separated equipment
    pub required_materials: Option<String>,    // comma-separated materials
    pub safety_notes: Option<String>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// An individual step within a procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    pub id: String,
    pub procedure_id: String,
    pub step_number: i32,
    pub instruction: String,
    pub is_checkpoint: bool,  // requires explicit sign-off
    pub notes: Option<String>,
}

/// A procedure with its steps included.
#[derive(Debug, Serialize)]
pub struct ProcedureWithSteps {
    #[serde(flatten)]
    pub procedure: Procedure,
    pub steps: Vec<ProcedureStep>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProcedure {
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub estimated_duration_minutes: Option<i32>,
    pub required_roles: Option<String>,
    pub required_equipment: Option<String>,
    pub required_materials: Option<String>,
    pub safety_notes: Option<String>,
    pub steps: Vec<CreateStep>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStep {
    pub instruction: String,
    pub is_checkpoint: Option<bool>,
    pub notes: Option<String>,
}
