<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `df6b7c2088c5994559898898c82f3bf868643d73b6199b99a090911dd4fad1f7`.

<book:entry path="crates/hickory-agent/Cargo.toml">
[package]
name = "hickory-agent"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
description = "Shared model transports, document tools and session evidence for Hickory agents"
license.workspace = true

[dependencies]
anyhow = "1"
sha2 = "0.10"
async-trait = "0.1"
bytes = "1"
chrono = { version = "0.4", default-features = false, features = ["clock"] }
futures = "0.3"
hick-exec.workspace = true
hick-lang.workspace = true
hick-project-env = { path = "../hick-project-env" }
hick-observe = { path = "../hick-observe" }
hick-literate = { path = "../hick-literate" }
hickory-executor = { path = "../hickory-executor" }
hickory-lineage = { path = "../hickory-lineage" }
log = "0.4"
reqwest = { version = "0.12", default-features = false, features = ["charset", "http2", "json", "stream", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }

[dev-dependencies]
hick-blocks = { path = "../hick-blocks" }
tempfile = "3"
</book:entry>

</book:doc>
