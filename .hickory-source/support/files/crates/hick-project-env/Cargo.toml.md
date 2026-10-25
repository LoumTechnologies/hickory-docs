<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-project-env/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f6937fca864f4de17c3ad0ed8b7f1c21aa977fe91c907b21a813b30cc6f24d7d`.

<book:entry path="crates/hick-project-env/Cargo.toml">
[package]
name = "hick-project-env"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
hick-lang.workspace = true
async-trait = "0.1"
globset = "0.4"
hickory-workspace = { path = "../hickory-workspace" }
ignore = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sha2 = "0.10"
tempfile = "3"
tokio = { version = "1", features = ["process", "time", "io-util"] }
toml = "0.5"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
</book:entry>

</book:doc>
