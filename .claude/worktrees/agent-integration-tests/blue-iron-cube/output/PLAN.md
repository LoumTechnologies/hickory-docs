

# Blue Iron Cube

**System of Record for French Drain Installation**

Blue Iron Cube is a business operations platform for a french drain installation
company. It tracks every aspect of a job that affects duration and cost, builds
a historical dataset, and uses linear regression to predict future job costs
so the owner can quote with confidence and hit target profit margins.

The system is designed from the start to support franchise standardization:
every process is captured as a standard operating procedure, every cost is
tracked for overhead recovery analysis, and every job becomes training data
for the estimating model.

## Usage

Generate the full system:

```bash
hick --param business_name="My Drainage Co" --features all blue-iron-cube/
```

Generate specific domains:

```bash
hick --param business_name="My Drainage Co" --features jobs,customers,estimating blue-iron-cube/
```

---

## Configuration Variables





| Variable | Default | Purpose |
|----------|---------|---------|
| `app_name` | blue-iron-cube | Application code name (used in paths, crate names) |
| `business_name` | Acme Drainage | Display name of the business |
| `rust_edition` | 2024 | Rust edition year |

---

## Feature Definitions

Each feature represents a business domain. The `jobs` domain is the core that
most other domains connect to. Enable only what you need.

**Core domains:**









**Advanced domains:**





**Meta-features:**





| Feature | Description | Requires |
|---------|-------------|----------|
| `customers` | Customer contacts and addresses | — |
| `jobs` | Central job tracking with partial completion | `customers` |
| `payments` | Payments, refunds, processing fees, balance tracking | `jobs` |
| `crew` | Workers, roles, time entries | — |
| `inventory` | Material stock and usage tracking | — |
| `fleet` | Vehicles, equipment, maintenance | — |
| `overhead` | Fixed cost tracking and recovery | — |
| `estimating` | Linear regression cost prediction | `jobs` |
| `crm` | Lead pipeline, interactions, follow-ups, lifetime value | `customers` |
| `sops` | Standard operating procedures | — |
| `all` | Everything | all domains |
| `core` | Jobs + payments + customers + crew + inventory | |
| `financial` | Jobs + payments + overhead + estimating | |

---

## Substitution Rules



---

## Verify Commands




---

## Project Configuration

### Cargo.toml

The backend is a single Rust binary with feature flags matching the domain features:


### `backend/Cargo.toml`

```toml
[package]
name = "blue-iron-cube"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v7", "serde"] }
rusqlite = { version = "0.32", features = ["bundled", "chrono"] }
rust_decimal = { version = "1", features = ["serde-str"] }

linfa = "0.7"
linfa-linear = "0.7"
ndarray = "0.15"


[dev-dependencies]
axum-test = "15"
```


### Entry Point


### `backend/src/main.rs`

```rust
use anyhow::Result;
use blue_iron_cube::run;

#[tokio::main]
async fn main() -> Result&lt;()&gt; {
    run().await
}
```


### Library Core


### `backend/src/lib.rs`

```rust
pub mod customers;
pub mod jobs;
pub mod payments;
pub mod crew;
pub mod inventory;
pub mod fleet;
pub mod overhead;
pub mod estimating;
pub mod crm;
pub mod sops;
pub mod db;
pub mod error;
pub mod types;

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub db: db::Database,
}

pub async fn run() -> Result&lt;()&gt; {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blue_iron_cube=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::Database::open("data/blue-iron-cube.db")?;
    db.run_migrations()?;

    let state = Arc::new(AppState { db });
    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &amp;'static str {
    "OK"
}

pub fn create_router(state: Arc&lt;AppState&gt;) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/health", axum::routing::get(health));
    router = router.merge(customers::routes());
    router = router.merge(jobs::routes());
    router = router.merge(payments::routes());
    router = router.merge(crew::routes());
    router = router.merge(inventory::routes());
    router = router.merge(fleet::routes());
    router = router.merge(overhead::routes());
    router = router.merge(estimating::routes());
    router = router.merge(crm::routes());
    router = router.merge(sops::routes());

    router
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
```


### Shared Types


### `backend/src/types.rs`

```rust
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

/// All monetary values are stored as Decimal to avoid floating-point errors.
pub type Money = Decimal;

/// Standard ID prefix format: "entity-uuid7"
pub fn new_id(prefix: &amp;str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::now_v7())
}

/// Read a Decimal from a rusqlite row column (stored as REAL in SQLite).
pub fn get_decimal(row: &amp;rusqlite::Row, idx: usize) -> rusqlite::Result&lt;Decimal&gt; {
    let val: f64 = row.get(idx)?;
    Ok(Decimal::from_f64(val).unwrap_or_default())
}

/// Read an optional Decimal from a rusqlite row column.
pub fn get_decimal_opt(row: &amp;rusqlite::Row, idx: usize) -> rusqlite::Result&lt;Option&lt;Decimal&gt;&gt; {
    let val: Option&lt;f64&gt; = row.get(idx)?;
    Ok(val.map(|v| Decimal::from_f64(v).unwrap_or_default()))
}

/// Soil types encountered during excavation. Affects labor time and equipment needs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SoilType {
    Clay,
    Sand,
    Loam,
    Rocky,
    Mixed,
}

/// Yard grade affects difficulty and drainage design.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum YardGrade {
    Flat,
    Slight,
    Moderate,
    Steep,
}

/// How accessible is the job site for equipment?
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SiteAccess {
    Easy,
    Moderate,
    Difficult,
    HandCarryOnly,
}
```


### Database Module


### `backend/src/db.rs`

```rust
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex&lt;Connection&gt;,
}

impl Database {
    pub fn open(path: &amp;str) -> Result&lt;Self&gt; {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn conn(&amp;self) -> std::sync::MutexGuard&lt;'_, Connection&gt; {
        self.conn.lock().expect("database lock poisoned")
    }

    pub fn run_migrations(&amp;self) -> Result&lt;()&gt; {
        let conn = self.conn();
        let migration_dir = Path::new("migrations");
        if !migration_dir.exists() {
            return Ok(());
        }
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )?;
        let mut entries: Vec&lt;_&gt; = std::fs::read_dir(migration_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let already_applied: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                [&amp;name],
                |row| row.get(0),
            )?;
            if !already_applied {
                let sql = std::fs::read_to_string(entry.path())?;
                conn.execute_batch(&amp;sql)?;
                conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [&amp;name])?;
                tracing::info!("applied migration: {}", name);
            }
        }
        Ok(())
    }
}
```


### Error Handling


### `backend/src/error.rs`

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &amp;self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From&lt;anyhow::Error&gt; for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From&lt;rusqlite::Error&gt; for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
```


### Toolchain



---

## Root Files

### README


### `README.md`

```markdown
# blue-iron-cube

**System of Record for Acme Drainage**

A french drain installation business operations platform. Tracks jobs, costs,
crew, inventory, and equipment to build a data-driven operation ready for
franchise standardization.

## Quick Start

```bash
cd backend
cargo run
```

The API runs at `http://localhost:3000`. Data is stored in `data/blue-iron-cube.db`.

## Domains


### Customers
- `GET /api/customers` — List customers
- `POST /api/customers` — Create customer
- `GET /api/customers/:id` — Get customer
- `PUT /api/customers/:id` — Update customer


### Jobs
- `GET /api/jobs` — List jobs (filterable by status, date range)
- `POST /api/jobs` — Create job
- `GET /api/jobs/:id` — Get job with all line items
- `PUT /api/jobs/:id` — Update job
- `POST /api/jobs/:id/line-items` — Add line item (material, labor, equipment, overhead)
- `GET /api/jobs/:id/costs` — Cost breakdown including processing fees
- `POST /api/jobs/:id/phase` — Mark a phase complete (assessment, excavation, pipe_install, backfill, restoration, walkthrough)
- `POST /api/jobs/:id/complete` — Mark fully complete
- `POST /api/jobs/:id/cancel` — Cancel or mark partially completed (with reason)


### Payments
- `POST /api/payments` — Record payment (cash, check, credit card, ACH, financing)
- `GET /api/payments/:id` — Get payment details
- `POST /api/refunds` — Issue a refund (partial or full)
- `GET /api/jobs/:id/payments` — Payment summary: balance due, fees absorbed, refunds
- `GET /api/payments/processing-fees/summary` — How much lost to credit card fees across all jobs


### Crew
- `GET /api/crew` — List crew members
- `POST /api/crew` — Add crew member
- `POST /api/crew/:id/time-entries` — Log hours against a job
- `GET /api/crew/:id/time-entries` — Time entry history


### Inventory
- `GET /api/inventory` — List materials with current stock
- `POST /api/inventory` — Add material type
- `POST /api/inventory/:id/receive` — Record stock received
- `POST /api/inventory/:id/use` — Record material used on a job
- `GET /api/inventory/low-stock` — Items below reorder point


### Fleet
- `GET /api/fleet/vehicles` — List vehicles
- `GET /api/fleet/equipment` — List equipment
- `POST /api/fleet/vehicles` — Add vehicle
- `POST /api/fleet/equipment` — Add equipment
- `POST /api/fleet/:id/maintenance` — Log maintenance
- `POST /api/fleet/:id/fuel` — Log fuel purchase
- `GET /api/fleet/:id/costs` — Total cost of ownership for an asset


### Overhead
- `GET /api/overhead/categories` — List overhead categories
- `POST /api/overhead/categories` — Add overhead category
- `POST /api/overhead/entries` — Record overhead expense
- `GET /api/overhead/recovery-rate` — Current overhead recovery rate
- `POST /api/overhead/allocate` — Allocate overhead to jobs in a period


### Estimating
- `POST /api/estimates` — Generate estimate for a new job
- `GET /api/estimates/:id` — Get a saved estimate
- `GET /api/estimates/model-stats` — Current regression model accuracy
- `POST /api/estimates/retrain` — Retrain model on latest job data


### CRM
- `GET /api/crm/leads` — List leads (filterable by status)
- `POST /api/crm/leads` — Create lead
- `GET /api/crm/leads/:id` — Get lead with customer details
- `PUT /api/crm/leads/:id` — Update lead status, mark won/lost
- `GET /api/crm/leads/pipeline` — Pipeline summary (count and value per stage)
- `POST /api/crm/interactions` — Log a customer interaction (call, email, text, site visit)
- `GET /api/crm/customers/:id/interactions` — Interaction history for a customer
- `GET /api/crm/follow-ups` — All pending follow-ups (sorted by due date)
- `POST /api/crm/follow-ups` — Schedule a follow-up
- `POST /api/crm/follow-ups/:id/complete` — Mark follow-up done
- `GET /api/crm/customers/:id/lifetime-value` — Total revenue, referrals, job history
- `GET /api/crm/conversion-rate` — Win/loss rate overall and by lead source


### Standard Operating Procedures
- `GET /api/sops` — List procedures
- `POST /api/sops` — Create procedure
- `GET /api/sops/:id` — Get procedure with steps
- `PUT /api/sops/:id` — Update procedure


## Architecture

This is a monolithic Rust backend with SQLite. Domains are organized as modules
with their own routes, models, and database tables. The data model is designed
so every dollar and every hour on a job is accounted for, enabling accurate
overhead recovery analysis and predictive estimating.

## License

Proprietary — Acme Drainage
```


### .gitignore


### `.gitignore`

```
# IDE
.idea/
.vscode/
*.swp
*~

# Rust
target/
Cargo.lock

# Data
data/
*.db
*.db-wal
*.db-shm

# OS
.DS_Store
Thumbs.db
```


---

## Domain Templates

The domain-specific models, routes, and migrations are defined in separate files:

- [Customers](domains/customers.hick) — Customer contacts and addresses
- [Jobs](domains/jobs.hick) — Central job tracking with phase-based partial completion
- [Payments](domains/payments.hick) — Payments, refunds, processing fees, balance tracking
- [Crew](domains/crew.hick) — Workers, roles, and time entries
- [Inventory](domains/inventory.hick) — Material stock management
- [Fleet](domains/fleet.hick) — Vehicles, equipment, maintenance, fuel
- [Overhead](domains/overhead.hick) — Fixed cost tracking and recovery analysis
- [Estimating](domains/estimating.hick) — Linear regression cost prediction
- [CRM](domains/crm.hick) — Lead pipeline, interactions, follow-ups, lifetime value
- [SOPs](domains/sops.hick) — Standard operating procedures

Shared types and enums are in [common.hick](shared/common.hick).





## Crew

Crew management tracks who did what work and for how long. Every hour is logged
against a specific job so labor cost flows into the job's cost breakdown.

Roles matter for two reasons:
1. Different roles have different hourly rates
2. The estimating model needs to know crew composition (e.g., a 2-person crew vs
   a 3-person crew changes the hours-per-linear-foot ratio)

### Data Model


### `backend/src/crew/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{CrewMember, TimeEntry};
```



### `backend/src/crew/model.rs`

```rust
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CrewRole {
    Foreman,
    Operator,        // runs mini excavator
    Laborer,
    Apprentice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub hourly_rate: Decimal,
    pub phone: Option&lt;String&gt;,
    pub certifications: Option&lt;String&gt;,  // comma-separated
    pub hire_date: NaiveDate,
    pub active: bool,
    pub created_at: NaiveDateTime,
}

/// A block of time worked by one crew member on one job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    pub crew_member_id: String,
    pub job_id: String,
    pub date: NaiveDate,
    pub hours_regular: Decimal,
    pub hours_overtime: Decimal,
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCrewMember {
    pub name: String,
    pub role: String,
    pub hourly_rate: Decimal,
    pub phone: Option&lt;String&gt;,
    pub certifications: Option&lt;String&gt;,
    pub hire_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct CreateTimeEntry {
    pub job_id: String,
    pub date: NaiveDate,
    pub hours_regular: Decimal,
    pub hours_overtime: Option&lt;Decimal&gt;,
    pub notes: Option&lt;String&gt;,
}
```


### Routes


### `backend/src/crew/routes.rs`

```rust
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

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/crew", get(list_crew).post(add_member))
        .route("/api/crew/:id/time-entries", get(list_time_entries).post(log_time))
}

