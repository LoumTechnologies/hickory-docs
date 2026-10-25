<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c1013a4e992af5968d9a8f7a90f86e3f8f436a8e6a6d950a5a2e2c4186b23458`.

<book:entry path="crates/hick-plugins/Cargo.toml">
[package]
name = "hick-plugins"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
tree-sitter = "0.25"
tree-sitter-python = "0.25"
tree-sitter-rust = "0.24"
tree-sitter-javascript = "0.25"
tree-sitter-typescript = "0.23"
anyhow = "1"
tokio = { version = "1", features = ["process", "io-util", "rt", "macros", "time", "sync"] }
serde_json = "1"
serde = { version = "1", features = ["derive"] }
graphql-parser = "0.4"
hick-project-env = { path = "../hick-project-env" }

[dev-dependencies]
tempfile = "3"
</book:entry>

</book:doc>
