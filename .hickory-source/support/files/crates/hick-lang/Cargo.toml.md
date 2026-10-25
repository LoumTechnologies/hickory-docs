<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c25460b89a5880559553d287d9ea0b1f4dfe7f1d39f6f0e92450f1f74ec6d041`.

<book:entry path="crates/hick-lang/Cargo.toml">
[package]
name = "hick-lang"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
thiserror = "2"
sha2 = "0.10"
serde_json = "1"
serde = { version = "1", features = ["derive"], optional = true }

[features]
serde = ["dep:serde"]

[dev-dependencies]
tempfile = "3"
</book:entry>

</book:doc>
