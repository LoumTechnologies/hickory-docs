<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-broker/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e4c53097a57c6940bd15d121b37e3bdb5df4af91a1bf125361128b264f4a00ae`.

<book:entry path="crates/hickory-broker/Cargo.toml">
[package]
name = "hickory-broker"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true
description = "The seal and the broker: an agent machine with no keys and one road out, with a toll booth on it"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util", "time", "sync"] }
log = "0.4"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util", "time", "sync"] }
</book:entry>

</book:doc>
