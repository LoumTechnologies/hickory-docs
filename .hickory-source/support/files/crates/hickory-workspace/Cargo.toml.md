<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-workspace/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0d7023750912652c2950c5d97b68f757639557027668b0383b49eaac4af38126`.

<book:entry path="crates/hickory-workspace/Cargo.toml">
[package]
name = "hickory-workspace"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Where the platform says per-user state belongs. MIT/Apache-2.0.
dirs = "6"
sha2 = "0.10"

[dev-dependencies]
tempfile = "3"
</book:entry>

</book:doc>
