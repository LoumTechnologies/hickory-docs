<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-merge/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cba1664d3fbc4a901667416bd43b7357ba4b8c4d2227c0dac23666b1f5b1b864`.

<book:entry path="crates/hick-merge/Cargo.toml">
[package]
name = "hick-merge"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
hick-store = { path = "../hick-store" }
async-trait = "0.1"
thiserror = "2"
# rustls, not the default native-tls: native-tls drags in `openssl-sys`, the
# workspace's only C dependency, and that is what makes cross-compiling the
# `hick` CLI (musl, aarch64) need a cross toolchain. Every other crate here
# already picks rustls; this was the odd one out.
#
# `http2` and `charset` are named explicitly because they used to arrive here
# by accident: this crate was the only one asking for reqwest's defaults, so
# every crate in the workspace got them through feature unification. Dropping
# the defaults without naming these would have quietly turned HTTP/2 off for
# the agent and the server too.
reqwest = { version = "0.12", default-features = false, features = [
    "charset",
    "http2",
    "json",
    "rustls-tls",
] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Line-level diffing for the merged view's N-way alignment. The same crate
# the reverse-edit path uses, so "the same line" means one thing across the
# product.
similar = "2"
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1", features = ["sync"] }
log = "0.4"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
wiremock = "0.6"
</book:entry>

</book:doc>
