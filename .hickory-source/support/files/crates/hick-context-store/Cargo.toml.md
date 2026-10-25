<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-context-store/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `40b73ff4b4a3a35d24c86738579e3535a8c16620c4872e7e34cf7537ce00a305`.

<book:entry path="crates/hick-context-store/Cargo.toml">
[package]
name = "hick-context-store"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
hick-context-protocol = { path = "../hick-context-protocol" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tempfile = "3"
fs2 = "0.4"
uuid = { version = "1", features = ["v4"] }
</book:entry>

</book:doc>
