<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Build the stream library

Only async stream coordination is needed here; parsing and serialization
are dependencies of the document adapter.

<storycode:entry path="crates/hick-flow/Cargo.toml">
[package]
name = "hick-flow"
version = "0.1.0"
edition = "2024"
license.workspace = true

[dependencies]
futures = "0.3"
tokio = { version = "1", features = ["sync", "rt", "macros"] }
tokio-util = "0.7"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
</storycode:entry>
</storycode:doc>
