<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor-canopy/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a4f6faed3f7770646f9151c2121aaa3e5b64a8f1f2002fa0764d299a19621310`.

<book:entry path="crates/hickory-executor-canopy/Cargo.toml">
[package]
name = "hickory-executor-canopy"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
anyhow = "1"
async-trait = "0.1"
base64 = "0.22"
hick-token = { path = "../hick-token" }
hickory-executor = { path = "../hickory-executor" }
hyper-util = "0.1"
log = "0.4"
prost = "0.13"
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "io-util", "sync", "time", "net"] }
tokio-stream = "0.1"
tonic = "0.12"
tower = "0.5"

[build-dependencies]
tonic-build = "0.12"
protoc-bin-vendored = "3"

[dev-dependencies]
tar = "0.4"
tempfile = "3"
</book:entry>

</book:doc>
