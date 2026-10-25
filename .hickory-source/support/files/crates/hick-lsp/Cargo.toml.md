<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lsp/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d58ec0b75648e7a871878f450368b8590f0eb3af37778eaf8227fc0198658082`.

<book:entry path="crates/hick-lsp/Cargo.toml">
[package]
name = "hick-lsp"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[[bin]]
name = "emit-languages"
path = "src/emit_languages.rs"

[dependencies]
hick-plugins = { path = "../hick-plugins" }
hick-lang = { path = "../hick-lang" }
hick-blocks = { path = "../hick-blocks" }
hick-project-env = { path = "../hick-project-env" }
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
tempfile = "3"
</book:entry>

</book:doc>
