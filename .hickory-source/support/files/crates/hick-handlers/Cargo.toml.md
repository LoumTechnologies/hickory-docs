<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-handlers/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `97ac30fe438cf893e715c23393421a14991aa4850cf96765a0bbec218fabfaba`.

<book:entry path="crates/hick-handlers/Cargo.toml">
[package]
name = "hick-handlers"
version = "0.1.0"
edition = "2024"
license.workspace = true

[dependencies]
hick-exec.workspace = true
hick-lang.workspace = true
anyhow = "1"
log = "0.4"
futures = "0.3.31"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
</book:entry>

</book:doc>
