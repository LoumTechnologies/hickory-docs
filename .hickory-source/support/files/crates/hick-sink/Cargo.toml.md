<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-sink/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `618aa45c7c86700dc032a34b3b31ff6fa372f42fc793f6f3261c2b6d7de3a397`.

<book:entry path="crates/hick-sink/Cargo.toml">
[package]
name = "hick-sink"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
hick-classify = { path = "../hick-classify" }
hick-token = { path = "../hick-token" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
async-trait = "0.1"
tokio = { version = "1", features = ["sync"] }
log = "0.4"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
</book:entry>

</book:doc>
