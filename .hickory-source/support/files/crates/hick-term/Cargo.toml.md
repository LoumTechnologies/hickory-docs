<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-term/Cargo.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0ced139dfd2b09204ab48b6d6066ddeae6aa525cf91b67f55410078108562413`.

<book:entry path="crates/hick-term/Cargo.toml">
[package]
name = "hick-term"
version = "0.1.0"
edition.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
anyhow = "1"
# The PTY itself. A library, not a CLI we shell out to: ConPTY on Windows,
# openpty everywhere else, behind one trait. Terminals are the one surface
# where "works on the maintainer's laptop" is not good enough — a download
# runs on all three platforms.
portable-pty = "0.8"
# A headless screen model, server-side. The client renders with xterm.js; this
# exists so a session nobody is looking at can still say what it is doing (the
# preview line on a folded row), and so a closed pane loses no state.
vt100 = "0.15"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["sync"] }
# Shell integration writes a startup file per session and deletes it with the
# session. Nothing the user owns is edited, so the scripts have to live
# somewhere that cleans itself up.
tempfile = "3"

[dev-dependencies]
</book:entry>

</book:doc>
