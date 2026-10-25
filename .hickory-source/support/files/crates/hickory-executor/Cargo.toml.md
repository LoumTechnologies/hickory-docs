<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2c2415aa4b2f54fad8fa8a8588bf99f8ee536313e3b7183f92813128c7b4d067`.

<book:entry path="crates/hickory-executor/Cargo.toml">
[package]
name = "hickory-executor"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
anyhow = "1"
async-trait = "0.1"
hick-token = { path = "../hick-token" }
log = "0.4"
serde = { version = "1", features = ["derive"] }
tar = "0.4"
tempfile = "3"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "io-util", "sync", "time", "process", "fs"] }

[target.'cfg(unix)'.dependencies]
# Process-group kill on timeout: SIGKILL to -pid so a cell's background
# children die with it. See LocalExecutor::kill_hard.
libc = "0.2"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
</book:entry>

</book:doc>
