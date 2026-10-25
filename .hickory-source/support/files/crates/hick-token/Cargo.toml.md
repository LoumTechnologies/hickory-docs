<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-token/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `54fbe1d6860cb848b6c31b4685dd98667b265f56a19478a3be4218d038014f46`.

<book:entry path="crates/hick-token/Cargo.toml">
[package]
name = "hick-token"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
chrono = { version = "0.4.43", features = ["serde"] }
hick-classify = { version = "0.1.0", path = "../hick-classify" }
macaroon = "0.3"
thiserror = "2"
</book:entry>

</book:doc>
