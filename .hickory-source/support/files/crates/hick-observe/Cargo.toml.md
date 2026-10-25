<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-observe/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `72eaece8cbde99befbe339c95d57c13bbe3d5516d4b2ba89a99bd9fb77fed286`.

<book:entry path="crates/hick-observe/Cargo.toml">
[package]
name = "hick-observe"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
sha2 = "0.10"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tempfile = "3"
fs2 = "0.4"

[target.'cfg(target_arch = "wasm32")'.dependencies]
uuid = { version = "1", features = ["js"] }
</book:entry>

</book:doc>
