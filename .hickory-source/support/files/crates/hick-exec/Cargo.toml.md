<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-exec/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `416f232c26f37da1646722bcf8cb13160bd03489cbd34bd2c1a19089b819d803`.

<book:entry path="crates/hick-exec/Cargo.toml">
[package]
name = "hick-exec"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
hick-condition = { path = "../hick-condition" }
hick-flow = { path = "../hick-flow" }
hick-lang = { path = "../hick-lang" }
tokio = { version = "1", features = ["sync", "rt", "macros"] }
tokio-stream = "0.1"
tokio-util = "0.7"
futures = "0.3"
thiserror = "2"
log = "0.4"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
</book:entry>

</book:doc>
