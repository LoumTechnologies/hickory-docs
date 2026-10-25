<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-store/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ad1551b5f422170fc06fc6a420069de72192c4790ef335f4b1a11559a33c5a13`.

<book:entry path="crates/hick-store/Cargo.toml">
[package]
name = "hick-store"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[features]
default = []
s3 = ["aws-sdk-s3", "aws-config"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
hex = "0.4"
thiserror = "2"
tokio = { version = "1", features = ["process", "fs", "sync", "io-util"] }
chrono = { version = "0.4", features = ["serde"] }
log = "0.4"

# S3 (optional)
aws-sdk-s3 = { version = "1", optional = true }
aws-config = { version = "1", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tempfile = "3"
</book:entry>

</book:doc>
