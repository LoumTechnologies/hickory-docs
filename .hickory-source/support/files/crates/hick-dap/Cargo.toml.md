<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-dap/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `da1ceda501bb304b94143534b6239964e910010da9517c103a28ab3bdeceb933`.

<book:entry path="crates/hick-dap/Cargo.toml">
[package]
name = "hick-dap"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
hick-plugins = { path = "../hick-plugins" }
hick-observe = { path = "../hick-observe" }
hick-lang.workspace = true
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["process", "io-util", "sync", "rt", "macros", "time"] }
tracing = "0.1"
# A capture run weaves the document into a scratch directory it deletes.
tempfile = "3"
# The document -> virtual-file position mapping, which debugging needs in
# exactly the form the language server already needed it.
hick-lsp = { path = "../hick-lsp" }

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
# The live tests weave a document by hand to produce the file they debug.
hick-lsp = { path = "../hick-lsp" }
</book:entry>

</book:doc>
