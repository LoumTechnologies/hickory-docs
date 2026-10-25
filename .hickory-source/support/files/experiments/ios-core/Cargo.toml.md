<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/ios-core/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `bb30942966242de2b7b5f715066a452b954dd874e970bf5c9b6b2289fac13f75`.

<book:entry path="experiments/ios-core/Cargo.toml">
# Can a phone actually do the phone's job?
#
# `notes-ide.md` says the phone reads and captures and never executes, and that
# `hick weave` — render from cached transcripts, mark never-run blocks as
# never-run — is the mobile story. Everything supporting that had been
# established by cross-compiling, which proves a crate *links* and nothing about
# whether it *works* inside an app container.
#
# This probe runs the story: parse a real document, build the block model the
# app and the CLI both render from, derive speaker turns from a real transcript,
# and confirm nothing executed. Installed as a real `.app` on a simulator, not
# run through `simctl spawn`, which would sit outside the sandbox.
[package]
name = "ios-core-probe"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[[bin]]
name = "ios-core-probe"
path = "src/main.rs"

[dependencies]
# The portable five, per crates/hickory-cli/tests/portability.rs.
hick-lang = { path = "../../crates/hick-lang" }
hick-transcript = { path = "../../crates/hick-transcript" }
# The weave path. NOT in the portable set, and linked here on purpose: the
# question this answers is whether the mobile story is reachable at all, and
# `build_block_model` is where the "never run" mark is actually made.
hick-literate = { path = "../../crates/hick-literate" }
hickory-executor = { path = "../../crates/hickory-executor" }
serde_json = "1"
</book:entry>

</book:doc>
