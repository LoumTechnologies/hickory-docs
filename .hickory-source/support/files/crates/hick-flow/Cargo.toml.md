<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-flow/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7aea6add607673a06f47ccebd6199f62be52902798322a61f83ba564739a401b`.

<book:entry path="crates/hick-flow/Cargo.toml">
[package]
name = "hick-flow"
version = "0.1.0"
edition = "2024"
license.workspace = true

[dependencies]
hick-lang = { path = "../hick-lang" }
futures = "0.3"
tokio = { version = "1", features = ["sync", "rt", "macros"] }
tokio-stream = "0.1"
tokio-util = "0.7"
log = "0.4"
serde = { version = "1", features = ["derive", "rc"], optional = true }

[features]
serde = ["dep:serde", "hick-lang/serde"]

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
# Self-dependency so `cargo test -p hick-flow` builds with `serde` on. Without
# it nothing in the workspace enables the feature, and every
# `#[cfg(feature = "serde")]` test silently compiles out — including the
# `SourceOrigin` round-trip that `docs/guarantees/lineage/` relies on.
hick-flow = { path = ".", features = ["serde"] }
</book:entry>

</book:doc>