async fn list_crew(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;CrewMember&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, role, hourly_rate, phone, certifications, hire_date, active, created_at
         FROM crew_members ORDER BY name"
    )?;
    let members = stmt.query_map([], |row| {
        Ok(CrewMember {
            id: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            hourly_rate: get_decimal(row, 3)?,
            phone: row.get(4)?,
            certifications: row.get(5)?,
            hire_date: row.get(6)?,
            active: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(members))
}

async fn add_member(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateCrewMember&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("crew");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO crew_members (id, name, role, hourly_rate, phone, certifications, hire_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, input.name, input.role, input.hourly_rate.to_f64().unwrap_or(0.0), input.phone, input.certifications, input.hire_date],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_time(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(crew_id): Path&lt;String&gt;,
    Json(input): Json&lt;CreateTimeEntry&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("time");
    let overtime = input.hours_overtime.unwrap_or(Decimal::ZERO);
    let conn = state.db.conn();

    // Look up hourly rate to auto-create a labor line item on the job
    let hourly_rate: Decimal = conn.query_row(
        "SELECT hourly_rate FROM crew_members WHERE id = ?1",
        [&amp;crew_id], |r| get_decimal(r, 0),
    ).map_err(|_| AppError::NotFound(format!("Crew member {} not found", crew_id)))?;

    let name: String = conn.query_row(
        "SELECT name FROM crew_members WHERE id = ?1",
        [&amp;crew_id], |r| r.get(0),
    )?;

    conn.execute(
        "INSERT INTO time_entries (id, crew_member_id, job_id, date, hours_regular, hours_overtime, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, crew_id, input.job_id, input.date, input.hours_regular.to_f64().unwrap_or(0.0), overtime.to_f64().unwrap_or(0.0), input.notes],
    )?;

    // Auto-create labor line item on the job
    let labor_cost = (input.hours_regular * hourly_rate) + (overtime * hourly_rate * Decimal::from(150) / Decimal::from(100));
    let total_hours = input.hours_regular + overtime;
    let li_id = new_id("li");
    conn.execute(
        "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost)
         VALUES (?1, ?2, 'labor', ?3, ?4, 'hours', ?5, ?6)",
        rusqlite::params![li_id, input.job_id, format!("Labor: {}", name), total_hours.to_f64().unwrap_or(0.0), hourly_rate.to_f64().unwrap_or(0.0), labor_cost.to_f64().unwrap_or(0.0)],
    )?;

    Ok(Json(serde_json::json!({ "id": id, "labor_cost": labor_cost })))
}

async fn list_time_entries(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(crew_id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;Vec&lt;TimeEntry&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, crew_member_id, job_id, date, hours_regular, hours_overtime, notes, created_at
         FROM time_entries WHERE crew_member_id = ?1 ORDER BY date DESC"
    )?;
    let entries = stmt.query_map([&amp;crew_id], |row| {
        Ok(TimeEntry {
            id: row.get(0)?,
            crew_member_id: row.get(1)?,
            job_id: row.get(2)?,
            date: row.get(3)?,
            hours_regular: get_decimal(row, 4)?,
            hours_overtime: get_decimal(row, 5)?,
            notes: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(entries))
}
```


### Migration


### `backend/migrations/003_crew.sql`

```sql
CREATE TABLE IF NOT EXISTS crew_members (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    hourly_rate REAL NOT NULL,
    phone TEXT,
    certifications TEXT,
    hire_date TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS time_entries (
    id TEXT PRIMARY KEY,
    crew_member_id TEXT NOT NULL REFERENCES crew_members(id),
    job_id TEXT NOT NULL,
    date TEXT NOT NULL,
    hours_regular REAL NOT NULL,
    hours_overtime REAL NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_time_entries_crew ON time_entries(crew_member_id);
CREATE INDEX idx_time_entries_job ON time_entries(job_id);
CREATE INDEX idx_time_entries_date ON time_entries(date);
```







## CRM

As the System of Record, this app needs to track the full customer lifecycle —
not just "who are they" but "every interaction we've had." The CRM domain adds:

- **Lead pipeline** — track leads from first contact through to won/lost
- **Interactions** — log every call, email, text, and site visit
- **Follow-ups** — scheduled reminders so no lead falls through the cracks
- **Referral tracking** — which customers bring in new business
- **Lifetime value** — total revenue from a customer across all their jobs

This is separate from the `customers` domain on purpose: `customers` is the
contact record (name, address, phone), while `crm` is the relationship history.
A franchise can operate with just `customers`, but the owner who wants to grow
needs the CRM layer.

### Data Model


### `backend/src/crm/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{Lead, Interaction, FollowUp};
```



### `backend/src/crm/model.rs`

```rust
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
    pub fn as_str(&amp;self) -> &amp;'static str {
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

    pub fn from_str(s: &amp;str) -> Self {
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
    pub estimated_value: Option&lt;Decimal&gt;,
    pub job_id: Option&lt;String&gt;,   // set when lead converts to a job
    pub assigned_to: Option&lt;String&gt;,
    pub lost_reason: Option&lt;String&gt;,
    pub lost_to_competitor: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// A logged interaction with a customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub customer_id: String,
    pub lead_id: Option&lt;String&gt;,
    pub date: NaiveDateTime,
    /// call, email, text, in_person, site_visit, voicemail, social_media
    pub interaction_type: String,
    /// inbound (customer initiated) or outbound (we initiated)
    pub direction: String,
    pub summary: String,
    pub duration_minutes: Option&lt;i32&gt;,
    pub created_at: NaiveDateTime,
}

/// A scheduled follow-up action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUp {
    pub id: String,
    pub customer_id: String,
    pub lead_id: Option&lt;String&gt;,
    pub due_date: NaiveDate,
    pub action: String,           // call, email, send_estimate, schedule_visit, etc.
    pub notes: Option&lt;String&gt;,
    pub completed: bool,
    pub completed_at: Option&lt;NaiveDateTime&gt;,
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
    pub first_job_date: Option&lt;String&gt;,
    pub last_job_date: Option&lt;String&gt;,
    pub avg_job_value: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateLead {
    pub customer_id: String,
    pub source: String,
    pub estimated_value: Option&lt;Decimal&gt;,
    pub assigned_to: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLead {
    pub status: Option&lt;String&gt;,
    pub estimated_value: Option&lt;Decimal&gt;,
    pub job_id: Option&lt;String&gt;,
    pub assigned_to: Option&lt;String&gt;,
    pub lost_reason: Option&lt;String&gt;,
    pub lost_to_competitor: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateInteraction {
    pub customer_id: String,
    pub lead_id: Option&lt;String&gt;,
    pub interaction_type: String,
    pub direction: String,
    pub summary: String,
    pub duration_minutes: Option&lt;i32&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowUp {
    pub customer_id: String,
    pub lead_id: Option&lt;String&gt;,
    pub due_date: NaiveDate,
    pub action: String,
    pub notes: Option&lt;String&gt;,
}
```


### Routes


### `backend/src/crm/routes.rs`

```rust
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

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
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
    status: Option&lt;String&gt;,
}

async fn list_leads(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Query(filter): Query&lt;LeadFilter&gt;,
) -> Result&lt;Json&lt;Vec&lt;Lead&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let leads = if let Some(status) = &amp;filter.status {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, status, source, estimated_value, job_id,
                    assigned_to, lost_reason, lost_to_competitor, created_at, updated_at
             FROM leads WHERE status = ?1 ORDER BY updated_at DESC"
        )?;
        stmt.query_map([status], map_lead)?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, status, source, estimated_value, job_id,
                    assigned_to, lost_reason, lost_to_competitor, created_at, updated_at
             FROM leads ORDER BY updated_at DESC"
        )?;
        stmt.query_map([], map_lead)?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?
    };
    Ok(Json(leads))
}

fn map_lead(row: &amp;rusqlite::Row) -> rusqlite::Result&lt;Lead&gt; {
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateLead&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();
    let lead = conn.query_row(
        "SELECT l.id, l.customer_id, c.name, l.status, l.source, l.estimated_value,
                l.job_id, l.assigned_to, l.lost_reason, l.lost_to_competitor,
                l.created_at, l.updated_at
         FROM leads l JOIN customers c ON l.customer_id = c.id
         WHERE l.id = ?1",
        [&amp;id],
        |row| Ok(serde_json::json!({
            "id": row.get::&lt;_, String&gt;(0)?,
            "customer_id": row.get::&lt;_, String&gt;(1)?,
            "customer_name": row.get::&lt;_, String&gt;(2)?,
            "status": row.get::&lt;_, String&gt;(3)?,
            "source": row.get::&lt;_, String&gt;(4)?,
            "estimated_value": row.get::&lt;_, Option&lt;f64&gt;&gt;(5)?,
            "job_id": row.get::&lt;_, Option&lt;String&gt;&gt;(6)?,
            "assigned_to": row.get::&lt;_, Option&lt;String&gt;&gt;(7)?,
            "lost_reason": row.get::&lt;_, Option&lt;String&gt;&gt;(8)?,
            "lost_to_competitor": row.get::&lt;_, Option&lt;String&gt;&gt;(9)?,
            "created_at": row.get::&lt;_, String&gt;(10)?,
            "updated_at": row.get::&lt;_, String&gt;(11)?,
        })),
    ).map_err(|_| AppError::NotFound(format!("Lead {} not found", id)))?;
    Ok(Json(lead))
}

async fn update_lead(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
    Json(input): Json&lt;UpdateLead&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
    let stages: Vec&lt;serde_json::Value&gt; = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "status": row.get::&lt;_, String&gt;(0)?,
            "count": row.get::&lt;_, i64&gt;(1)?,
            "total_value": row.get::&lt;_, f64&gt;(2)?,
        }))
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(serde_json::json!({ "stages": stages })))
}

async fn log_interaction(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateInteraction&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(customer_id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;Vec&lt;Interaction&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, customer_id, lead_id, date, interaction_type, direction,
                summary, duration_minutes, created_at
         FROM interactions WHERE customer_id = ?1 ORDER BY date DESC"
    )?;
    let interactions = stmt.query_map([&amp;customer_id], |row| {
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
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(interactions))
}

async fn pending_follow_ups(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;serde_json::Value&gt;&gt;, AppError&gt; {
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
            "id": row.get::&lt;_, String&gt;(0)?,
            "customer_id": row.get::&lt;_, String&gt;(1)?,
            "customer_name": row.get::&lt;_, String&gt;(2)?,
            "customer_phone": row.get::&lt;_, String&gt;(3)?,
            "lead_id": row.get::&lt;_, Option&lt;String&gt;&gt;(4)?,
            "due_date": row.get::&lt;_, String&gt;(5)?,
            "action": row.get::&lt;_, String&gt;(6)?,
            "notes": row.get::&lt;_, Option&lt;String&gt;&gt;(7)?,
        }))
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(follow_ups))
}

async fn create_follow_up(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateFollowUp&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();
    conn.execute(
        "UPDATE follow_ups SET completed = 1, completed_at = datetime('now') WHERE id = ?1",
        [&amp;id],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "completed": true })))
}

