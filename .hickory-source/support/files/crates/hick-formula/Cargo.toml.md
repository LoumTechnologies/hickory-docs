<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-formula/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3e1d8aa98be58302709e25ed2dc016a857d36a100c9aee6dcccdc150449669dc`.

<book:entry path="crates/hick-formula/Cargo.toml">
[package]
name = "hick-formula"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
hick-plugins = { path = "../hick-plugins" }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["process", "io-util", "rt", "macros", "time", "sync"] }

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["full"] }
</book:entry>

</book:doc>
