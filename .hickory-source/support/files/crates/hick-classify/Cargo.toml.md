<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-classify/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ff82b7a71252ed34c2a190a71c461128742560ee7a67cf397ed0795714ae96dc`.

<book:entry path="crates/hick-classify/Cargo.toml">
[package]
name = "hick-classify"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[features]
host-mask = ["arrow", "parquet", "blake3"]

[dependencies.arrow]
version = "54"
optional = true

[dependencies.parquet]
version = "54"
features = ["arrow"]
optional = true

[dependencies.blake3]
version = "1"
optional = true

[dev-dependencies]
tempfile = "3"
</book:entry>

</book:doc>
