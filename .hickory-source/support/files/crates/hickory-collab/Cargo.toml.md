<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-collab/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `206d4ebd5477cdf16d713b01030947ecb74a0588bed0318843814df64ab98662`.

<book:entry path="crates/hickory-collab/Cargo.toml">
[package]
name = "hickory-collab"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
anyhow = "1"
async-trait = "0.1"
log = "0.4"
serde_json = "1"
tokio = { version = "1", features = ["rt", "sync", "time", "macros"] }
uuid = { version = "1", features = ["v4"] }
yrs = { version = "0.21", features = ["sync"] }

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["full"] }
</book:entry>

</book:doc>
