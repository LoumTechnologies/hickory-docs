






This template generates a `justfile` for building all enabled platforms using
the [just](https://just.systems/) command runner.

**Why just?**
- Simple, human-readable syntax
- Cross-platform support
- Dependency management between tasks
- Environment variable integration
- No complex makefile syntax








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
| `project_name` | BlueIronCube | Xcode project name (PascalCase) |
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
async fn main() -> Result&lt;()&gt; {
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

# devenv
.devenv/
dev-data/
dev-backups/
.env
*.ts.net.crt
*.ts.net.key

# Xcode (generated by xcodegen)
*.xcodeproj
*.xcworkspace
DerivedData/
build/
xcuserdata/

# Android
android/.gradle/
android/build/
android/app/build/
android/local.properties
*.apk
*.aab

# .NET / Avalonia
desktop/**/bin/
desktop/**/obj/
*.user
*.suo

# Node / Web
web/node_modules/
web/dist/
browser-extension/node_modules/
browser-extension/dist/
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
RUN mkdir src &amp;&amp; echo "fn main() {}" > src/main.rs &amp;&amp; echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update &amp;&amp; apt-get install -y --no-install-recommends curl &amp;&amp; rm -rf /var/lib/apt/lists/*
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
  "name": "BlueIronCube",
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






















# Cross-Platform Project Scaffolding

This template generates a multi-platform project with a consistent structure across:

- **SwiftUI** — iOS, iPadOS, and macOS using Apple's declarative UI framework
- **Jetpack Compose** — Android with Kotlin and modern declarative UI
- **Avalonia** — Cross-platform desktop (Windows, macOS, Linux) with .NET
- **React SPA** — Single-page web application with TypeScript
- **Browser Extension** — Chrome/Firefox extension with React TypeScript
- **Rust Backend** — API server using Axum for high performance

Each platform is behind a feature flag, so you only generate what you need.

## Usage

Generate a project with specific platforms:

```bash
hick --param project_name=MyApp --features swiftui,backend scaffolding/
```

Generate all platforms:

```bash
hick --param project_name=MyApp --features all scaffolding/
```

---

## Configuration Variables

These variables control project metadata and can be overridden via `--param`:












| Variable | Default | Purpose |
|----------|---------|---------|
| `project_name` | BlueIronCube | Project name (used in file paths, class names) |
| `bundle_id` | com.acmedrainage.blueironcube | Reverse-domain bundle identifier |
| `organization` | Example Inc | Company/organization name for copyright |
| `min_ios_version` | 17.0 | Minimum iOS deployment target |
| `min_macos_version` | 14.0 | Minimum macOS deployment target |
| `min_android_sdk` | 26 | Minimum Android SDK level |
| `target_android_sdk` | 35 | Target Android SDK level |
| `dotnet_version` | 8.0 | .NET version for Avalonia |
| `rust_edition` | 2024 | Rust edition year |
| `node_version` | 20 | Node.js version for web projects |

---

## Feature Definitions

Features control which platforms are generated. They can be combined freely.














**Build system:**



**Meta-features** for convenience:







**Verify commands** run after each matrix combination to catch scaffolding errors:









| Feature | Description |
|---------|-------------|
| `swiftui` | iOS, iPadOS, macOS with SwiftUI |
| `compose` | Android with Jetpack Compose |
| `avalonia` | Desktop with Avalonia (.NET) |
| `spa` | React TypeScript web app |
| `extension` | Browser extension |
| `backend` | Rust API server |
| `rust-frontend-lib` | Shared Rust library (WASM + FFI) |
| `sqlite` | SQLite database in Rust library (requires rust-frontend-lib) |
| `iroh` | Iroh P2P networking in Rust library (requires rust-frontend-lib) |
| `decentralized-auth` | Decentralized identity and authentication (requires rust-frontend-lib) |
| `event-sourcing` | Event sourcing with CRDT conflict resolution (requires rust-frontend-lib, sqlite) |
| `iroh-relay` | Iroh relay server for NAT traversal |
| `just` | Justfile for build automation |
| `all` | All platforms (not build systems) |
| `mobile` | SwiftUI + Compose |
| `desktop` | SwiftUI + Avalonia |
| `web` | SPA + Extension |
| `native` | Native platforms + Rust library |

---

## Substitution Rules

The project name is substituted throughout all generated files using case variants:



This means `BlueIronCube`, `blueIronCube`, `blue_iron_cube`, `BLUE_IRON_CUBE`, and `blue-iron-cube`
in the template files will all be replaced with the corresponding case variant of
your chosen project name.

---

## Shared Content Blocks

### License

We use MIT License by default. This cut block is referenced elsewhere without
appearing directly in any file output:



### Common Gitignore Patterns

These patterns are shared across all platforms:



---

## Root Project Files

### LICENSE

The license file at the project root:


### `LICENSE`

```
MIT License

Copyright (c) 2024 

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.```


### README.md

The main README is dynamically generated based on which features are enabled,
so users always see accurate platform-specific instructions:


### `README.md`

```markdown
# BlueIronCube

A cross-platform application by Example Inc.

## Platforms












## License

MIT License - see [LICENSE](LICENSE) for details.
```


### .gitignore

The gitignore combines common patterns with platform-specific ones based on
enabled features:


### `.gitignore`

```
# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Build outputs
dist/
build/
out/








```


---

## Platform Templates

The platform-specific files are defined in separate `.hick` files for maintainability.
Each platform template is guarded by its feature flag.

See the individual platform documentation:
- [SwiftUI Platform](platforms/swiftui.hick) — Apple platforms with XcodeGen
- [Compose Platform](platforms/compose.hick) — Android with Kotlin DSL Gradle
- [Avalonia Platform](platforms/avalonia.hick) — .NET cross-platform desktop
- [SPA Platform](platforms/spa.hick) — React + Vite web application
- [Extension Platform](platforms/extension.hick) — Browser extension
- [Backend Platform](platforms/backend.hick) — Rust Axum server
- [Rust Frontend Library](platforms/rust-frontend-lib.hick) — Shared Rust library with WASM + FFI
- [Iroh Relay](platforms/iroh-relay.hick) — Iroh relay server for P2P networking

Build system template:
- [Just Build System](platforms/just.hick) — Justfile for task automation



























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







This is a demo hick document for testing the LSP server.
It defines a simple Rust program and a Python script.




### `Cargo.toml`

```toml

[package]
name = "hick-demo"
version = "0.1.0"
edition = "2021"
```



### `src/main.rs`

```rust

fn main() {
    let message = "Hello from hick!";
    println!("{}", message);

    let numbers: Vec<i32> = (1..=10).collect();
    for n in &numbers {
        if n % 2 == 0 {
            println!("{} is even", n);
        }
    }
}
```



### `hello.py`

```python

def greet(name: str) -> str:
    return f"Hello, {name}!"

if __name__ == "__main__":
    print(greet("world"))
```



### `config.json`

```json

{
    "name": "hick-demo",
    "version": "0.1.0",
    "greeting": "Hello from hick!"
}
```


Try editing this file in Kate — if the LSP is working, you should see:
- Parse errors highlighted as diagnostics if you break the hick syntax
- No errors on valid documents like this one

Try introducing an error, for example delete the closing tag on one
of the hick:file blocks, and you should see a diagnostic appear.


































$ echo "Built by hick" > /out/stamp.txt


$ cat /out/stamp.txt
$ cat /out/stamp.txt





$ cat /out/stamp.txt && echo "Fork inherited builder state"
$ cat /out/stamp.txt &amp;&amp; echo "Fork inherited builder state"







### `docs/hick-guide.md`

```markdown

# The Hick Language

Hick is a document format for describing secure, containerised workflows.
A `.hick` file is XML that uses a single namespace to define containers,
capabilities, execution steps, and file outputs — all validated as a
directed acyclic graph before anything runs.

Namespace URI: `http://www.hickorydocs.com/1.0`

---

## 1. Document Structure

Every hick file is an XML document whose root element binds a prefix to
the hick namespace. The conventional prefix is `hick:`, but any prefix
works as long as the xmlns points to the right URI.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  
</hick:doc>
```

Only tags with the bound prefix are parsed as structured elements.
Everything else — including `&amp;`, `&lt;`, `&gt;` — is raw text, no
escaping required.

---

## 2. Containers and Capabilities

Containers declare isolated execution environments with explicit
capability grants. The security model is deny-by-default when a
`deny` rule is present.

```xml
<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-write="/output/*" />
</hick:container>
```

### Network rules

| Rule | Effect |
|------|--------|
| `<hick:allow network="host:port" />` | Permit traffic to host:port |
| `<hick:allow network="*:443" />` | Wildcard port or host |
| `<hick:deny network="*" />` | Deny all not explicitly allowed |

Without a `deny` rule, all network traffic is permitted. Once a `deny`
is present, only explicit `allow` rules are honoured.

### File rules

| Rule | Effect |
|------|--------|
| `<hick:allow file-read="/path/*" />` | Permit reading under path |
| `<hick:allow file-write="/path/*" />` | Permit writing (implies read) |

### Secrets

Secrets are injected as environment variables, sourced from an external
secrets provider (age-encrypted files by default).

```xml
<hick:container name="deploy" image="alpine">
  <hick:secret name="PULUMI_TOKEN" from="pulumi-token" />
</hick:container>
```

---

## 3. Execution

The `exec` tag runs commands inside a container. Multiple execs in the
same container are sequenced automatically (container-state dependency).

```xml
<hick:exec container="reporter" image="alpine">
apk add curl
</hick:exec>

<hick:exec container="reporter">
curl --version
</hick:exec>
```

If the container hasn't been declared with a `container` tag, providing
`image` on the first exec creates it implicitly.

### Volumes

Shared volumes let containers exchange data. The first exec to mount a
volume is treated as the writer; subsequent mounters become readers,
creating a data-flow dependency.

```xml
<hick:volume name="shared-output" />

<hick:exec container="writer" mount="shared-output:/output">
echo "data" &gt; /output/result.txt
</hick:exec>

<hick:exec container="reader" mount="shared-output:/input">
cat /input/result.txt
</hick:exec>
```

---

## 4. File Outputs

The `file` tag declares an output file. Its content is a mix of literal
text, exec placeholders, and paste references.

```xml
<hick:file path="report.md">
# Report

Build output:
<hick:exec container="builder">make build</hick:exec>

Version: <hick:paste select="#version" />
</hick:file>
```

Exec tags inside a file render the full container transcript — every
command prefixed with `$ ` followed by its output. In dry-run mode only
the commands appear; in live mode the real output follows each command.
Paste tags are resolved from matching copy/cut blocks.

---

## 5. Copy, Cut, and Paste

Content reuse across the document uses an id-based clipboard.

- **copy** — stores content and keeps it in output
- **cut** — stores content but removes it from output
- **paste** — inserts stored content by id selector

```xml
<hick:copy id="version">2.0.0</hick:copy>

<hick:cut id="internal-notes">
This text is available to paste but not in the output.
</hick:cut>

<hick:file path="manifest.txt">
Version: <hick:paste select="#version" />
Notes: <hick:paste select="#internal-notes" />
</hick:file>
```

Copy/paste works across documents when multiple `.hick` files are
processed in the same pipeline run.

---

## 6. Fork and Attenuate

### Fork

Fork clones a container's state into a new container, optionally adding
restrictions. The fork depends on the source container's last exec.

```xml
<hick:container name="base" image="ubuntu:22.04" />

<hick:exec container="base">
apt-get update &amp;&amp; apt-get install -y build-essential
</hick:exec>

<hick:fork from="base" to="analyzer">
  <hick:deny network="*" />
</hick:fork>

<hick:exec container="analyzer">
./run-analysis
</hick:exec>
```

### Attenuate

Attenuate adds restrictions to an existing container without cloning.
Capabilities can only be narrowed, never escalated.

```xml
<hick:attenuate container="sandbox">
  <hick:deny network="*" />
</hick:attenuate>
```

---

## 7. Confirmation Gates

The `confirm` tag pauses execution and requires user approval before
proceeding.

```xml
<hick:confirm message="Deploy to production?" />
```

---

## 8. Variables

Variables let you define reusable values and reference them in file outputs.

### Declaring variables

```xml
<hick:var name="version">2.0.0</hick:var>
<hick:var name="env">production</hick:var>
```

### Using variables in file output

Inside `<hick:file>`, the `<hick:val>` tag interpolates a variable:

```xml
<hick:file path="banner.txt">
Application v<hick:val name="version" />
Environment: <hick:val name="env" />
</hick:file>
```

### CLI parameter overrides

Variables can be overridden from the command line with `--param`:

```
hick --param env=staging file.hick
```

CLI params take priority over `<hick:var>` declarations, which in turn
take priority over `_hick.yml` defaults.

---

## 9. Conditional Content

Nodes can be included or excluded based on variable values.

### The `<hick:when>` tag

Wraps a block of content that is included only when the condition is true.
When true, the wrapper is removed and children are promoted into the
parent. When false, the entire block is dropped.

```xml
<hick:var name="mode">prod</hick:var>

<hick:when test="mode=prod">
  <hick:exec container="deploy">./deploy.sh</hick:exec>
</hick:when>

<hick:when test="!debug">
  <hick:file path="release.txt">production build</hick:file>
</hick:when>
```

### The `when` attribute

Any hick tag can carry a `when` attribute for inline conditional logic:

```xml
<hick:exec container="linter" when="lint">
eslint src/
</hick:exec>

<hick:file path="debug.log" when="debug">
Debug output here
</hick:file>
```

### Condition syntax

| Condition | Meaning |
|-----------|---------|
| `var_name` | True if defined and non-empty |
| `!var_name` | True if undefined or empty |
| `var_name=value` | True if equals value |
| `var_name!=value` | True if not equal to value |

---

## 10. Exec Show Modes

When an `<hick:exec>` appears inside a `<hick:file>`, the `show`
attribute controls which parts of the transcript are rendered:

| Value | Renders |
|-------|---------|
| *(default)* | Commands (`$ ...`) and output |
| `command` | Commands only |
| `output` | Output only |
| `none` | Nothing (side-effect only) |

```xml
<hick:file path="report.md">
Full transcript:
<hick:exec container="builder">make build</hick:exec>

Output only:
<hick:exec container="builder" show="output">make test</hick:exec>

Side-effect (no output in file):
<hick:exec container="builder" show="none">make clean</hick:exec>
</hick:file>
```

---

## 11. Volume Access Rules

Volumes support explicit access rules that control which containers can
read or write. This replaces the default heuristic (first mounter writes,
others read) with fine-grained permissions.

```xml
<hick:volume name="project" output="src/MyApi/">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="linter" read="**" />
  <hick:allow container="patcher" read="**" write="Controllers/**" />
</hick:volume>
```

### Volume kinds

| Declaration | Behaviour |
|-------------|-----------|
| `<hick:volume name="x" />` | Ephemeral scratch (no host mapping) |
| `<hick:volume name="x" input="." />` | Snapshot of host directory (read source) |
| `<hick:volume name="x" output="dist/" />` | Contents become pipeline file outputs |
| `<hick:volume name="x" input="." output="." />` | Read existing state AND write updates |

When access rules are present, the DAG builder uses them to determine
writer/reader edges instead of the first-mounter heuristic. Multiple
writers on the same volume are sequenced automatically.

---

## 12. Project Configuration

A `_hick.yml` file in the project root provides defaults for the pipeline.
Hick searches for it by walking up directories (stopping at a `.git`
boundary).

```yaml
files:
  - docs/*.hick
  - reference.hick

vars:
  version: "2.0.0"
  env: staging

defaults:
  image: alpine:3.20
  images-dir: /opt/hick/images

output-dir: dist/

secrets:
  key-file: /etc/hick/key.txt
  secrets-dir: /etc/hick/secrets
```

With a config file, you can simply run `hick` with no arguments — files
and defaults are read from `_hick.yml`. CLI flags and `--param` values
override config file values.

---

## 13. Execution Caching

The `--cache` flag caches exec results to avoid re-running containers
when inputs haven't changed. Cache keys are SHA-256 hashes of the
container image, capabilities, command text, and secret names.

```
hick --cache file.hick          # Cache results, reuse on match
hick --cache --freeze file.hick # Require cached results, never execute
hick --clear-cache file.hick    # Clear cache before running
```

Cached transcripts are stored in `.hick-cache/transcripts/`. In freeze
mode, any uncached exec step is an error — useful in CI to ensure
reproducible builds.

---

## 14. Multi-Stage Pipeline

When a pipeline produces `.hick` files as output, they are automatically
fed as sources to a subsequent pipeline stage. This enables containers
to generate hick documents that drive further execution.

```xml

<hick:container name="scaffolder" image="alpine" />
<hick:exec container="scaffolder">
cat &gt; /output/build.hick &lt;&lt;'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="generated.txt">built by scaffolder</hick:file>
</hick:doc>
EOF
</hick:exec>
```

Rules:
- `.hick` files are removed from the final output (they are intermediate)
- The pipeline repeats up to `max_stages` times (default 3)
- If `.hick` files are still being produced at the limit, an error is raised
- Stage 1 files get `HickFile` provenance, stage 2+ get `GeneratedHick`

---

## 15. Watch Mode (`hick up`)

The `hick up` command runs the pipeline and watches source files for
changes, re-running automatically with merge-aware snapshots.

```
hick up docs/guide.hick

Options:
  --store auto|git|builtin   Version store backend (default: auto)
  --branch <name>            Snapshot branch (default: main)
  --merge-api <url>          LLM merge API for conflict resolution
  --max-stages <n>           Multi-stage limit (default: 3)
```

### How it works

1. Initial pipeline run produces output files
2. Output is merged with any user edits via three-way merge
3. A snapshot is committed to the version store
4. File watcher monitors `.hick` sources (debounced 100ms)
5. On change: re-run pipeline, merge, snapshot, write outputs

### Three-way merge

The merge system tracks three versions of each file:
- **Base** — the last snapshot's version
- **Generated** — the new pipeline output
- **Edited** — what's currently on disk (may have user edits)

| Base vs Edited | Base vs Generated | Result |
|---------------|-------------------|--------|
| Same | Different | Use generated (no user edits) |
| Different | Same | Keep edited (no pipeline changes) |
| Both different | — | Delegate to merge strategy |

Files with `HickFile` provenance always use the generated version
(users edit the `.hick` source, not the output).

### Store backends

| Backend | Storage | When to use |
|---------|---------|-------------|
| `auto` | Detects `.git` → git, else builtin | Default |
| `git` | Git plumbing at `.hick/git/` | When you want git-compatible history |
| `builtin` | Content-addressed files at `.hick/store/` | Standalone projects |

---

## 16. Information-Flow DAG

Hick validates the entire document as a directed acyclic graph before
execution. Dependencies are inferred automatically:

| Dependency type | Trigger |
|-----------------|---------|
| Container state | Multiple execs in same container |
| Volume flow | Writer then reader on same volume |
| Volume access rules | Explicit read/write permissions on volumes |
| Copy/paste | Producer then consumer of an id |
| Fork | Source container's last exec before fork target |
| Attenuate | Barrier between execs in same container |

Cycles are rejected at validation time with clear error messages.

---

## 17. Capability Tokens

Each container receives a macaroon-based capability token at pipeline
start. Tokens encode the container's allowed operations and can be
attenuated (restricted) but never escalated. This enables secure
delegation — a container can pass a restricted version of its own
token to a subprocess.

---

## 18. Custom Namespace Prefix

The parser detects the namespace prefix from the `xmlns:` declaration.
This means you can use any prefix, not just `hick:`:

```xml
<x:doc xmlns:x="http://www.hickorydocs.com/1.0">
  <x:file path="out.txt">hello</x:file>
</x:doc>
```

This is how this guide itself is written — using the `h:` prefix so
that `hick:` examples in the text body are treated as literal content
rather than parsed tags.

---

## 19. Running Hick

```
hick [file.hick ...]               Single-shot pipeline (default)
hick run [file.hick ...] [options]  Explicit single-shot mode
hick up <file.hick> [options]       Watch mode with merge snapshots

Common options:
  --config <path>          Path to _hick.yml (default: auto-discover)
  --param key=value        Set/override a variable
  --key-file <path>        Age identity file
  --secrets-dir <path>     Secrets directory
  --images-dir <path>      Pre-converted .wasm images
  --dry-run                Produce placeholders instead of running containers
  --cache                  Cache exec results, reuse on match
  --freeze                 Require cached results, never re-execute
  --clear-cache            Clear cache before running
  -v, --verbose            Enable verbose logging

Watch mode options (hick up):
  --store auto|git|builtin Version store backend
  --branch <name>          Snapshot branch name
  --merge-api <url>        LLM merge API endpoint
  --max-stages <n>         Multi-stage pipeline limit
```

If no files are specified and a `_hick.yml` exists, files are read from
the config. Output files declared with `<hick:file>` are written to disk
relative to the current directory.

---

## 20. Live Demo

This guide is itself a `.hick` file (`docs/hick-guide.hick`). It uses
the `h:` prefix so that `hick:` code examples appear as literal text.

The file defines two containers with restricted capabilities, a shared
volume, a fork, and exec steps that exercise the full pipeline — DAG
validation, capability minting, volume flow, forking, and copy/paste —
every time the guide is built.

### Containers used to build this guide

```
guide-builder   (alpine:3.20, network denied, file-write /out/*)
guide-verifier  (alpine:3.20, network denied, file-read  /out/*)
guide-forked    (forked from guide-builder, network denied)
```

### Container transcript

The following block is an exec tag rendered by the pipeline — it shows
the full transcript for the referenced container (commands and output):

> $ cat /out/stamp.txt
$ cat /out/stamp.txt


### Forked container transcript

The `guide-forked` container is a fork of `guide-builder`. It inherits
the builder's filesystem state (including `/out/stamp.txt`) without
re-running the original commands on the original container. The fork
also adds a network deny rule, further restricting the builder's
capabilities:

> $ cat /out/stamp.txt && echo "Fork inherited builder state"
$ cat /out/stamp.txt &amp;&amp; echo "Fork inherited builder state"


### Pasted build command

The builder's command is captured with `<hick:copy>` and pasted here:

> `echo "Built by hick" > /out/stamp.txt`

### Variables and conditionals

This guide defines variables and uses conditional rendering:

> Guide version: 0.2.0

The following line only appears because `show-features` is defined:

> Feature list is enabled via conditional rendering.

### Pipeline features exercised

- **Namespace prefix detection** — `h:` prefix bound via xmlns
- **Container capabilities** — network deny, file allow
- **Volume flow** — `guide-builder` writes, `guide-verifier` reads
- **DAG validation** — volume dependency enforces ordering
- **Fork** — `guide-forked` clones builder state with added restrictions
- **Copy/paste** — content extracted from exec, pasted into output
- **Cut/paste** — namespace URI pasted from a cut block
- **Variables** — `guide-version` declared with `<hick:var>`, rendered with `<hick:val>`
- **Conditional content** — `<hick:when>` block controlled by `show-features` variable
- **File output** — this entire document
```




















## Shared Configuration

These files are shared across all platforms.

### EditorConfig

EditorConfig ensures consistent formatting across different editors and IDEs.
This is especially important for cross-platform projects where team members
may use different tools:




### `.editorconfig`

```
# EditorConfig helps maintain consistent coding styles
root = true

[*]
indent_style = space
indent_size = 2
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.md]
trim_trailing_whitespace = false

[*.{rs,swift}]
indent_size = 4

[*.{kt,java}]
indent_size = 4

[*.cs]
indent_size = 4

[Makefile]
indent_style = tab
```


Language-specific indentation:
- **2 spaces** — JavaScript, TypeScript, JSON, YAML (web ecosystem standard)
- **4 spaces** — Rust, Swift, Kotlin, Java, C# (native ecosystem standard)
- **Tabs** — Makefiles (required by make)

### GitHub Actions CI

The CI workflow runs builds and tests for all enabled platforms. Each platform
gets its own job for parallel execution:



CI configuration highlights:
- **Parallel jobs** — Each platform builds independently
- **Caching** — npm and Rust dependencies are cached for faster builds
- **Platform-appropriate runners** — macOS for Apple, Ubuntu for others

