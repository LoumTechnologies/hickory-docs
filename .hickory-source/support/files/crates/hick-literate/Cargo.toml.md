<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e56ffa83d9c0937340c52eff5e43de4d9209ab59745b842f53f6a9793e421c7d`.

<book:entry path="crates/hick-literate/Cargo.toml">
[package]
name = "hick-literate"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[lib]
name = "hick_literate"
path = "src/lib.rs"

[dependencies]
hick-plugins = { path = "../hick-plugins" }
ignore = "0.4"
hickory-executor = { path = "../hickory-executor" }
hick-observe = { path = "../hick-observe" }
tempfile = "3"
hick-case = { path = "../hick-case" }
hick-condition = { path = "../hick-condition" }
# `<hick:capture>` runs the document under a debug adapter. hick-dap reaches
# hick-lsp for the position mapping and no further, so this is not a cycle.
hick-dap = { path = "../hick-dap" }
hick-exec.workspace = true
hick-handlers = { path = "../hick-handlers" }
hick-feature = { path = "../hick-feature" }
hick-lang.workspace = true
hick-blocks.workspace = true
hick-transcript = { workspace = true }
hick-secrets = { path = "../hick-secrets" }
hick-token = { path = "../hick-token" }
hick-store = { path = "../hick-store" }
hick-merge = { path = "../hick-merge" }

async-trait = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "io-util", "sync", "time", "signal"] }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
env_logger = "0.11"
log = "0.4"
rand = "0.8"
regex = "1"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
serde_yaml = "0.9.34"
thiserror = "2"
glob = "0.3.3"
indexmap = "2"
walkdir = "2"
sha2 = "0.10.9"
base64 = "0.22"
tar = "0.4"
zip = "2"
notify = "7"
futures = "0.3"
sysinfo = "0.38.0"
pulldown-cmark = { version = "0.12", default-features = false, features = ["html"] }
</book:entry>

</book:doc>
