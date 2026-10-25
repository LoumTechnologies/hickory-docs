<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-fleet/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `102ad2563328fd4f70d1b3dfbd76e272031e9f403c9914d7885d68b720a88fab`.

<book:entry path="crates/hickory-fleet/Cargo.toml">
[package]
name = "hickory-fleet"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true
description = "Machine identity for one engineer's several machines: keypairs, a mutual key list, and per-verb grants"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
base64 = "0.22"
# Machine identity. BSD-3-Clause — permissive, not copyleft, so it may be
# linked here (the workspace rule forbids GPL and copyleft, not BSD).
ed25519-dalek = { version = "2", features = ["rand_core"] }
rand_core = { version = "0.6", features = ["getrandom"] }
hickory-workspace = { path = "../hickory-workspace" }
fs2 = "0.4"
tempfile = "3"
</book:entry>

</book:doc>
