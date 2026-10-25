<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang-wasm/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `794a94dc0239d352cb4d87dc1f986a59b3df2d93b0dce4782f176604c44bf3db`.

<book:entry path="crates/hick-lang-wasm/Cargo.toml">
[package]
name = "hick-lang-wasm"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true
repository.workspace = true
description = "The hick parser for the editor: hick-lang compiled to WebAssembly"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
hick-observe = { path = "../hick-observe" }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
hick-lang = { workspace = true, features = ["serde"] }
serde_json = "1"
wasm-bindgen = "0.2"

# The distribution's `wasm-opt` (binaryen) refuses features the current
# toolchain emits, and the parser is 80 KB unoptimised — not worth a second
# toolchain to shave. The release profile's LTO already applies.
[package.metadata.wasm-pack.profile.release]
wasm-opt = false
</book:entry>

</book:doc>
