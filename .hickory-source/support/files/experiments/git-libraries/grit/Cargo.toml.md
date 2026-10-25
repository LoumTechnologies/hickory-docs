<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/git-libraries/grit/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `02054c298a59a260208667dfcf2a8c1bb096016e83856d26bd9acd8776611347`.

<book:entry path="experiments/git-libraries/grit/Cargo.toml">
# Candidate 3: grit, a from-scratch Rust reimplementation of Git (GitButler).
# `grit-lib` ONLY: `grit-cli` is GPL-2.0 and this product is MIT.
[package]
name = "candidate-grit"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
# `http-ureq` is not on by default, and without it grit-lib builds with NO HTTP
# transport at all — so a cross-compile check that omits it is measuring a
# library that cannot reach a remote over HTTPS, which is the only transport
# that matters on a phone.
grit-lib = { version = "0.5", features = ["http-ureq"] }
</book:entry>

</book:doc>
