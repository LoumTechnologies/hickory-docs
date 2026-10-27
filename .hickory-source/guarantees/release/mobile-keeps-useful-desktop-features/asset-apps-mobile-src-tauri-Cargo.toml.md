<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Build a thin Tauri v2 mobile host

Build a thin Tauri v2 mobile host. This literal construction preserves build configuration exactly.

<mobile:entry path="apps/mobile/src-tauri/Cargo.toml">
[package]
name = "hickory-mobile"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "hickory_mobile_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
hickory-mobile-core = { path = "../core" }
hickory-fleet = { path = "../../../crates/hickory-fleet" }
hickory-peer = { path = "../../../crates/hickory-peer" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync"] }
</mobile:entry>
</mobile:doc>
