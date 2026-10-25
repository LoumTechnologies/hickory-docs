<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/git-libraries/gix/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `839f7daa84a842f12c67209d710abb41d4c99bda2f0d4c4ab2e3dba0f57d1908`.

<book:entry path="experiments/git-libraries/gix/Cargo.toml">
# Candidate 1: gitoxide. Pure Rust — with a caveat this experiment exists to
# measure.
#
# `gix`'s own `blocking-http-transport-reqwest-rust-tls` feature resolves to
# rustls' default crypto provider, which is aws-lc-sys — C, and therefore an
# NDK build. To find out whether gitoxide can be genuinely C-free over HTTPS,
# the TLS stack is selected here instead: the transport feature without a TLS
# opinion, plus reqwest pinned to rustls-with-ring.
[package]
name = "candidate-gix"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
gix = { version = "0.86", default-features = false, features = [
    "sha1",
    "blocking-network-client",
    "blocking-http-transport-reqwest",
] }
reqwest = { version = "0.12", default-features = false, features = [
    "blocking",
    "rustls-tls-manual-roots-no-provider",
] }
rustls = { version = "0.23", default-features = false, features = ["ring", "std", "tls12"] }
</book:entry>

</book:doc>