async fn customer_lifetime_value(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(customer_id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;CustomerLifetimeValue&gt;, AppError&gt; {
    let conn = state.db.conn();

    let name: String = conn.query_row(
        "SELECT name FROM customers WHERE id = ?1",
        [&amp;customer_id], |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("Customer {} not found", customer_id)))?;

    let total_jobs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM jobs WHERE customer_id = ?1", [&amp;customer_id], |r| r.get(0),
    )?;
    let total_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(actual_revenue, quoted_price, 0)), 0) FROM jobs WHERE customer_id = ?1",
        [&amp;customer_id], |r| r.get(0),
    )?;
    let first_job: Option&lt;String&gt; = conn.query_row(
        "SELECT MIN(created_at) FROM jobs WHERE customer_id = ?1",
        [&amp;customer_id], |r| r.get(0),
    ).ok();
    let last_job: Option&lt;String&gt; = conn.query_row(
        "SELECT MAX(created_at) FROM jobs WHERE customer_id = ?1",
        [&amp;customer_id], |r| r.get(0),
    ).ok();

    // Count customers who listed this customer as their referrer
    let referral_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM customers WHERE referred_by = ?1", [&amp;customer_id], |r| r.get(0),
    )?;
    let referral_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(j.actual_revenue, j.quoted_price, 0)), 0)
         FROM jobs j JOIN customers c ON j.customer_id = c.id
         WHERE c.referred_by = ?1",
        [&amp;customer_id], |r| r.get(0),
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
    let by_source: Vec&lt;serde_json::Value&gt; = stmt.query_map([], |row| {
        let s_won: i64 = row.get(1)?;
        let s_lost: i64 = row.get(2)?;
        let s_total: i64 = row.get(3)?;
        let s_closed = s_won + s_lost;
        Ok(serde_json::json!({
            "source": row.get::&lt;_, String&gt;(0)?,
            "won": s_won,
            "lost": s_lost,
            "open": s_total - s_closed,
            "conversion_rate_pct": if s_closed > 0 { s_won as f64 / s_closed as f64 * 100.0 } else { 0.0 },
        }))
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;

    Ok(Json(serde_json::json!({
        "total_won": won,
        "total_lost": lost,
        "overall_conversion_rate_pct": rate,
        "by_source": by_source,
    })))
}
```


### Migration


### `backend/migrations/010_crm.sql`

```sql
CREATE TABLE IF NOT EXISTS leads (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    status TEXT NOT NULL DEFAULT 'new_lead'
        CHECK(status IN ('new_lead','contacted','site_visit_scheduled','site_visit_completed',
                         'estimate_sent','negotiating','won','lost')),
    source TEXT NOT NULL,
    estimated_value REAL,
    job_id TEXT,
    assigned_to TEXT,
    lost_reason TEXT,
    lost_to_competitor TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS interactions (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    lead_id TEXT REFERENCES leads(id),
    date TEXT NOT NULL,
    interaction_type TEXT NOT NULL CHECK(interaction_type IN ('call','email','text','in_person','site_visit','voicemail','social_media')),
    direction TEXT NOT NULL CHECK(direction IN ('inbound','outbound')),
    summary TEXT NOT NULL,
    duration_minutes INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS follow_ups (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    lead_id TEXT REFERENCES leads(id),
    due_date TEXT NOT NULL,
    action TEXT NOT NULL,
    notes TEXT,
    completed BOOLEAN NOT NULL DEFAULT 0,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_leads_customer ON leads(customer_id);
CREATE INDEX idx_leads_status ON leads(status);
CREATE INDEX idx_leads_source ON leads(source);
CREATE INDEX idx_interactions_customer ON interactions(customer_id);
CREATE INDEX idx_interactions_lead ON interactions(lead_id);
CREATE INDEX idx_interactions_date ON interactions(date);
CREATE INDEX idx_follow_ups_due ON follow_ups(due_date);
CREATE INDEX idx_follow_ups_incomplete ON follow_ups(completed) WHERE completed = 0;
```







## Customers

Every job starts with a customer. Customers are the first entity created and are
referenced by jobs, estimates, and invoices. Referral tracking matters because
word-of-mouth is the primary growth channel for drain installation.

### Data Model


### `backend/src/customers/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::Customer;
```



### `backend/src/customers/model.rs`

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A customer who has or may have french drain work done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub email: Option&lt;String&gt;,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    /// How did they find us? "referral", "google", "yard_sign", "repeat", etc.
    pub referral_source: Option&lt;String&gt;,
    /// If referred, who referred them? (customer ID or free text)
    pub referred_by: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomer {
    pub name: String,
    pub phone: String,
    pub email: Option&lt;String&gt;,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub referral_source: Option&lt;String&gt;,
    pub referred_by: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomer {
    pub name: Option&lt;String&gt;,
    pub phone: Option&lt;String&gt;,
    pub email: Option&lt;String&gt;,
    pub address: Option&lt;String&gt;,
    pub city: Option&lt;String&gt;,
    pub state: Option&lt;String&gt;,
    pub zip: Option&lt;String&gt;,
    pub referral_source: Option&lt;String&gt;,
    pub referred_by: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
}
```


### API Routes


### `backend/src/customers/routes.rs`

```rust
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use crate::{AppState, error::AppError, types::new_id};
use super::model::{CreateCustomer, Customer, UpdateCustomer};

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/customers", get(list_customers).post(create_customer))
        .route("/api/customers/:id", get(get_customer).put(update_customer))
}

async fn list_customers(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;Customer&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, phone, email, address, city, state, zip,
                referral_source, referred_by, notes, created_at, updated_at
         FROM customers ORDER BY name"
    )?;
    let customers = stmt.query_map([], |row| {
        Ok(Customer {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            address: row.get(4)?,
            city: row.get(5)?,
            state: row.get(6)?,
            zip: row.get(7)?,
            referral_source: row.get(8)?,
            referred_by: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(customers))
}

async fn create_customer(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateCustomer&gt;,
) -> Result&lt;Json&lt;Customer&gt;, AppError&gt; {
    let id = new_id("cust");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO customers (id, name, phone, email, address, city, state, zip,
                                referral_source, referred_by, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            id, input.name, input.phone, input.email,
            input.address, input.city, input.state, input.zip,
            input.referral_source, input.referred_by, input.notes,
        ],
    )?;
    let customer = get_customer_by_id(&amp;conn, &amp;id)?;
    Ok(Json(customer))
}

async fn get_customer(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;Customer&gt;, AppError&gt; {
    let conn = state.db.conn();
    let customer = get_customer_by_id(&amp;conn, &amp;id)?;
    Ok(Json(customer))
}

async fn update_customer(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
    Json(input): Json&lt;UpdateCustomer&gt;,
) -> Result&lt;Json&lt;Customer&gt;, AppError&gt; {
    let conn = state.db.conn();
    // Verify exists
    get_customer_by_id(&amp;conn, &amp;id)?;
    conn.execute(
        "UPDATE customers SET
            name = COALESCE(?2, name),
            phone = COALESCE(?3, phone),
            email = COALESCE(?4, email),
            address = COALESCE(?5, address),
            city = COALESCE(?6, city),
            state = COALESCE(?7, state),
            zip = COALESCE(?8, zip),
            referral_source = COALESCE(?9, referral_source),
            referred_by = COALESCE(?10, referred_by),
            notes = COALESCE(?11, notes),
            updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![
            id, input.name, input.phone, input.email,
            input.address, input.city, input.state, input.zip,
            input.referral_source, input.referred_by, input.notes,
        ],
    )?;
    let customer = get_customer_by_id(&amp;conn, &amp;id)?;
    Ok(Json(customer))
}

fn get_customer_by_id(conn: &amp;rusqlite::Connection, id: &amp;str) -> Result&lt;Customer, AppError&gt; {
    conn.query_row(
        "SELECT id, name, phone, email, address, city, state, zip,
                referral_source, referred_by, notes, created_at, updated_at
         FROM customers WHERE id = ?1",
        [id],
        |row| Ok(Customer {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            address: row.get(4)?,
            city: row.get(5)?,
            state: row.get(6)?,
            zip: row.get(7)?,
            referral_source: row.get(8)?,
            referred_by: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("Customer {} not found", id)))
}
```


### Migration


### `backend/migrations/001_customers.sql`

```sql
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    email TEXT,
    address TEXT NOT NULL,
    city TEXT NOT NULL,
    state TEXT NOT NULL,
    zip TEXT NOT NULL,
    referral_source TEXT,
    referred_by TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_customers_name ON customers(name);
CREATE INDEX idx_customers_referral ON customers(referral_source);
```







## Estimating

The estimating engine is the strategic core of the system. It uses linear
regression on completed jobs to predict what a new job will cost, then adds the
desired profit margin to produce a quote.

**Feature vector** (inputs to the regression model):
- `linear_feet` — length of drain run (strongest predictor)
- `trench_depth_inches` — deeper = more labor and gravel
- `soil_type` — one-hot encoded (clay is hardest, sand is easiest)
- `yard_grade` — one-hot encoded (steep requires more effort)
- `site_access` — one-hot encoded (hand-carry-only multiplies labor)
- `includes_sump_pump` — binary (adds fixed cost)
- `includes_downspout_tie_in` — binary
- `number_of_catch_basins` — integer

**Target variables** (what we predict):
- `total_direct_cost` — materials + labor + equipment
- `total_crew_hours` — for scheduling

**Quoting formula:**
```
quoted_price = predicted_direct_cost
             + (predicted_direct_cost * overhead_rate)
             + (total * desired_margin_pct)
```

The model improves with every completed job. After ~20 jobs, predictions should
be within 15% of actual. After ~50 jobs, within 10%.

### Data Model


### `backend/src/estimating/mod.rs`

```rust
mod model;
mod regression;
mod routes;

pub use routes::routes;
```



### `backend/src/estimating/model.rs`

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// An estimate generated for a potential job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub id: String,
    pub customer_id: Option&lt;String&gt;,

    // Input features
    pub linear_feet: Decimal,
    pub trench_depth_inches: Decimal,
    pub soil_type: String,
    pub yard_grade: String,
    pub site_access: String,
    pub includes_sump_pump: bool,
    pub includes_downspout_tie_in: bool,
    pub number_of_catch_basins: i32,

    // Predictions
    pub predicted_material_cost: Decimal,
    pub predicted_labor_cost: Decimal,
    pub predicted_equipment_cost: Decimal,
    pub predicted_direct_cost: Decimal,
    pub predicted_overhead: Decimal,
    pub predicted_total_cost: Decimal,
    pub predicted_crew_hours: Decimal,

    // Quote
    pub desired_margin_pct: Decimal,
    pub quoted_price: Decimal,

    // Model metadata
    pub model_version: i64,
    pub confidence_note: String,
    pub sample_size: i64,

    pub created_at: NaiveDateTime,
}

/// Snapshot of a trained regression model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub version: i64,
    pub trained_at: NaiveDateTime,
    pub sample_size: i64,
    pub r_squared: f64,
    pub mean_absolute_error: f64,
    pub feature_names: Vec&lt;String&gt;,
    pub feature_weights: Vec&lt;f64&gt;,
    pub intercept: f64,
}

#[derive(Debug, Deserialize)]
pub struct EstimateRequest {
    pub customer_id: Option&lt;String&gt;,
    pub linear_feet: Decimal,
    pub trench_depth_inches: Decimal,
    pub soil_type: String,
    pub yard_grade: String,
    pub site_access: String,
    pub includes_sump_pump: Option&lt;bool&gt;,
    pub includes_downspout_tie_in: Option&lt;bool&gt;,
    pub number_of_catch_basins: Option&lt;i32&gt;,
    /// Target profit margin as a percentage (e.g., 25 for 25%)
    pub desired_margin_pct: Decimal,
}
```


### Regression Engine

This module builds and applies the linear regression model using the `linfa`
crate. It queries completed jobs, encodes features, trains the model, and
stores the weights for prediction.


### `backend/src/estimating/regression.rs`

```rust
use anyhow::Result;
use ndarray::{Array1, Array2};
use rusqlite::Connection;

/// Feature names in the order they appear in the feature matrix.
pub const FEATURE_NAMES: &amp;[&amp;str] = &amp;[
    "linear_feet",
    "trench_depth_inches",
    "soil_clay",
    "soil_rocky",
    "soil_sand",
    "soil_loam",
    "grade_slight",
    "grade_moderate",
    "grade_steep",
    "access_moderate",
    "access_difficult",
    "access_hand_carry",
    "sump_pump",
    "downspout_tie_in",
    "catch_basins",
];

/// One-hot encode a categorical value.
fn one_hot(value: &amp;str, categories: &amp;[&amp;str]) -> Vec&lt;f64&gt; {
    categories.iter().map(|c| if *c == value { 1.0 } else { 0.0 }).collect()
}

/// Build the feature matrix from completed jobs.
pub fn build_training_data(conn: &amp;Connection) -> Result&lt;(Array2&lt;f64&gt;, Array1&lt;f64&gt;, i64)&gt; {
    let mut stmt = conn.prepare(
        "SELECT j.linear_feet_actual, j.trench_depth_inches, j.soil_type, j.yard_grade,
                j.site_access, j.includes_sump_pump, j.includes_downspout_tie_in,
                j.number_of_catch_basins,
                COALESCE((SELECT SUM(total_cost) FROM job_line_items WHERE job_id = j.id AND category != 'overhead'), 0)
         FROM jobs j
         WHERE j.status IN ('completed', 'invoiced')
           AND j.linear_feet_actual IS NOT NULL"
    )?;

    let mut features: Vec&lt;Vec&lt;f64&gt;&gt; = Vec::new();
    let mut targets: Vec&lt;f64&gt; = Vec::new();

    let rows = stmt.query_map([], |row| {
        let linear_feet: f64 = row.get(0)?;
        let depth: f64 = row.get(1)?;
        let soil: String = row.get(2)?;
        let grade: String = row.get(3)?;
        let access: String = row.get(4)?;
        let sump: bool = row.get(5)?;
        let downspout: bool = row.get(6)?;
        let basins: i32 = row.get(7)?;
        let direct_cost: f64 = row.get(8)?;

        let mut feat = vec![linear_feet, depth];
        feat.extend(one_hot(&amp;soil, &amp;["clay", "rocky", "sand", "loam"]));
        feat.extend(one_hot(&amp;grade, &amp;["slight", "moderate", "steep"]));
        feat.extend(one_hot(&amp;access, &amp;["moderate", "difficult", "hand_carry_only"]));
        feat.push(if sump { 1.0 } else { 0.0 });
        feat.push(if downspout { 1.0 } else { 0.0 });
        feat.push(basins as f64);

        Ok((feat, direct_cost))
    })?;

    for row in rows {
        let (feat, target) = row?;
        features.push(feat);
        targets.push(target);
    }

    let n = features.len();
    let p = FEATURE_NAMES.len();
    let sample_size = n as i64;

    if n &lt; 3 {
        anyhow::bail!("Need at least 3 completed jobs to train the model, have {}", n);
    }

    let flat: Vec&lt;f64&gt; = features.into_iter().flatten().collect();
    let x = Array2::from_shape_vec((n, p), flat)?;
    let y = Array1::from_vec(targets);

    Ok((x, y, sample_size))
}

/// Train the model and return weights + intercept.
pub fn train(conn: &amp;Connection) -> Result&lt;(Vec&lt;f64&gt;, f64, f64, i64)&gt; {
    use linfa::prelude::*;
    use linfa_linear::LinearRegression;

    let (x, y, sample_size) = build_training_data(conn)?;

    let dataset = linfa::Dataset::new(x, y);
    let model = LinearRegression::default().fit(&amp;dataset)?;

    let weights: Vec&lt;f64&gt; = model.params().iter().copied().collect();
    let intercept = model.intercept();

    // Compute R-squared
    let predictions = model.predict(dataset.records());
    let targets = dataset.targets();
    let ss_res: f64 = predictions.iter().zip(targets.iter())
        .map(|(p, t)| (p - t).powi(2)).sum();
    let mean: f64 = targets.mean().unwrap_or(0.0);
    let ss_tot: f64 = targets.iter().map(|t| (t - mean).powi(2)).sum();
    let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

    Ok((weights, intercept, r_squared, sample_size))
}

/// Predict direct cost for given features.
pub fn predict(weights: &amp;[f64], intercept: f64, features: &amp;[f64]) -> f64 {
    let dot: f64 = weights.iter().zip(features.iter()).map(|(w, f)| w * f).sum();
    (dot + intercept).max(0.0)
}
```


### Routes


### `backend/src/estimating/routes.rs`

```rust
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::new_id};
use super::model::*;
use super::regression;

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/estimates", post(create_estimate))
        .route("/api/estimates/model-stats", get(model_stats))
        .route("/api/estimates/retrain", post(retrain))
        .route("/api/estimates/:id", get(get_estimate))
}

