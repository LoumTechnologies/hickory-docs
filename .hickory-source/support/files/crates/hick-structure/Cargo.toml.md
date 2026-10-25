<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-structure/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `82712af9978f8e0abdcf830fd8b25f251482f9438aa027a94e3f68d572c52da5`.

<book:entry path="crates/hick-structure/Cargo.toml">
[package]
name = "hick-structure"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
hick-plugins = { path = "../hick-plugins" }
# Tree-sitter and its grammars are C, compiled into this binary. That is the
# whole reason this crate exists rather than a language-server integration:
# structural navigation has to work in a download that installs nothing.
tree-sitter = "0.25"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
serde_json = "1"
</book:entry>

</book:doc>
