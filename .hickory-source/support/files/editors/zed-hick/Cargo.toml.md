<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# editors/zed-hick/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cc15dca99f14daf074a418e957a1be0f24b9a56a810f8699fdcb65539ae98b4a`.

<book:entry path="editors/zed-hick/Cargo.toml">
# Standalone crate: built by Zed's extension toolchain for wasm32-wasip1,
# deliberately NOT a member of the root workspace (see [workspace] below).
[workspace]

[package]
name = "hick-zed"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.5.0"
</book:entry>

</book:doc>
