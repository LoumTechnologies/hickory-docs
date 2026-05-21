

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
| `bundle_id` | com.acmedrainage.blueironcube | iOS app bundle identifier |
| `min_ios_version` | 17.0 | Minimum iOS version |
| `url_scheme` | blueironcube | URL scheme for devenv launch |
| `project_name` | MyProject | Xcode project name (PascalCase) |
| `min_android_sdk` | 26 | Minimum Android SDK version |
| `target_android_sdk` | 35 | Target Android SDK version |
| `dotnet_version` | 8.0 | .NET target framework version |
| `node_version` | 20 | Node.js version for web platforms |

---

## Feature Definitions

Each feature represents a business domain. The `jobs` domain is the core that
most other domains connect to. Enable only what you need.

**Core domains:**









**Advanced domains:**





**Platform features:**







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
| `swiftui` | SwiftUI iOS field operations app | — |
| `compose` | Jetpack Compose Android field operations app | — |
| `avalonia` | Avalonia desktop office management app | — |
| `spa` | React TypeScript web dashboard | — |
| `extension` | Browser extension for quick job lookup | — |
| `all` | Everything | all domains |
| `core` | Jobs + payments + customers + crew + inventory | |
| `financial` | Jobs + payments + overhead + estimating | |
| `mobile` | Mobile platforms | `swiftui compose` |
| `desktop` | Desktop platforms | `avalonia` |
| `web` | Web platforms | `spa extension` |
| `frontends` | All frontend platforms | all platform features |

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


[dev-dependencies]
axum-test = "15"
```


### Entry Point


### `backend/src/main.rs`

```rust
use anyhow::Result;
use blue_iron_cube::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
```


### Library Core


### `backend/src/lib.rs`

```rust










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

pub async fn run() -> Result<()> {
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

async fn health() -> &'static str {
    "OK"
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/health", axum::routing::get(health));











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
pub fn new_id(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::now_v7())
}

/// Read a Decimal from a rusqlite row column (stored as REAL in SQLite).
pub fn get_decimal(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Decimal> {
    let val: f64 = row.get(idx)?;
    Ok(Decimal::from_f64(val).unwrap_or_default())
}

/// Read an optional Decimal from a rusqlite row column.
pub fn get_decimal_opt(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Option<Decimal>> {
    let val: Option<f64> = row.get(idx)?;
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
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database lock poisoned")
    }

    pub fn run_migrations(&self) -> Result<()> {
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
        let mut entries: Vec<_> = std::fs::read_dir(migration_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let already_applied: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )?;
            if !already_applied {
                let sql = std::fs::read_to_string(entry.path())?;
                conn.execute_batch(&sql)?;
                conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [&name])?;
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
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
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

# OS
.DS_Store
Thumbs.db

# devenv
.devenv/
.env

# Generated by hick pipeline
blue-iron-cube/output/
scaffolding/output/
TEMPLATE.md

# Agent runtime state
.hick/
.copper-ledge/
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

Platform templates:

- [SwiftUI](platforms/swiftui.hick) — iOS field operations app
- [Compose](platforms/compose.hick) — Android field operations app
- [Avalonia](platforms/avalonia.hick) — Desktop office management app
- [SPA](platforms/spa.hick) — React TypeScript web dashboard
- [Extension](platforms/extension.hick) — Browser extension for quick job lookup

---

## Infrastructure

### Dockerfile


### `backend/Dockerfile`

```
FROM rust:slim-bookworm AS builder
RUN rustup default nightly
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/blue-iron-cube /app/blue-iron-cube
COPY --from=builder /app/migrations /app/migrations
RUN mkdir -p /app/data
EXPOSE 3000
CMD ["/app/blue-iron-cube"]
```


### Docker Compose


### `docker-compose.yml`

```yaml
x-devenv:
  version: "1"

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "${BACKEND_PORT:-3000}:3000"
    volumes:
      - ./dev-data/backend:/app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 5s
      timeout: 3s
      retries: 10
      start_period: 10s
    environment:
      - RUST_LOG=blue_iron_cube=debug,tower_http=debug
    x-devenv:
      portEnvVar: BACKEND_PORT
      healthCheck: /health
      healthCheckProtocol: http
      healthTimeout: 60
      projectPath: ./backend
      command: cargo run
      developerUrl: /health

```


### Environment


### `.env.example`

```
BACKEND_PORT=3000
```


### Host Prerequisites


### `devenv-packages.yml`

```yaml
packages:
  - name: docker
    check: docker --version
    install: "Install Docker Desktop from https://docker.com"
  - name: rust-nightly
    check: rustup show active-toolchain | grep nightly
    install: "rustup default nightly"
  - name: cargo-watch
    check: cargo watch --version
    install: "cargo install cargo-watch"
```


### App Launchers


### `.devenv/launchers.yml`

```yaml
launchers:
```


### Devcontainer


### `.devcontainer/devcontainer.json`

```json
{
  "name": "MyProject",
  "dockerComposeFile": "../docker-compose.yml",
  "service": "backend",
  "workspaceFolder": "/app",
  "forwardPorts": [3000],
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer"
      ]
    }
  }
}
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














