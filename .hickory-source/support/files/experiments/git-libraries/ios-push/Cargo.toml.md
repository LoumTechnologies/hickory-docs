<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/git-libraries/ios-push/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d8d875f5d36375d7d5c424fc08cc816875282abbff4de6cf1740d2f4e4c04e4f`.

<book:entry path="experiments/git-libraries/ios-push/Cargo.toml">
# Does `git2` actually push, from inside the iOS sandbox, over HTTPS with a
# token? The other two candidates cannot push at all
# (`experiments/git-libraries/README.md`), so this probe only has to answer the
# question for the one that can.
#
# Built for an iOS **simulator** target, bundled as a real `.app`, installed,
# and launched — not run through `simctl spawn`, which would sit outside the app
# sandbox and so could not answer the question this exists for.
[package]
name = "ios-push-probe"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[[bin]]
name = "ios-push-probe"
path = "src/main.rs"

[dependencies]
# `vendored-openssl` is the feature under discussion: without it the iOS build
# fails at `openssl-sys`, which has no cross-compiling pkg-config to find.
git2 = { version = "0.20", features = ["vendored-openssl"] }
</book:entry>

</book:doc>