async fn create_estimate(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;EstimateRequest&gt;,
) -> Result&lt;Json&lt;Estimate&gt;, AppError&gt; {
    let conn = state.db.conn();

    // Load latest model weights from DB
    let model = conn.query_row(
        "SELECT version, weights, intercept, r_squared, sample_size FROM regression_models ORDER BY version DESC LIMIT 1",
        [],
        |row| {
            let weights_json: String = row.get(1)?;
            let weights: Vec&lt;f64&gt; = serde_json::from_str(&amp;weights_json).unwrap_or_default();
            Ok((row.get::&lt;_, i64&gt;(0)?, weights, row.get::&lt;_, f64&gt;(2)?, row.get::&lt;_, f64&gt;(3)?, row.get::&lt;_, i64&gt;(4)?))
        },
    ).map_err(|_| AppError::BadRequest("No trained model found. POST /api/estimates/retrain first.".into()))?;

    let (version, weights, intercept, r_squared, sample_size) = model;

    // Build feature vector
    let lf = input.linear_feet.to_f64().unwrap_or(0.0);
    let depth = input.trench_depth_inches.to_f64().unwrap_or(0.0);
    let mut features = vec![lf, depth];
    let soil_cats = ["clay", "rocky", "sand", "loam"];
    for cat in &amp;soil_cats {
        features.push(if input.soil_type == *cat { 1.0 } else { 0.0 });
    }
    let grade_cats = ["slight", "moderate", "steep"];
    for cat in &amp;grade_cats {
        features.push(if input.yard_grade == *cat { 1.0 } else { 0.0 });
    }
    let access_cats = ["moderate", "difficult", "hand_carry_only"];
    for cat in &amp;access_cats {
        features.push(if input.site_access == *cat { 1.0 } else { 0.0 });
    }
    features.push(if input.includes_sump_pump.unwrap_or(false) { 1.0 } else { 0.0 });
    features.push(if input.includes_downspout_tie_in.unwrap_or(false) { 1.0 } else { 0.0 });
    features.push(input.number_of_catch_basins.unwrap_or(0) as f64);

    let predicted_direct = regression::predict(&amp;weights, intercept, &amp;features);

    // Estimate cost breakdown (rough split based on industry norms)
    let material_pct = 0.45;
    let labor_pct = 0.40;
    let equipment_pct = 0.15;

    let material = predicted_direct * material_pct;
    let labor = predicted_direct * labor_pct;
    let equipment = predicted_direct * equipment_pct;

    // Overhead from recovery rate
    let overhead_rate: f64 = conn.query_row(
        "SELECT COALESCE(
            (SELECT SUM(monthly_budget) FROM overhead_categories) /
            NULLIF((SELECT AVG(COALESCE(actual_revenue, quoted_price)) FROM jobs WHERE status IN ('completed','invoiced')), 0),
            0.15
        )", [], |r| r.get(0),
    ).unwrap_or(0.15);

    let overhead = predicted_direct * overhead_rate;
    let total_cost = predicted_direct + overhead;
    let margin_pct = input.desired_margin_pct.to_f64().unwrap_or(25.0) / 100.0;
    let quoted_price = total_cost / (1.0 - margin_pct);

    // Estimate crew hours (rough: ~1 hour per 4 linear feet for a 2-person crew)
    let crew_hours = lf / 4.0;

    let confidence = if sample_size &lt; 10 {
        format!("Low confidence ({} jobs). Need 20+ for reliable estimates.", sample_size)
    } else if sample_size &lt; 20 {
        format!("Moderate confidence ({} jobs, R²={:.2}). Predictions within ~15%.", sample_size, r_squared)
    } else {
        format!("Good confidence ({} jobs, R²={:.2}). Predictions within ~10%.", sample_size, r_squared)
    };

    let id = new_id("est");
    use rust_decimal::prelude::FromPrimitive;
    let estimate = Estimate {
        id: id.clone(),
        customer_id: input.customer_id,
        linear_feet: input.linear_feet,
        trench_depth_inches: input.trench_depth_inches,
        soil_type: input.soil_type,
        yard_grade: input.yard_grade,
        site_access: input.site_access,
        includes_sump_pump: input.includes_sump_pump.unwrap_or(false),
        includes_downspout_tie_in: input.includes_downspout_tie_in.unwrap_or(false),
        number_of_catch_basins: input.number_of_catch_basins.unwrap_or(0),
        predicted_material_cost: Decimal::from_f64(material).unwrap_or_default(),
        predicted_labor_cost: Decimal::from_f64(labor).unwrap_or_default(),
        predicted_equipment_cost: Decimal::from_f64(equipment).unwrap_or_default(),
        predicted_direct_cost: Decimal::from_f64(predicted_direct).unwrap_or_default(),
        predicted_overhead: Decimal::from_f64(overhead).unwrap_or_default(),
        predicted_total_cost: Decimal::from_f64(total_cost).unwrap_or_default(),
        predicted_crew_hours: Decimal::from_f64(crew_hours).unwrap_or_default(),
        desired_margin_pct: input.desired_margin_pct,
        quoted_price: Decimal::from_f64(quoted_price).unwrap_or_default(),
        model_version: version,
        confidence_note: confidence,
        sample_size,
        created_at: chrono::Utc::now().naive_utc(),
    };

    // Save the estimate
    conn.execute(
        "INSERT INTO estimates (id, customer_id, linear_feet, trench_depth_inches, soil_type,
            yard_grade, site_access, includes_sump_pump, includes_downspout_tie_in,
            number_of_catch_basins, predicted_direct_cost, predicted_overhead,
            predicted_total_cost, predicted_crew_hours, desired_margin_pct, quoted_price,
            model_version, confidence_note, sample_size)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        rusqlite::params![
            id, estimate.customer_id,
            estimate.linear_feet.to_f64().unwrap_or(0.0),
            estimate.trench_depth_inches.to_f64().unwrap_or(0.0),
            estimate.soil_type, estimate.yard_grade, estimate.site_access,
            estimate.includes_sump_pump, estimate.includes_downspout_tie_in,
            estimate.number_of_catch_basins, predicted_direct, overhead, total_cost,
            crew_hours, estimate.desired_margin_pct.to_f64().unwrap_or(0.0),
            quoted_price, version, estimate.confidence_note,
            sample_size,
        ],
    )?;

    Ok(Json(estimate))
}

async fn get_estimate(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();
    let row = conn.query_row(
        "SELECT id, customer_id, linear_feet, quoted_price, confidence_note, created_at
         FROM estimates WHERE id = ?1",
        [&amp;id],
        |row| Ok(serde_json::json!({
            "id": row.get::&lt;_, String&gt;(0)?,
            "customer_id": row.get::&lt;_, Option&lt;String&gt;&gt;(1)?,
            "linear_feet": row.get::&lt;_, f64&gt;(2)?,
            "quoted_price": row.get::&lt;_, f64&gt;(3)?,
            "confidence_note": row.get::&lt;_, String&gt;(4)?,
            "created_at": row.get::&lt;_, String&gt;(5)?,
        })),
    ).map_err(|_| AppError::NotFound(format!("Estimate {} not found", id)))?;
    Ok(Json(row))
}

