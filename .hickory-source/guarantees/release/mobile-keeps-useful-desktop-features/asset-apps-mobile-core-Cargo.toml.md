<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Link only storage and peer services into the mobile core

Link only storage and peer services into the mobile core. This literal construction preserves build configuration exactly.

<mobile:entry path="apps/mobile/core/Cargo.toml">
[package]
name = "hickory-mobile-core"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
base64 = "0.22"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "io-util", "time"] }
hickory-fleet = { path = "../../../crates/hickory-fleet" }
hickory-peer = { path = "../../../crates/hickory-peer" }
iroh = { version = "~1.0.3", default-features = false, features = ["tls-ring"] }
futures-util = { version = "0.3", features = ["sink"] }
tokio-tungstenite = { version = "0.24", default-features = false, features = ["handshake"] }
hyper = { version = "1", features = ["client", "http1"] }
hyper-util = { version = "0.1", features = ["tokio"] }
http-body-util = "0.1"
bytes = "1"

[dev-dependencies]
axum = { version = "0.8", features = ["ws"] }
</mobile:entry>
</mobile:doc>
