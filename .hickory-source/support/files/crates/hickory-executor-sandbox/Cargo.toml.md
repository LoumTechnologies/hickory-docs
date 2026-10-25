<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor-sandbox/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f1c4b3d3b1fa814cbcb4af6a38409107f2dabe000ea9f176f531962e52de299b`.

<book:entry path="crates/hickory-executor-sandbox/Cargo.toml">
[package]
name = "hickory-executor-sandbox"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
hick-plugins = { path = "../hick-plugins" }
anyhow = "1"
async-trait = "0.1"
log = "0.4"
hick-token = { path = "../hick-token" }
hickory-executor = { path = "../hickory-executor" }

[dev-dependencies]
hick-token = { path = "../hick-token" }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "fs"] }
tempfile = "3"

[target.'cfg(windows)'.dependencies]
# AppContainer, the ACL that makes the workdir writable, and the job object.
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_Security",
  "Win32_Security_Authorization",
  "Win32_Security_Isolation",
  "Win32_System_JobObjects",
  "Win32_System_Threading",
] }
</book:entry>

</book:doc>
