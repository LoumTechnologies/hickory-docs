<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/core-check/Cargo.toml

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/tools/core-check/Cargo.toml">
[package]
name = "hickory-dogfood-core-check"
version = "0.1.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
hick-lang = { path = "../../../crates/hick-lang" }
serde_json = "1"
sha2 = "0.10"
hick-project-env = { path = "../../../crates/hick-project-env" }
</maint:entry>
</maint:doc>
