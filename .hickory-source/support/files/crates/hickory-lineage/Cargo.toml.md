<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-lineage/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `04c883ca295faf698fe0d3996b2cf8b3ac9d551f0d6aa4e7e0c732240cf6579c`.

<book:entry path="crates/hickory-lineage/Cargo.toml">
[package]
name = "hickory-lineage"
version = "0.1.0"
edition = "2024"
publish = false
license.workspace = true

[dependencies]
hick-plugins = { path = "../hick-plugins" }
hick-flow = { path = "../hick-flow" }
hick-lang = { path = "../hick-lang" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
</book:entry>

</book:doc>
