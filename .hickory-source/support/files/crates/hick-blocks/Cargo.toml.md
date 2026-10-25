<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-blocks/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d1c3c935650c30670ea4b1aa0578939f5f668fd3ed06a864d2f59042407f45aa`.

<book:entry path="crates/hick-blocks/Cargo.toml">
[package]
name = "hick-blocks"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true
repository.workspace = true
description = "The element registry: what a hick tag is, how it renders to a block, and what it can do"

[dependencies]
hick-lang.workspace = true
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
</book:entry>

</book:doc>
