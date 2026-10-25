<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor-docker/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `718f6746378d74f1d41eb91455ac24794c601076438252bb17cd70b1350f0e06`.

<book:entry path="crates/hickory-executor-docker/Cargo.toml">
[package]
name = "hickory-executor-docker"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true
description = "Executor backend that runs hick containers as real Docker containers"

[dependencies]
anyhow = "1"
async-trait = "0.1"
hick-token = { path = "../hick-token" }
hickory-executor = { path = "../hickory-executor" }
log = "0.4"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "io-util", "process", "sync", "time"] }

[dev-dependencies]
tar = "0.4"
tempfile = "3"
</book:entry>

</book:doc>
