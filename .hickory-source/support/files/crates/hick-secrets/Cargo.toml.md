<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-secrets/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0faa31b1ee12ebbd3b5df4541fa56e3d868615e80caa7dd181cdf69c5e043616`.

<book:entry path="crates/hick-secrets/Cargo.toml">
[package]
name = "hick-secrets"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
age = "0.11"
async-trait = "0.1"
dashmap = "6"
secrecy = { version = "0.10", features = ["serde"] }
thiserror = "2"
tokio = { version = "1", features = ["rt"] }

[dev-dependencies]
tokio = { version = "1", features = ["rt", "macros"] }
</book:entry>

</book:doc>