async fn model_stats(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;ModelStats&gt;, AppError&gt; {
    let conn = state.db.conn();
    let stats = conn.query_row(
        "SELECT version, trained_at, sample_size, r_squared, mae, weights, intercept
         FROM regression_models ORDER BY version DESC LIMIT 1",
        [],
        |row| {
            let weights_json: String = row.get(5)?;
            let weights: Vec&lt;f64&gt; = serde_json::from_str(&amp;weights_json).unwrap_or_default();
            Ok(ModelStats {
                version: row.get(0)?,
                trained_at: row.get(1)?,
                sample_size: row.get(2)?,
                r_squared: row.get(3)?,
                mean_absolute_error: row.get(4)?,
                feature_names: regression::FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
                feature_weights: weights,
                intercept: row.get(6)?,
            })
        },
    ).map_err(|_| AppError::NotFound("No trained model found. POST /api/estimates/retrain first.".into()))?;
    Ok(Json(stats))
}

async fn retrain(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;ModelStats&gt;, AppError&gt; {
    let conn = state.db.conn();
    let (weights, intercept, r_squared, sample_size) = regression::train(&amp;conn)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Compute MAE on training data
    let (x, y, _) = regression::build_training_data(&amp;conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mae: f64 = x.outer_iter().zip(y.iter()).map(|(row, target)| {
        let pred = regression::predict(&amp;weights, intercept, row.as_slice().unwrap());
        (pred - target).abs()
    }).sum::&lt;f64&gt;() / sample_size as f64;

    let weights_json = serde_json::to_string(&amp;weights).unwrap();
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM regression_models", [], |r| r.get(0),
    )?;

    conn.execute(
        "INSERT INTO regression_models (version, sample_size, r_squared, mae, weights, intercept)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![version, sample_size, r_squared, mae, weights_json, intercept],
    )?;

    Ok(Json(ModelStats {
        version,
        trained_at: chrono::Utc::now().naive_utc(),
        sample_size,
        r_squared,
        mean_absolute_error: mae,
        feature_names: regression::FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
        feature_weights: weights,
        intercept,
    }))
}
```


### Migration


### `backend/migrations/007_estimating.sql`

```sql
CREATE TABLE IF NOT EXISTS regression_models (
    version INTEGER PRIMARY KEY,
    trained_at TEXT NOT NULL DEFAULT (datetime('now')),
    sample_size INTEGER NOT NULL,
    r_squared REAL NOT NULL,
    mae REAL NOT NULL,
    weights TEXT NOT NULL,  -- JSON array of feature weights
    intercept REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS estimates (
    id TEXT PRIMARY KEY,
    customer_id TEXT,
    linear_feet REAL NOT NULL,
    trench_depth_inches REAL NOT NULL,
    soil_type TEXT NOT NULL,
    yard_grade TEXT NOT NULL,
    site_access TEXT NOT NULL,
    includes_sump_pump BOOLEAN NOT NULL DEFAULT 0,
    includes_downspout_tie_in BOOLEAN NOT NULL DEFAULT 0,
    number_of_catch_basins INTEGER NOT NULL DEFAULT 0,
    predicted_direct_cost REAL NOT NULL,
    predicted_overhead REAL NOT NULL,
    predicted_total_cost REAL NOT NULL,
    predicted_crew_hours REAL NOT NULL,
    desired_margin_pct REAL NOT NULL,
    quoted_price REAL NOT NULL,
    model_version INTEGER NOT NULL,
    confidence_note TEXT NOT NULL,
    sample_size INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```







## Fleet

Fleet management tracks vehicles and equipment. Every asset has a total cost of
ownership that factors into overhead recovery. Equipment usage on a job creates
an equipment line item so its cost is attributed to that job.

**Typical fleet for a drain installer:**
- Trucks (F-250/F-350 with dump bed or trailer)
- Trailers (equipment hauling, dump trailers)
- Mini excavators (the primary production tool)
- Plate compactors
- Sod cutters
- Trenchers (for tight-access jobs)
- Pumps (dewatering)
- Levels, lasers, and survey equipment

### Data Model


### `backend/src/fleet/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{Vehicle, Equipment, MaintenanceRecord, FuelLog};
```



### `backend/src/fleet/model.rs`

```rust
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
    pub license_plate: Option&lt;String&gt;,
    pub vin: Option&lt;String&gt;,
    pub status: String,           // active, maintenance, retired
    pub monthly_payment: Option&lt;Decimal&gt;,
    pub insurance_monthly: Option&lt;Decimal&gt;,
    pub created_at: NaiveDateTime,
}

/// A piece of equipment (excavator, compactor, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub id: String,
    pub equipment_type: String,   // mini_excavator, plate_compactor, trencher, sod_cutter, pump, laser_level
    pub make: String,
    pub model: String,
    pub serial_number: Option&lt;String&gt;,
    pub owned_or_rented: String,  // owned, rented
    pub daily_rate: Option&lt;Decimal&gt;, // cost per day if rented, or depreciation rate if owned
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
    pub vendor: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
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
    pub odometer: Option&lt;i64&gt;,
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
    pub license_plate: Option&lt;String&gt;,
    pub vin: Option&lt;String&gt;,
    pub monthly_payment: Option&lt;Decimal&gt;,
    pub insurance_monthly: Option&lt;Decimal&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateEquipment {
    pub equipment_type: String,
    pub make: String,
    pub model: String,
    pub serial_number: Option&lt;String&gt;,
    pub owned_or_rented: String,
    pub daily_rate: Option&lt;Decimal&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenance {
    pub date: NaiveDate,
    pub maintenance_type: String,
    pub cost: Decimal,
    pub vendor: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateFuelLog {
    pub date: NaiveDate,
    pub gallons: Decimal,
    pub cost_per_gallon: Decimal,
    pub odometer: Option&lt;i64&gt;,
}
```


### Routes


### `backend/src/fleet/routes.rs`

```rust
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/fleet/vehicles", get(list_vehicles).post(add_vehicle))
        .route("/api/fleet/equipment", get(list_equipment).post(add_equipment))
        .route("/api/fleet/vehicles/:id/maintenance", post(log_vehicle_maintenance))
        .route("/api/fleet/equipment/:id/maintenance", post(log_equipment_maintenance))
        .route("/api/fleet/vehicles/:id/fuel", post(log_fuel))
        .route("/api/fleet/vehicles/:id/costs", get(vehicle_costs))
}

async fn list_vehicles(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;Vehicle&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, vehicle_type, year, make, model, license_plate, vin, status,
                monthly_payment, insurance_monthly, created_at
         FROM vehicles ORDER BY year DESC"
    )?;
    let vehicles = stmt.query_map([], |row| {
        Ok(Vehicle {
            id: row.get(0)?,
            vehicle_type: row.get(1)?,
            year: row.get(2)?,
            make: row.get(3)?,
            model: row.get(4)?,
            license_plate: row.get(5)?,
            vin: row.get(6)?,
            status: row.get(7)?,
            monthly_payment: get_decimal_opt(row, 8)?,
            insurance_monthly: get_decimal_opt(row, 9)?,
            created_at: row.get(10)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(vehicles))
}

async fn list_equipment(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;Equipment&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, equipment_type, make, model, serial_number, owned_or_rented,
                daily_rate, status, created_at
         FROM equipment ORDER BY equipment_type"
    )?;
    let items = stmt.query_map([], |row| {
        Ok(Equipment {
            id: row.get(0)?,
            equipment_type: row.get(1)?,
            make: row.get(2)?,
            model: row.get(3)?,
            serial_number: row.get(4)?,
            owned_or_rented: row.get(5)?,
            daily_rate: get_decimal_opt(row, 6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(items))
}

async fn add_vehicle(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateVehicle&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("veh");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO vehicles (id, vehicle_type, year, make, model, license_plate, vin, monthly_payment, insurance_monthly)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, input.vehicle_type, input.year, input.make, input.model,
            input.license_plate, input.vin,
            input.monthly_payment.map(|d| d.to_f64().unwrap_or(0.0)),
            input.insurance_monthly.map(|d| d.to_f64().unwrap_or(0.0))],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn add_equipment(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateEquipment&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("equip");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO equipment (id, equipment_type, make, model, serial_number, owned_or_rented, daily_rate)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, input.equipment_type, input.make, input.model,
            input.serial_number, input.owned_or_rented, input.daily_rate.map(|d| d.to_f64().unwrap_or(0.0))],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_vehicle_maintenance(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(vehicle_id): Path&lt;String&gt;,
    Json(input): Json&lt;CreateMaintenance&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("maint");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO maintenance_records (id, asset_type, asset_id, date, maintenance_type, cost, vendor, notes)
         VALUES (?1, 'vehicle', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, vehicle_id, input.date, input.maintenance_type, input.cost.to_f64().unwrap_or(0.0), input.vendor, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_equipment_maintenance(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(equipment_id): Path&lt;String&gt;,
    Json(input): Json&lt;CreateMaintenance&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("maint");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO maintenance_records (id, asset_type, asset_id, date, maintenance_type, cost, vendor, notes)
         VALUES (?1, 'equipment', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, equipment_id, input.date, input.maintenance_type, input.cost.to_f64().unwrap_or(0.0), input.vendor, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn log_fuel(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(vehicle_id): Path&lt;String&gt;,
    Json(input): Json&lt;CreateFuelLog&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("fuel");
    let total = input.gallons * input.cost_per_gallon;
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO fuel_logs (id, vehicle_id, date, gallons, cost_per_gallon, total_cost, odometer)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, vehicle_id, input.date, input.gallons.to_f64().unwrap_or(0.0), input.cost_per_gallon.to_f64().unwrap_or(0.0), total.to_f64().unwrap_or(0.0), input.odometer],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "total_cost": total })))
}

async fn vehicle_costs(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(vehicle_id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;AssetCostSummary&gt;, AppError&gt; {
    let conn = state.db.conn();
    let maintenance: f64 = conn.query_row(
        "SELECT COALESCE(SUM(cost), 0) FROM maintenance_records WHERE asset_id = ?1",
        [&amp;vehicle_id], |r| r.get(0),
    )?;
    let fuel: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_cost), 0) FROM fuel_logs WHERE vehicle_id = ?1",
        [&amp;vehicle_id], |r| r.get(0),
    )?;
    // Payments and insurance are monthly; sum over months since creation
    let (payment, insurance): (f64, f64) = conn.query_row(
        "SELECT COALESCE(monthly_payment, 0) * CAST((julianday('now') - julianday(created_at)) / 30 AS INTEGER),
                COALESCE(insurance_monthly, 0) * CAST((julianday('now') - julianday(created_at)) / 30 AS INTEGER)
         FROM vehicles WHERE id = ?1",
        [&amp;vehicle_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| AppError::NotFound(format!("Vehicle {} not found", vehicle_id)))?;

    use rust_decimal::prelude::FromPrimitive;
    let m = rust_decimal::Decimal::from_f64(maintenance).unwrap_or_default();
    let f = rust_decimal::Decimal::from_f64(fuel).unwrap_or_default();
    let p = rust_decimal::Decimal::from_f64(payment).unwrap_or_default();
    let i = rust_decimal::Decimal::from_f64(insurance).unwrap_or_default();

    Ok(Json(AssetCostSummary {
        asset_id: vehicle_id,
        maintenance_total: m,
        fuel_total: f,
        payment_total: p,
        insurance_total: i,
        total_cost: m + f + p + i,
    }))
}
```


### Migration


### `backend/migrations/005_fleet.sql`

```sql
CREATE TABLE IF NOT EXISTS vehicles (
    id TEXT PRIMARY KEY,
    vehicle_type TEXT NOT NULL,
    year INTEGER NOT NULL,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    license_plate TEXT,
    vin TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    monthly_payment REAL,
    insurance_monthly REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS equipment (
    id TEXT PRIMARY KEY,
    equipment_type TEXT NOT NULL,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    serial_number TEXT,
    owned_or_rented TEXT NOT NULL DEFAULT 'owned',
    daily_rate REAL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS maintenance_records (
    id TEXT PRIMARY KEY,
    asset_type TEXT NOT NULL CHECK(asset_type IN ('vehicle', 'equipment')),
    asset_id TEXT NOT NULL,
    date TEXT NOT NULL,
    maintenance_type TEXT NOT NULL,
    cost REAL NOT NULL,
    vendor TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fuel_logs (
    id TEXT PRIMARY KEY,
    vehicle_id TEXT NOT NULL REFERENCES vehicles(id),
    date TEXT NOT NULL,
    gallons REAL NOT NULL,
    cost_per_gallon REAL NOT NULL,
    total_cost REAL NOT NULL,
    odometer INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_maintenance_asset ON maintenance_records(asset_id);
CREATE INDEX idx_fuel_vehicle ON fuel_logs(vehicle_id);
```







## Inventory

French drain installation uses a consistent set of materials. Tracking inventory
serves two purposes:
1. Know when to reorder (prevent job delays from stockouts)
2. Track actual material cost per job (feed into estimating model)

**Common french drain materials:**
- Corrugated drain pipe (4" perforated, 4" solid)
- Gravel/aggregate (washed #57 stone, pea gravel)
- Filter fabric (landscape fabric, filter sock)
- Fittings (couplers, tees, elbows, end caps, pop-up emitters)
- Catch basins and grates
- Sump pumps and check valves
- PVC pipe (for sump discharge)
- Concrete/mortar (for basin installation)
- Sod/seed (for restoration)

### Data Model


### `backend/src/inventory/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{InventoryItem, InventoryTransaction};
```



### `backend/src/inventory/model.rs`

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub category: String,       // pipe, gravel, fabric, fitting, pump, restoration
    pub unit: String,            // feet, tons, sq_ft, each, bags
    pub quantity_on_hand: Decimal,
    pub reorder_point: Decimal,
    pub unit_cost: Decimal,      // last known cost per unit
    pub supplier: Option&lt;String&gt;,
    pub supplier_sku: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Purchase,    // stock received from supplier
    Usage,       // material used on a job
    Adjustment,  // manual correction (damage, count correction)
    Return,      // returned to supplier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransaction {
    pub id: String,
    pub item_id: String,
    pub transaction_type: String,
    pub quantity: Decimal,       // positive for in, negative for out
    pub unit_cost: Option&lt;Decimal&gt;,
    pub job_id: Option&lt;String&gt;,  // set when type=usage
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub name: String,
    pub category: String,
    pub unit: String,
    pub reorder_point: Decimal,
    pub unit_cost: Decimal,
    pub supplier: Option&lt;String&gt;,
    pub supplier_sku: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct ReceiveStock {
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    pub notes: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct UseStock {
    pub quantity: Decimal,
    pub job_id: String,
    pub notes: Option&lt;String&gt;,
}
```


### Routes


### `backend/src/inventory/routes.rs`

```rust
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/inventory", get(list_items).post(add_item))
        .route("/api/inventory/low-stock", get(low_stock))
        .route("/api/inventory/:id/receive", post(receive_stock))
        .route("/api/inventory/:id/use", post(use_stock))
}

async fn list_items(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;InventoryItem&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, category, unit, quantity_on_hand, reorder_point,
                unit_cost, supplier, supplier_sku, notes, created_at, updated_at
         FROM inventory_items ORDER BY category, name"
    )?;
    let items = stmt.query_map([], |row| {
        Ok(InventoryItem {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            unit: row.get(3)?,
            quantity_on_hand: get_decimal(row, 4)?,
            reorder_point: get_decimal(row, 5)?,
            unit_cost: get_decimal(row, 6)?,
            supplier: row.get(7)?,
            supplier_sku: row.get(8)?,
            notes: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(items))
}

async fn add_item(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateItem&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("inv");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO inventory_items (id, name, category, unit, quantity_on_hand, reorder_point, unit_cost, supplier, supplier_sku, notes)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, input.name, input.category, input.unit, input.reorder_point.to_f64().unwrap_or(0.0), input.unit_cost.to_f64().unwrap_or(0.0), input.supplier, input.supplier_sku, input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn low_stock(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;InventoryItem&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, category, unit, quantity_on_hand, reorder_point,
                unit_cost, supplier, supplier_sku, notes, created_at, updated_at
         FROM inventory_items
         WHERE quantity_on_hand &lt;= reorder_point
         ORDER BY (quantity_on_hand - reorder_point)"
    )?;
    let items = stmt.query_map([], |row| {
        Ok(InventoryItem {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            unit: row.get(3)?,
            quantity_on_hand: get_decimal(row, 4)?,
            reorder_point: get_decimal(row, 5)?,
            unit_cost: get_decimal(row, 6)?,
            supplier: row.get(7)?,
            supplier_sku: row.get(8)?,
            notes: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(items))
}

async fn receive_stock(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(item_id): Path&lt;String&gt;,
    Json(input): Json&lt;ReceiveStock&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("txn");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO inventory_transactions (id, item_id, transaction_type, quantity, unit_cost, notes)
         VALUES (?1, ?2, 'purchase', ?3, ?4, ?5)",
        rusqlite::params![id, item_id, input.quantity.to_f64().unwrap_or(0.0), input.unit_cost.to_f64().unwrap_or(0.0), input.notes],
    )?;
    conn.execute(
        "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand + ?2, unit_cost = ?3, updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![item_id, input.quantity.to_f64().unwrap_or(0.0), input.unit_cost.to_f64().unwrap_or(0.0)],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn use_stock(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(item_id): Path&lt;String&gt;,
    Json(input): Json&lt;UseStock&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("txn");
    let conn = state.db.conn();

    // Record the transaction
    conn.execute(
        "INSERT INTO inventory_transactions (id, item_id, transaction_type, quantity, job_id, notes)
         VALUES (?1, ?2, 'usage', ?3, ?4, ?5)",
        rusqlite::params![id, item_id, -input.quantity.to_f64().unwrap_or(0.0), input.job_id, input.notes],
    )?;
    conn.execute(
        "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand - ?2, updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![item_id, input.quantity.to_f64().unwrap_or(0.0)],
    )?;

    // Auto-create material line item on the job
    let (name, unit_cost, unit): (String, f64, String) = conn.query_row(
        "SELECT name, unit_cost, unit FROM inventory_items WHERE id = ?1",
        [&amp;item_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let qty_f64 = input.quantity.to_f64().unwrap_or(0.0);
    let total: f64 = qty_f64 * unit_cost;
    let li_id = new_id("li");
    conn.execute(
        "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost)
         VALUES (?1, ?2, 'material', ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![li_id, input.job_id, name, qty_f64, unit, unit_cost, total],
    )?;

    Ok(Json(serde_json::json!({ "id": id, "material_cost": total })))
}
```


### Migration


### `backend/migrations/004_inventory.sql`

```sql
CREATE TABLE IF NOT EXISTS inventory_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    unit TEXT NOT NULL,
    quantity_on_hand REAL NOT NULL DEFAULT 0,
    reorder_point REAL NOT NULL DEFAULT 0,
    unit_cost REAL NOT NULL DEFAULT 0,
    supplier TEXT,
    supplier_sku TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inventory_transactions (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES inventory_items(id),
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('purchase', 'usage', 'adjustment', 'return')),
    quantity REAL NOT NULL,
    unit_cost REAL,
    job_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_inv_txn_item ON inventory_transactions(item_id);
CREATE INDEX idx_inv_txn_job ON inventory_transactions(job_id);
CREATE INDEX idx_inv_category ON inventory_items(category);
```







## Jobs

The job is the central entity. Every dollar spent and every hour worked ties back
to a job. A job captures the site conditions (soil type, grade, access) that drive
cost variability — these become the feature vector for the estimating model.

**Cost tracking philosophy:** A job's total cost is the sum of its line items. Line
items are categorized as material, labor, equipment, or overhead. This granularity
lets us answer "what did this job actually cost?" at any level of detail.

**Key metrics per job:**
- **Gross revenue** — what the customer paid
- **Direct cost** — materials + labor + equipment
- **Processing fees** — credit card fees absorbed by the business
- **Allocated overhead** — share of fixed costs
- **True cost** — direct cost + processing fees + allocated overhead
- **True profit margin** — (revenue - true cost) / revenue

**Partial completion:** Jobs aren't always all-or-nothing. A job can be partially
completed (e.g., excavation done but rain halted restoration). The `completion_pct`
field and per-phase tracking let the owner see exactly where things stand. A
partially completed job still has real costs that must be accounted for.

**Job phases** (natural milestones for a french drain install):
1. Site assessment and marking
2. Excavation
3. Pipe installation and grading
4. Backfill and compaction
5. Restoration (sod/seed/cleanup)
6. Final walkthrough and sign-off

### Data Model


### `backend/src/jobs/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{Job, JobLineItem, JobStatus, LineItemCategory};
```



### `backend/src/jobs/model.rs`

```rust
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status progression: lead -> scheduled -> in_progress -> completed -> invoiced
/// A job can also be partially_completed (work stopped before 100%) or cancelled.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Lead,
    Scheduled,
    InProgress,
    PartiallyCompleted,
    Completed,
    Invoiced,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&amp;self) -> &amp;'static str {
        match self {
            Self::Lead => "lead",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::PartiallyCompleted => "partially_completed",
            Self::Completed => "completed",
            Self::Invoiced => "invoiced",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &amp;str) -> Option&lt;Self&gt; {
        match s {
            "lead" => Some(Self::Lead),
            "scheduled" => Some(Self::Scheduled),
            "in_progress" => Some(Self::InProgress),
            "partially_completed" => Some(Self::PartiallyCompleted),
            "completed" => Some(Self::Completed),
            "invoiced" => Some(Self::Invoiced),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LineItemCategory {
    Material,
    Labor,
    Equipment,
    Overhead,
}

/// A french drain installation job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub customer_id: String,
    pub status: JobStatus,

    // Site characteristics — these drive cost variability
    pub site_address: String,
    pub soil_type: String,        // clay, sand, loam, rocky, mixed
    pub yard_grade: String,       // flat, slight, moderate, steep
    pub site_access: String,      // easy, moderate, difficult, hand_carry_only
    pub site_notes: Option&lt;String&gt;,

    // Scope
    pub linear_feet_planned: Option&lt;Decimal&gt;,
    pub linear_feet_actual: Option&lt;Decimal&gt;,
    pub trench_depth_inches: Option&lt;Decimal&gt;,
    pub includes_sump_pump: bool,
    pub includes_downspout_tie_in: bool,
    pub number_of_catch_basins: i32,

    // Schedule
    pub scheduled_date: Option&lt;NaiveDate&gt;,
    pub start_date: Option&lt;NaiveDate&gt;,
    pub end_date: Option&lt;NaiveDate&gt;,
    pub estimated_crew_hours: Option&lt;Decimal&gt;,
    pub actual_crew_hours: Option&lt;Decimal&gt;,

    // Conditions on the day
    pub weather_conditions: Option&lt;String&gt;,
    pub ground_was_wet: Option&lt;bool&gt;,

    // Completion tracking
    pub completion_pct: i32,              // 0-100
    pub phase_assessment: bool,
    pub phase_excavation: bool,
    pub phase_pipe_install: bool,
    pub phase_backfill: bool,
    pub phase_restoration: bool,
    pub phase_walkthrough: bool,
    pub cancellation_reason: Option&lt;String&gt;,

    // Financials (computed from line items and payments)
    pub quoted_price: Option&lt;Decimal&gt;,
    pub actual_revenue: Option&lt;Decimal&gt;,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// An individual cost line on a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLineItem {
    pub id: String,
    pub job_id: String,
    pub category: LineItemCategory,
    pub description: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_cost: Decimal,
    pub total_cost: Decimal,
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
}

/// Cost summary computed from line items and payments.
#[derive(Debug, Serialize)]
pub struct JobCostBreakdown {
    pub job_id: String,
    pub material_cost: Decimal,
    pub labor_cost: Decimal,
    pub equipment_cost: Decimal,
    pub processing_fees: Decimal,   // credit card fees absorbed
    pub overhead_allocated: Decimal,
    pub direct_cost: Decimal,       // material + labor + equipment
    pub true_cost: Decimal,         // direct + processing_fees + overhead
    pub revenue: Decimal,
    pub gross_margin: Decimal,      // revenue - direct
    pub true_margin: Decimal,       // revenue - true cost
    pub true_margin_pct: Decimal,   // true_margin / revenue
    pub completion_pct: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateJob {
    pub customer_id: String,
    pub site_address: String,
    pub soil_type: String,
    pub yard_grade: String,
    pub site_access: String,
    pub site_notes: Option&lt;String&gt;,
    pub linear_feet_planned: Option&lt;Decimal&gt;,
    pub trench_depth_inches: Option&lt;Decimal&gt;,
    pub includes_sump_pump: Option&lt;bool&gt;,
    pub includes_downspout_tie_in: Option&lt;bool&gt;,
    pub number_of_catch_basins: Option&lt;i32&gt;,
    pub scheduled_date: Option&lt;NaiveDate&gt;,
    pub estimated_crew_hours: Option&lt;Decimal&gt;,
    pub quoted_price: Option&lt;Decimal&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateLineItem {
    pub category: LineItemCategory,
    pub description: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_cost: Decimal,
    pub notes: Option&lt;String&gt;,
}
```


### Routes


### `backend/src/jobs/routes.rs`

```rust
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

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/jobs", get(list_jobs).post(create_job))
        .route("/api/jobs/:id", get(get_job))
        .route("/api/jobs/:id/line-items", post(add_line_item))
        .route("/api/jobs/:id/costs", get(cost_breakdown))
        .route("/api/jobs/:id/complete", post(complete_job))
        .route("/api/jobs/:id/phase", post(update_phase))
        .route("/api/jobs/:id/cancel", post(cancel_job))
}

async fn list_jobs(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;Job&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, customer_id, status, site_address, soil_type, yard_grade,
                site_access, site_notes, linear_feet_planned, linear_feet_actual,
                trench_depth_inches, includes_sump_pump, includes_downspout_tie_in,
                number_of_catch_basins, scheduled_date, start_date, end_date,
                estimated_crew_hours, actual_crew_hours, weather_conditions,
                ground_was_wet, completion_pct, phase_assessment, phase_excavation,
                phase_pipe_install, phase_backfill, phase_restoration, phase_walkthrough,
                cancellation_reason, quoted_price, actual_revenue, created_at, updated_at
         FROM jobs ORDER BY created_at DESC"
    )?;
    let jobs = stmt.query_map([], |row| {
        Ok(Job {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            status: JobStatus::from_str(&amp;row.get::&lt;_, String&gt;(2)?).unwrap_or(JobStatus::Lead),
            site_address: row.get(3)?,
            soil_type: row.get(4)?,
            yard_grade: row.get(5)?,
            site_access: row.get(6)?,
            site_notes: row.get(7)?,
            linear_feet_planned: get_decimal_opt(row, 8)?,
            linear_feet_actual: get_decimal_opt(row, 9)?,
            trench_depth_inches: get_decimal_opt(row, 10)?,
            includes_sump_pump: row.get(11)?,
            includes_downspout_tie_in: row.get(12)?,
            number_of_catch_basins: row.get(13)?,
            scheduled_date: row.get(14)?,
            start_date: row.get(15)?,
            end_date: row.get(16)?,
            estimated_crew_hours: get_decimal_opt(row, 17)?,
            actual_crew_hours: get_decimal_opt(row, 18)?,
            weather_conditions: row.get(19)?,
            ground_was_wet: row.get(20)?,
            completion_pct: row.get(21)?,
            phase_assessment: row.get(22)?,
            phase_excavation: row.get(23)?,
            phase_pipe_install: row.get(24)?,
            phase_backfill: row.get(25)?,
            phase_restoration: row.get(26)?,
            phase_walkthrough: row.get(27)?,
            cancellation_reason: row.get(28)?,
            quoted_price: get_decimal_opt(row, 29)?,
            actual_revenue: get_decimal_opt(row, 30)?,
            created_at: row.get(31)?,
            updated_at: row.get(32)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(jobs))
}

async fn create_job(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateJob&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("job");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO jobs (id, customer_id, status, site_address, soil_type, yard_grade,
                           site_access, site_notes, linear_feet_planned, trench_depth_inches,
                           includes_sump_pump, includes_downspout_tie_in, number_of_catch_basins,
                           scheduled_date, estimated_crew_hours, quoted_price)
         VALUES (?1, ?2, 'lead', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![
            id, input.customer_id, input.site_address, input.soil_type,
            input.yard_grade, input.site_access, input.site_notes,
            input.linear_feet_planned.map(|d| d.to_f64().unwrap_or(0.0)),
            input.trench_depth_inches.map(|d| d.to_f64().unwrap_or(0.0)),
            input.includes_sump_pump.unwrap_or(false),
            input.includes_downspout_tie_in.unwrap_or(false),
            input.number_of_catch_basins.unwrap_or(0),
            input.scheduled_date,
            input.estimated_crew_hours.map(|d| d.to_f64().unwrap_or(0.0)),
            input.quoted_price.map(|d| d.to_f64().unwrap_or(0.0)),
        ],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn get_job(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();
    // Get job + line items in one response
    let job_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM jobs WHERE id = ?1", [&amp;id], |r| r.get(0)
    )?;
    if !job_exists {
        return Err(AppError::NotFound(format!("Job {} not found", id)));
    }

    let mut stmt = conn.prepare(
        "SELECT id, category, description, quantity, unit, unit_cost, total_cost, notes, created_at
         FROM job_line_items WHERE job_id = ?1 ORDER BY created_at"
    )?;
    let line_items: Vec&lt;serde_json::Value&gt; = stmt.query_map([&amp;id], |row| {
        Ok(serde_json::json!({
            "id": row.get::&lt;_, String&gt;(0)?,
            "category": row.get::&lt;_, String&gt;(1)?,
            "description": row.get::&lt;_, String&gt;(2)?,
            "quantity": row.get::&lt;_, f64&gt;(3)?,
            "unit": row.get::&lt;_, String&gt;(4)?,
            "unit_cost": row.get::&lt;_, f64&gt;(5)?,
            "total_cost": row.get::&lt;_, f64&gt;(6)?,
            "notes": row.get::&lt;_, Option&lt;String&gt;&gt;(7)?,
        }))
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;

    Ok(Json(serde_json::json!({
        "id": id,
        "line_items": line_items,
    })))
}

async fn add_line_item(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(job_id): Path&lt;String&gt;,
    Json(input): Json&lt;CreateLineItem&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("li");
    let total = input.quantity * input.unit_cost;
    let category = match input.category {
        LineItemCategory::Material => "material",
        LineItemCategory::Labor => "labor",
        LineItemCategory::Equipment => "equipment",
        LineItemCategory::Overhead => "overhead",
    };
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, job_id, category, input.description,
            input.quantity.to_f64().unwrap_or(0.0), input.unit,
            input.unit_cost.to_f64().unwrap_or(0.0), total.to_f64().unwrap_or(0.0),
            input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id, "total_cost": total })))
}

async fn cost_breakdown(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;JobCostBreakdown&gt;, AppError&gt; {
    let conn = state.db.conn();
    let (revenue, completion_pct): (Decimal, i32) = conn.query_row(
        "SELECT COALESCE(actual_revenue, quoted_price, 0), completion_pct FROM jobs WHERE id = ?1",
        [&amp;id], |r| Ok((get_decimal(r, 0)?, r.get(1)?)),
    ).map_err(|_| AppError::NotFound(format!("Job {} not found", id)))?;

    let sum_category = |cat: &amp;str| -> Result&lt;Decimal, AppError&gt; {
        Ok(conn.query_row(
            "SELECT COALESCE(SUM(total_cost), 0) FROM job_line_items WHERE job_id = ?1 AND category = ?2",
            rusqlite::params![&amp;id, cat], |r| get_decimal(r, 0),
        )?)
    };

    let material = sum_category("material")?;
    let labor = sum_category("labor")?;
    let equipment = sum_category("equipment")?;
    let processing_fees = sum_category("processing_fee")?;
    let overhead = sum_category("overhead")?;
    let direct = material + labor + equipment;
    let true_cost = direct + processing_fees + overhead;
    let gross_margin = revenue - direct;
    let true_margin = revenue - true_cost;
    let true_margin_pct = if revenue > Decimal::ZERO {
        true_margin / revenue * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    Ok(Json(JobCostBreakdown {
        job_id: id,
        material_cost: material,
        labor_cost: labor,
        equipment_cost: equipment,
        processing_fees,
        overhead_allocated: overhead,
        direct_cost: direct,
        true_cost,
        revenue,
        gross_margin,
        true_margin,
        true_margin_pct,
        completion_pct,
    }))
}

async fn complete_job(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();
    conn.execute(
        "UPDATE jobs SET status = 'completed', completion_pct = 100, end_date = date('now'),
                phase_assessment = 1, phase_excavation = 1, phase_pipe_install = 1,
                phase_backfill = 1, phase_restoration = 1, phase_walkthrough = 1,
                updated_at = datetime('now')
         WHERE id = ?1",
        [&amp;id],
    )?;
    Ok(Json(serde_json::json!({ "status": "completed", "completion_pct": 100 })))
}

#[derive(Debug, serde::Deserialize)]
struct PhaseUpdate {
    phase: String,
    completed: bool,
}

/// Mark individual phases complete/incomplete. Completion percentage auto-computes
/// from the number of completed phases (each phase = ~17%).
async fn update_phase(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
    Json(input): Json&lt;PhaseUpdate&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let col = match input.phase.as_str() {
        "assessment" => "phase_assessment",
        "excavation" => "phase_excavation",
        "pipe_install" => "phase_pipe_install",
        "backfill" => "phase_backfill",
        "restoration" => "phase_restoration",
        "walkthrough" => "phase_walkthrough",
        _ => return Err(AppError::BadRequest(format!("Unknown phase: {}", input.phase))),
    };
    let conn = state.db.conn();
    // Update the phase
    conn.execute(
        &amp;format!("UPDATE jobs SET {} = ?2, updated_at = datetime('now') WHERE id = ?1", col),
        rusqlite::params![id, input.completed],
    )?;
    // Recompute completion_pct from phases
    let pct: i32 = conn.query_row(
        "SELECT (CAST(phase_assessment AS INTEGER) + CAST(phase_excavation AS INTEGER) +
                 CAST(phase_pipe_install AS INTEGER) + CAST(phase_backfill AS INTEGER) +
                 CAST(phase_restoration AS INTEGER) + CAST(phase_walkthrough AS INTEGER)) * 100 / 6
         FROM jobs WHERE id = ?1",
        [&amp;id], |r| r.get(0),
    )?;
    // Auto-set status based on completion
    let status = if pct == 100 { "completed" } else if pct > 0 { "in_progress" } else { "scheduled" };
    conn.execute(
        "UPDATE jobs SET completion_pct = ?2, status = ?3, updated_at = datetime('now') WHERE id = ?1",
        rusqlite::params![id, pct, status],
    )?;
    Ok(Json(serde_json::json!({ "completion_pct": pct, "status": status })))
}

#[derive(Debug, serde::Deserialize)]
struct CancelRequest {
    reason: String,
}

async fn cancel_job(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
    Json(input): Json&lt;CancelRequest&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();
    let current_pct: i32 = conn.query_row(
        "SELECT completion_pct FROM jobs WHERE id = ?1",
        [&amp;id], |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("Job {} not found", id)))?;

    let status = if current_pct > 0 { "partially_completed" } else { "cancelled" };
    conn.execute(
        "UPDATE jobs SET status = ?2, cancellation_reason = ?3, end_date = date('now'),
                updated_at = datetime('now')
         WHERE id = ?1",
        rusqlite::params![id, status, input.reason],
    )?;
    Ok(Json(serde_json::json!({ "status": status, "completion_pct": current_pct })))
}
```


### Migration

The jobs table captures every variable that affects cost. These columns become
the feature vector for the estimating model's linear regression.


### `backend/migrations/002_jobs.sql`

```sql
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    status TEXT NOT NULL DEFAULT 'lead',

    -- Site characteristics (cost drivers)
    site_address TEXT NOT NULL,
    soil_type TEXT NOT NULL DEFAULT 'mixed',
    yard_grade TEXT NOT NULL DEFAULT 'flat',
    site_access TEXT NOT NULL DEFAULT 'easy',
    site_notes TEXT,

    -- Scope
    linear_feet_planned REAL,
    linear_feet_actual REAL,
    trench_depth_inches REAL,
    includes_sump_pump BOOLEAN NOT NULL DEFAULT 0,
    includes_downspout_tie_in BOOLEAN NOT NULL DEFAULT 0,
    number_of_catch_basins INTEGER NOT NULL DEFAULT 0,

    -- Schedule
    scheduled_date TEXT,
    start_date TEXT,
    end_date TEXT,
    estimated_crew_hours REAL,
    actual_crew_hours REAL,

    -- Conditions
    weather_conditions TEXT,
    ground_was_wet BOOLEAN,

    -- Completion tracking
    completion_pct INTEGER NOT NULL DEFAULT 0,
    phase_assessment BOOLEAN NOT NULL DEFAULT 0,
    phase_excavation BOOLEAN NOT NULL DEFAULT 0,
    phase_pipe_install BOOLEAN NOT NULL DEFAULT 0,
    phase_backfill BOOLEAN NOT NULL DEFAULT 0,
    phase_restoration BOOLEAN NOT NULL DEFAULT 0,
    phase_walkthrough BOOLEAN NOT NULL DEFAULT 0,
    cancellation_reason TEXT,

    -- Financials
    quoted_price REAL,
    actual_revenue REAL,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS job_line_items (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    category TEXT NOT NULL CHECK(category IN ('material', 'labor', 'equipment', 'overhead', 'processing_fee')),
    description TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit TEXT NOT NULL,
    unit_cost REAL NOT NULL,
    total_cost REAL NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_jobs_customer ON jobs(customer_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_scheduled ON jobs(scheduled_date);
CREATE INDEX idx_jobs_completion ON jobs(completion_pct);
CREATE INDEX idx_line_items_job ON job_line_items(job_id);
CREATE INDEX idx_line_items_category ON job_line_items(category);
```







## Overhead Recovery

This is the domain the business owner cares about most. Overhead recovery answers
the question: "Am I charging enough per job to cover my fixed costs?"

**How overhead recovery works:**
1. Define overhead categories (insurance, rent, vehicle payments, marketing, etc.)
2. Record actual overhead expenses as they occur
3. At the end of each period (weekly/monthly), allocate overhead to completed jobs
4. Compare allocated overhead to actual overhead to compute the recovery rate

**Allocation methods:**
- **Per-job equal split** — total overhead / number of jobs (simplest)
- **Revenue-weighted** — proportional to each job's revenue (most common)
- **Labor-hour-weighted** — proportional to labor hours (best for this business)

A recovery rate below 100% means jobs are underpriced. The estimating model uses
the target recovery rate when computing quoted prices.

### Data Model


### `backend/src/overhead/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{OverheadCategory, OverheadEntry};
```



### `backend/src/overhead/model.rs`

```rust
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
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
}

/// An actual overhead expense incurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverheadEntry {
    pub id: String,
    pub category_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: Option&lt;String&gt;,
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
    pub allocations: Vec&lt;JobAllocation&gt;,
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
    pub notes: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntry {
    pub category_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct AllocateRequest {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    /// "equal", "revenue_weighted", or "labor_hour_weighted"
    pub method: String,
}
```


### Routes


### `backend/src/overhead/routes.rs`

```rust
use std::sync::Arc;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::{AppState, error::AppError, types::{new_id, get_decimal, get_decimal_opt}};
use super::model::*;

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/overhead/categories", get(list_categories).post(add_category))
        .route("/api/overhead/entries", post(add_entry))
        .route("/api/overhead/recovery-rate", get(recovery_rate))
        .route("/api/overhead/allocate", post(allocate))
}

async fn list_categories(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;OverheadCategory&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, frequency, monthly_budget, notes, created_at
         FROM overhead_categories ORDER BY name"
    )?;
    let cats = stmt.query_map([], |row| {
        Ok(OverheadCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            frequency: row.get(2)?,
            monthly_budget: get_decimal(row, 3)?,
            notes: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(cats))
}

async fn add_category(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateCategory&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("ohcat");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO overhead_categories (id, name, frequency, monthly_budget, notes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, input.name, input.frequency, input.monthly_budget.to_f64().unwrap_or(0.0), input.notes],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn add_entry(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateEntry&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("oh");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO overhead_entries (id, category_id, date, amount, description)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, input.category_id, input.date, input.amount.to_f64().unwrap_or(0.0), input.description],
    )?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn recovery_rate(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;RecoveryRateReport&gt;, AppError&gt; {
    let conn = state.db.conn();

    let total_incurred: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM overhead_entries", [], |r| get_decimal(r, 0),
    )?;
    let total_allocated: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(total_cost), 0) FROM job_line_items WHERE category = 'overhead'",
        [], |r| get_decimal(r, 0),
    )?;
    let monthly_budget: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(monthly_budget), 0) FROM overhead_categories", [], |r| get_decimal(r, 0),
    )?;
    let job_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT job_id) FROM job_line_items WHERE category = 'overhead'",
        [], |r| r.get(0),
    )?;

    let recovery_pct = if total_incurred > Decimal::ZERO {
        total_allocated / total_incurred * Decimal::from(100)
    } else {
        Decimal::ZERO
    };
    let avg_per_job = if job_count > 0 {
        total_allocated / Decimal::from(job_count)
    } else {
        Decimal::ZERO
    };

    Ok(Json(RecoveryRateReport {
        total_overhead_incurred: total_incurred,
        total_overhead_allocated: total_allocated,
        recovery_rate_pct: recovery_pct,
        monthly_budget,
        avg_overhead_per_job: avg_per_job,
    }))
}

async fn allocate(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;AllocateRequest&gt;,
) -> Result&lt;Json&lt;AllocationResult&gt;, AppError&gt; {
    let conn = state.db.conn();

    // Sum overhead entries in the period
    let total_overhead: Decimal = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM overhead_entries WHERE date BETWEEN ?1 AND ?2",
        rusqlite::params![input.period_start, input.period_end], |r| get_decimal(r, 0),
    )?;

    // Find completed jobs in the period
    let mut stmt = conn.prepare(
        "SELECT id FROM jobs WHERE status IN ('completed', 'invoiced') AND end_date BETWEEN ?1 AND ?2"
    )?;
    let job_ids: Vec&lt;String&gt; = stmt.query_map(
        rusqlite::params![input.period_start, input.period_end],
        |row| row.get(0),
    )?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;

    let job_count = job_ids.len() as i64;
    if job_count == 0 {
        return Ok(Json(AllocationResult {
            period_start: input.period_start,
            period_end: input.period_end,
            total_overhead,
            jobs_in_period: 0,
            allocation_method: input.method.clone(),
            allocations: vec![],
            recovery_rate: Decimal::ZERO,
        }));
    }

    // Simple equal-split allocation (other methods follow same pattern)
    let per_job = total_overhead / Decimal::from(job_count);
    let mut allocations = Vec::new();

    for job_id in &amp;job_ids {
        let li_id = new_id("li");
        conn.execute(
            "INSERT INTO job_line_items (id, job_id, category, description, quantity, unit, unit_cost, total_cost)
             VALUES (?1, ?2, 'overhead', 'Overhead allocation', 1, 'allocation', ?3, ?3)",
            rusqlite::params![li_id, job_id, per_job.to_f64().unwrap_or(0.0)],
        )?;
        allocations.push(JobAllocation {
            job_id: job_id.clone(),
            allocated_amount: per_job,
        });
    }

    let allocated = per_job * Decimal::from(job_count);
    let recovery = if total_overhead > Decimal::ZERO {
        allocated / total_overhead * Decimal::from(100)
    } else {
        Decimal::from(100)
    };

    Ok(Json(AllocationResult {
        period_start: input.period_start,
        period_end: input.period_end,
        total_overhead,
        jobs_in_period: job_count,
        allocation_method: input.method,
        allocations,
        recovery_rate: recovery,
    }))
}
```


### Migration


### `backend/migrations/006_overhead.sql`

```sql
CREATE TABLE IF NOT EXISTS overhead_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    frequency TEXT NOT NULL CHECK(frequency IN ('monthly', 'annual', 'per_job', 'one_time')),
    monthly_budget REAL NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS overhead_entries (
    id TEXT PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES overhead_categories(id),
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_overhead_entries_date ON overhead_entries(date);
CREATE INDEX idx_overhead_entries_cat ON overhead_entries(category_id);

-- Seed common overhead categories for a drain installation business
INSERT INTO overhead_categories (id, name, frequency, monthly_budget, notes) VALUES
    ('ohcat-insurance', 'General Liability Insurance', 'monthly', 500, 'GL insurance premium'),
    ('ohcat-vehicle-ins', 'Vehicle Insurance', 'monthly', 300, 'Commercial auto policy'),
    ('ohcat-workers-comp', 'Workers Compensation', 'monthly', 400, 'Workers comp premium'),
    ('ohcat-rent', 'Shop/Yard Rent', 'monthly', 800, 'Equipment storage yard'),
    ('ohcat-truck-payment', 'Truck Payments', 'monthly', 600, 'Fleet financing'),
    ('ohcat-phone', 'Phone/Internet', 'monthly', 150, 'Business phone and internet'),
    ('ohcat-accounting', 'Accounting/Bookkeeping', 'monthly', 200, 'Monthly bookkeeping service'),
    ('ohcat-marketing', 'Marketing/Advertising', 'monthly', 300, 'Google ads, yard signs, etc.'),
    ('ohcat-licenses', 'Licenses and Permits', 'annual', 50, 'Business license, contractor license'),
    ('ohcat-software', 'Software/Subscriptions', 'monthly', 100, 'Business software');
```







## Payments

Payments track every dollar in and out of a job. This domain handles:
- **Partial payments** — a job can have multiple payments (deposit, progress, final)
- **Payment methods** — cash, check, credit card, ACH, financing
- **Processing fees** — credit card transactions cost 2.5–3.5%. The system tracks
  this per payment and can recover it as a surcharge or line-item fee.
- **Refunds** — partial or full, tied to a specific payment or the job as a whole
- **Balance tracking** — outstanding balance = quoted price - sum(payments) + sum(refunds)

**Processing fee recovery model:**

When a customer pays by credit card, the business loses ~3% to Visa/Mastercard.
On a $5,000 job that's $150 gone. The owner has three options:

1. **Absorb** — eat the fee as a cost of doing business (default).
   The fee becomes a `processing_fee` line item on the job.
2. **Surcharge** — add the fee on top of the quoted price.
   The customer pays $5,150 instead. No cost to the business.
3. **Cash discount** — quote a higher price and discount for non-card payments.
   Equivalent to surcharge but framed as a discount.

The system defaults to absorbing but flags when a non-card method is used so
the owner can see how much they're saving on fees.

### Data Model


### `backend/src/payments/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{Payment, Refund, PaymentMethod};
```



### `backend/src/payments/model.rs`

```rust
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
    pub fn as_str(&amp;self) -> &amp;'static str {
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

    pub fn from_str(s: &amp;str) -> Self {
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
    pub fn has_processing_fee(&amp;self) -> bool {
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
    pub reference: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
    pub created_at: NaiveDateTime,
}

/// A refund issued against a job (and optionally a specific payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refund {
    pub id: String,
    pub job_id: String,
    pub payment_id: Option&lt;String&gt;,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub reason: String,
    pub method: String,
    /// Was the original processing fee refunded by the processor?
    pub processor_refunded_fee: bool,
    pub notes: Option&lt;String&gt;,
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
    pub payments: Vec&lt;Payment&gt;,
    pub refunds: Vec&lt;Refund&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayment {
    pub job_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub method: String,
    /// Override fee pct. If omitted, uses default for the payment method.
    pub processing_fee_pct: Option&lt;Decimal&gt;,
    /// If true, the fee is added on top (customer pays amount + fee).
    /// If false (default), fee is absorbed by the business.
    pub fee_recovered: Option&lt;bool&gt;,
    pub reference: Option&lt;String&gt;,
    pub notes: Option&lt;String&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateRefund {
    pub job_id: String,
    pub payment_id: Option&lt;String&gt;,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub reason: String,
    pub method: Option&lt;String&gt;,
    pub processor_refunded_fee: Option&lt;bool&gt;,
    pub notes: Option&lt;String&gt;,
}
```


### Routes


### `backend/src/payments/routes.rs`

```rust
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

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/payments", post(record_payment))
        .route("/api/payments/:id", get(get_payment))
        .route("/api/refunds", post(record_refund))
        .route("/api/jobs/:job_id/payments", get(job_payment_summary))
        .route("/api/payments/processing-fees/summary", get(processing_fee_summary))
}

async fn record_payment(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreatePayment&gt;,
) -> Result&lt;Json&lt;Payment&gt;, AppError&gt; {
    let id = new_id("pay");
    let method = PaymentMethod::from_str(&amp;input.method);
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
        "SELECT COUNT(*) > 0 FROM jobs WHERE id = ?1", [&amp;input.job_id], |r| r.get(0),
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
    if fee_amount > Decimal::ZERO &amp;&amp; !fee_recovered {
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
        [&amp;input.job_id],
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;Payment&gt;, AppError&gt; {
    let conn = state.db.conn();
    let payment = conn.query_row(
        "SELECT id, job_id, date, amount, method, processing_fee_pct,
                processing_fee_amount, fee_recovered, reference, notes, created_at
         FROM payments WHERE id = ?1",
        [&amp;id],
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateRefund&gt;,
) -> Result&lt;Json&lt;Refund&gt;, AppError&gt; {
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
        [&amp;input.job_id],
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
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(job_id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;JobPaymentSummary&gt;, AppError&gt; {
    let conn = state.db.conn();

    let quoted: Decimal = conn.query_row(
        "SELECT COALESCE(quoted_price, 0) FROM jobs WHERE id = ?1",
        [&amp;job_id], |r| get_decimal(r, 0),
    ).map_err(|_| AppError::NotFound(format!("Job {} not found", job_id)))?;

    // Fetch payments
    let mut stmt = conn.prepare(
        "SELECT id, job_id, date, amount, method, processing_fee_pct,
                processing_fee_amount, fee_recovered, reference, notes, created_at
         FROM payments WHERE job_id = ?1 ORDER BY date"
    )?;
    let payments: Vec&lt;Payment&gt; = stmt.query_map([&amp;job_id], |row| {
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
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;

    // Fetch refunds
    let mut stmt = conn.prepare(
        "SELECT id, job_id, payment_id, date, amount, reason, method,
                processor_refunded_fee, notes, created_at
         FROM refunds WHERE job_id = ?1 ORDER BY date"
    )?;
    let refunds: Vec&lt;Refund&gt; = stmt.query_map([&amp;job_id], |row| {
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
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;

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
        is_paid_in_full: balance &lt;= Decimal::ZERO,
        payments,
        refunds,
    }))
}

/// Summary of processing fees across all jobs — how much has been lost to processors.
async fn processing_fee_summary(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
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
```


### Migration


### `backend/migrations/009_payments.sql`

```sql
CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    method TEXT NOT NULL CHECK(method IN ('cash', 'check', 'credit_card', 'debit_card', 'ach', 'financing', 'other')),
    processing_fee_pct REAL NOT NULL DEFAULT 0,
    processing_fee_amount REAL NOT NULL DEFAULT 0,
    fee_recovered BOOLEAN NOT NULL DEFAULT 0,
    reference TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS refunds (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    payment_id TEXT REFERENCES payments(id),
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    reason TEXT NOT NULL,
    method TEXT NOT NULL,
    processor_refunded_fee BOOLEAN NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_payments_job ON payments(job_id);
CREATE INDEX idx_payments_method ON payments(method);
CREATE INDEX idx_payments_date ON payments(date);
CREATE INDEX idx_refunds_job ON refunds(job_id);
CREATE INDEX idx_refunds_payment ON refunds(payment_id);
```







## Standard Operating Procedures

SOPs are the franchise-readiness backbone. Every repeatable process gets documented
as a numbered, versioned procedure with discrete steps. When a procedure changes,
the version increments so the old version is preserved for audit.

**Key SOPs for a french drain installer:**
- Pre-job site assessment
- Trench layout and marking (call 811, locate utilities)
- Excavation procedure (by equipment type)
- Pipe installation and grading
- Backfill and compaction
- Sod/lawn restoration
- Sump pump installation
- Catch basin installation
- Post-job cleanup and inspection
- Customer walkthrough and sign-off
- Equipment daily pre-use inspection
- End-of-day equipment maintenance
- Vehicle loading checklist
- New employee onboarding
- Safety incident reporting

### Data Model


### `backend/src/sops/mod.rs`

```rust
mod model;
mod routes;

pub use routes::routes;
pub use model::{Procedure, ProcedureStep};
```



### `backend/src/sops/model.rs`

```rust
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
    pub description: Option&lt;String&gt;,
    pub estimated_duration_minutes: Option&lt;i32&gt;,
    pub required_roles: Option&lt;String&gt;,       // comma-separated roles
    pub required_equipment: Option&lt;String&gt;,    // comma-separated equipment
    pub required_materials: Option&lt;String&gt;,    // comma-separated materials
    pub safety_notes: Option&lt;String&gt;,
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
    pub notes: Option&lt;String&gt;,
}

/// A procedure with its steps included.
#[derive(Debug, Serialize)]
pub struct ProcedureWithSteps {
    #[serde(flatten)]
    pub procedure: Procedure,
    pub steps: Vec&lt;ProcedureStep&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateProcedure {
    pub title: String,
    pub category: String,
    pub description: Option&lt;String&gt;,
    pub estimated_duration_minutes: Option&lt;i32&gt;,
    pub required_roles: Option&lt;String&gt;,
    pub required_equipment: Option&lt;String&gt;,
    pub required_materials: Option&lt;String&gt;,
    pub safety_notes: Option&lt;String&gt;,
    pub steps: Vec&lt;CreateStep&gt;,
}

#[derive(Debug, Deserialize)]
pub struct CreateStep {
    pub instruction: String,
    pub is_checkpoint: Option&lt;bool&gt;,
    pub notes: Option&lt;String&gt;,
}
```


### Routes


### `backend/src/sops/routes.rs`

```rust
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use crate::{AppState, error::AppError, types::new_id};
use super::model::*;

pub fn routes() -> Router&lt;Arc&lt;AppState&gt;&gt; {
    Router::new()
        .route("/api/sops", get(list_procedures).post(create_procedure))
        .route("/api/sops/:id", get(get_procedure).put(update_procedure))
}

async fn list_procedures(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
) -> Result&lt;Json&lt;Vec&lt;Procedure&gt;&gt;, AppError&gt; {
    let conn = state.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, category, version, description, estimated_duration_minutes,
                required_roles, required_equipment, required_materials, safety_notes,
                active, created_at, updated_at
         FROM procedures WHERE active = 1 ORDER BY category, title"
    )?;
    let procs = stmt.query_map([], |row| {
        Ok(Procedure {
            id: row.get(0)?,
            title: row.get(1)?,
            category: row.get(2)?,
            version: row.get(3)?,
            description: row.get(4)?,
            estimated_duration_minutes: row.get(5)?,
            required_roles: row.get(6)?,
            required_equipment: row.get(7)?,
            required_materials: row.get(8)?,
            safety_notes: row.get(9)?,
            active: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;
    Ok(Json(procs))
}

async fn create_procedure(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Json(input): Json&lt;CreateProcedure&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let id = new_id("sop");
    let conn = state.db.conn();
    conn.execute(
        "INSERT INTO procedures (id, title, category, version, description, estimated_duration_minutes,
                                 required_roles, required_equipment, required_materials, safety_notes)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            id, input.title, input.category, input.description,
            input.estimated_duration_minutes, input.required_roles,
            input.required_equipment, input.required_materials, input.safety_notes,
        ],
    )?;

    for (i, step) in input.steps.iter().enumerate() {
        let step_id = new_id("step");
        conn.execute(
            "INSERT INTO procedure_steps (id, procedure_id, step_number, instruction, is_checkpoint, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![step_id, id, (i + 1) as i32, step.instruction, step.is_checkpoint.unwrap_or(false), step.notes],
        )?;
    }

    Ok(Json(serde_json::json!({ "id": id, "version": 1, "steps_created": input.steps.len() })))
}

async fn get_procedure(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
) -> Result&lt;Json&lt;ProcedureWithSteps&gt;, AppError&gt; {
    let conn = state.db.conn();
    let procedure = conn.query_row(
        "SELECT id, title, category, version, description, estimated_duration_minutes,
                required_roles, required_equipment, required_materials, safety_notes,
                active, created_at, updated_at
         FROM procedures WHERE id = ?1",
        [&amp;id],
        |row| Ok(Procedure {
            id: row.get(0)?,
            title: row.get(1)?,
            category: row.get(2)?,
            version: row.get(3)?,
            description: row.get(4)?,
            estimated_duration_minutes: row.get(5)?,
            required_roles: row.get(6)?,
            required_equipment: row.get(7)?,
            required_materials: row.get(8)?,
            safety_notes: row.get(9)?,
            active: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("Procedure {} not found", id)))?;

    let mut stmt = conn.prepare(
        "SELECT id, procedure_id, step_number, instruction, is_checkpoint, notes
         FROM procedure_steps WHERE procedure_id = ?1 ORDER BY step_number"
    )?;
    let steps = stmt.query_map([&amp;id], |row| {
        Ok(ProcedureStep {
            id: row.get(0)?,
            procedure_id: row.get(1)?,
            step_number: row.get(2)?,
            instruction: row.get(3)?,
            is_checkpoint: row.get(4)?,
            notes: row.get(5)?,
        })
    })?.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?;

    Ok(Json(ProcedureWithSteps { procedure, steps }))
}

async fn update_procedure(
    State(state): State&lt;Arc&lt;AppState&gt;&gt;,
    Path(id): Path&lt;String&gt;,
    Json(input): Json&lt;CreateProcedure&gt;,
) -> Result&lt;Json&lt;serde_json::Value&gt;, AppError&gt; {
    let conn = state.db.conn();

    // Bump version, replace steps
    let current_version: i32 = conn.query_row(
        "SELECT version FROM procedures WHERE id = ?1",
        [&amp;id], |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("Procedure {} not found", id)))?;

    let new_version = current_version + 1;
    conn.execute(
        "UPDATE procedures SET title=?2, category=?3, version=?4, description=?5,
                estimated_duration_minutes=?6, required_roles=?7, required_equipment=?8,
                required_materials=?9, safety_notes=?10, updated_at=datetime('now')
         WHERE id = ?1",
        rusqlite::params![
            id, input.title, input.category, new_version, input.description,
            input.estimated_duration_minutes, input.required_roles,
            input.required_equipment, input.required_materials, input.safety_notes,
        ],
    )?;

    // Replace steps
    conn.execute("DELETE FROM procedure_steps WHERE procedure_id = ?1", [&amp;id])?;
    for (i, step) in input.steps.iter().enumerate() {
        let step_id = new_id("step");
        conn.execute(
            "INSERT INTO procedure_steps (id, procedure_id, step_number, instruction, is_checkpoint, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![step_id, id, (i + 1) as i32, step.instruction, step.is_checkpoint.unwrap_or(false), step.notes],
        )?;
    }

    Ok(Json(serde_json::json!({ "id": id, "version": new_version })))
}
```


### Migration


### `backend/migrations/008_sops.sql`

```sql
CREATE TABLE IF NOT EXISTS procedures (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    estimated_duration_minutes INTEGER,
    required_roles TEXT,
    required_equipment TEXT,
    required_materials TEXT,
    safety_notes TEXT,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS procedure_steps (
    id TEXT PRIMARY KEY,
    procedure_id TEXT NOT NULL REFERENCES procedures(id),
    step_number INTEGER NOT NULL,
    instruction TEXT NOT NULL,
    is_checkpoint BOOLEAN NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX idx_procedures_category ON procedures(category);
CREATE INDEX idx_steps_procedure ON procedure_steps(procedure_id);
```





## Shared Configuration

### EditorConfig


### `.editorconfig`

```
root = true

[*]
indent_style = space
indent_size = 4
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.{yml,yaml,toml}]
indent_size = 2

[*.sql]
indent_size = 4
```


