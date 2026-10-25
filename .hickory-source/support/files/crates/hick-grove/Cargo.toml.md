<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-grove/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3b4110604dee50d05d5a5b3022e1d4e3961dffdf7976aba62f8636b55c98700d`.

<book:entry path="crates/hick-grove/Cargo.toml">
[package]
name = "hick-grove"
version = "0.1.0"
edition = "2024"
license.workspace = true

[dependencies]
hick-flow = { path = "../hick-flow" }
yrs = "0.21"
tokio = { version = "1", features = ["sync", "rt", "macros"] }
quick-xml = "0.37"
futures = "0.3"
async-stream = "0.3"
uuid = { version = "1", features = ["v7"] }
anyhow = "1"
log = "0.4"
serde = { version = "1", features = ["derive"] }

[dependencies.rusqlite]
version = "0.32"
features = ["bundled"]
optional = true

[features]
default = []
sqlite-example = ["rusqlite"]

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
</book:entry>

</book:doc>
