# A Download Runs Without A Rust Toolchain

Given a machine with no Rust toolchain, no C compiler, and no checkout of this
repository, when a person downloads the release archive matching their
platform and unpacks it, then `hickory --version` and `hickory test examples/`
both work — using only the archive's own contents plus whatever the example
documents themselves invoke.

Every published archive therefore carries four things and not just a binary:
the `hickory` executable, `LICENSE`, `README.md`, and the full `examples/`
tree including its committed outputs. `hickory test` compares re-derived
output against committed output, so shipping the examples without their `.md`,
`.svg`, and `.html` products would ship something that cannot be verified.

The platforms and what each promises:

| Artifact | Runs on |
|---|---|
| `x86_64-unknown-linux-gnu` | glibc 2.35 or newer (Ubuntu 22.04, Debian 12, RHEL 9) |
| `aarch64-unknown-linux-gnu` | the same, on arm64 |
| `aarch64-apple-darwin` | macOS 11 or newer, Apple Silicon |
| `x86_64-apple-darwin` | macOS 11 or newer, Intel |
| `x86_64-pc-windows-msvc` | Windows x86_64 |

Two limits are stated rather than papered over. There is no musl artifact, so
Alpine and any glibc older than 2.35 build from source. And the Windows binary
runs, but executing a *document* there additionally needs a POSIX `sh`,
because the local executor spawns one per cell; `--version`, `--help`, and
parsing work without one. Both are in `docs/users/install.md`, where someone
about to download would look.

The build must not require a container-based cross-compilation harness, a
self-hosted machine, or a paid runner class. It uses stock GitHub-hosted
runners, one per architecture, and installs nothing beyond NASM on Windows.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: partially verified — see caveats
- Evidence:
  - `scripts/dist.sh` builds `-p hickory-cli` for one target and stages the
    binary, `README.md`, `LICENSE`, and `examples/` into
    `dist/hickory-<version>-<target>/` before archiving, then writes a
    `.sha256` beside the archive. The binary path it reads
    (`target/<triple>/release/hickory[.exe]`) follows from
    `[[bin]] name = "hickory"` in `crates/hickory-cli/Cargo.toml`, and the
    script fails loudly, naming that manifest, if the file is absent.
  - Run end to end on this machine for `x86_64-unknown-linux-gnu`
    (`./scripts/dist.sh x86_64-unknown-linux-gnu 0.0.0-localtest`): 1m11s,
    producing a 4.0 MB tarball around a 9.2 MB stripped binary, plus a
    `.sha256` that `sha256sum -c` accepts. Unpacked into a directory outside
    the repository, that binary reported `hickory 0.0.0-localtest` and then
    `hickory test examples/` exited 0 with all three example documents `ok` —
    run entirely against the archive's own copy of `examples/`.
  - `.github/workflows/release-build.yml` builds all five targets by calling
    that same script — the same entry point `just dist` uses, so CI cannot
    package differently from a maintainer reproducing it.
  - Its `clean-machine` job is the direct test of this guarantee: it downloads
    only the x86_64 Linux artifact onto a runner with no checkout, runs
    `rustup self uninstall` and asserts `cargo` is gone from `PATH`, then
    unpacks the tarball and runs `hickory test examples/`. It runs on
    `ubuntu-latest` while the artifact is built on `ubuntu-22.04`, so it also
    catches a binary tied to its build image.
  - Version reporting: `crates/hickory-cli/src/main.rs` prefers
    `option_env!("HICKORY_VERSION")` over `CARGO_PKG_VERSION`, and
    `scripts/dist.sh` exports it. Verified locally — a plain build reports
    `hickory 0.1.0`, one with `HICKORY_VERSION=9.9.9-test` reports that.
  - Why the matrix has no cross-compilation: `cargo tree -p hickory-cli`
    shows `ring`, `libsodium-sys` (autotools, not the `cc` crate),
    `zstd-sys`, `bzip2-sys`, and `lzma-sys` all compiling C. Each Linux
    target is therefore built on a runner of its own architecture.
- Caveats — what LLM review could NOT establish without a real run:
  - **Only the x86_64 Linux target has ever been built.** The macOS legs
    (both Apple targets from one arm64 runner, which is the one cross-compile
    kept) and the Windows leg were not exercised at all. The most likely first
    failures are `libsodium-sys`'s autotools `--host` handling for
    `x86_64-apple-darwin`, and `ring`'s NASM requirement on MSVC — the
    workflow installs NASM via Chocolatey and prepends `C:\Program Files\NASM`
    to `PATH`, which was not verified against the runner image's actual
    layout.
  - `aarch64-unknown-linux-gnu` builds on `ubuntu-22.04-arm`. Whether arm64
    Linux runners are available to this (private) repository was not
    confirmed; if they are not, that matrix entry is the one line to remove.
  - The archive was unpacked and run on the machine that built it — which has
    a Rust toolchain, a C compiler, `duckdb`, and `uv` already. Nothing in the
    run used them, but "no toolchain present" was not actually demonstrated.
    That is what the `clean-machine` job exists to establish, and it has not
    yet run.
- Test coverage: the `clean-machine` job in
  `.github/workflows/release-build.yml` is the executable form of this
  guarantee, and runs on every Unstable and Stable Release.
