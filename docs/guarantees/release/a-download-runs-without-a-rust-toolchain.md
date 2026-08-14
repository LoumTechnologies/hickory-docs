# A Download Runs Without A Rust Toolchain

Given a machine with no Rust toolchain, no C compiler, and no checkout of this
repository, when a person downloads the release archive matching their
platform and unpacks it, then `hick --version` and `hick test examples/`
both work — using only the archive's own contents plus whatever the example
documents themselves invoke.

Every published archive therefore carries five things and not just a binary:
the `hick` executable, the `hick-lsp` language server, `LICENSE`,
`README.md`, and the full `examples/` tree including its committed outputs.
`hick test` compares re-derived output against committed output, so shipping
the examples without their `.md`, `.svg`, and `.html` products would ship
something that cannot be verified.

`hick-lsp` is in the archive for the same reason the examples are: it is
advertised (README, `docs/users/editor-setup.md`, the home page) as part of
what this product is, and the only other way to get it would be
`cargo install`, which is precisely the Rust toolchain this guarantee says a
download does not need. Its smoke test is `printf '' | hick-lsp` — it speaks
LSP over stdio and has no `--version`, and closed stdin is the one input that
makes it exit rather than wait, so an exit code proves the binary loads and
runs.

The platforms and what each promises:

| Artifact | Runs on |
|---|---|
| `x86_64-unknown-linux-gnu` | glibc 2.35 or newer (Ubuntu 22.04, Debian 12, RHEL 9) |
| `aarch64-unknown-linux-gnu` | the same, on arm64 |
| `aarch64-apple-darwin` | macOS 11 or newer, Apple Silicon |
| `x86_64-apple-darwin` | macOS 11 or newer, Intel |
| `x86_64-pc-windows-msvc` | Windows x86_64 |

**What is proven per target, and what is only built.** The release build runs
a smoke test that unpacks the published archive and executes it, but a runner
can only execute a binary of its own architecture. The step compares the
target triple's architecture against `uname -m` and skips itself — saying so
with a `::notice` in the log — when they differ, rather than failing on a
cross-compiled artifact it was never able to run. So:

| Artifact | Built | `--version` run | `hick test` run | `hick-lsp` run |
|---|---|---|---|---|
| `x86_64-unknown-linux-gnu` | yes | yes | yes, plus the `clean-machine` job | yes |
| `aarch64-unknown-linux-gnu` | yes | yes | yes | yes |
| `aarch64-apple-darwin` | yes | yes | yes | yes |
| `x86_64-apple-darwin` | yes | no — cross-compiled from an arm64 runner | no | no |
| `x86_64-pc-windows-msvc` | yes | yes | no — see below | no — presence only |

`x86_64-apple-darwin` is therefore the one artifact whose *runnability* this
repository has never observed; it is published on the strength of Apple's
toolchain treating its two architectures as one first-class cross-compile.
Rosetta is not guaranteed on the runner image, so the honest options were to
skip the smoke test or to pretend, and this skips it.

Three limits are stated rather than papered over. There is no musl artifact,
so Alpine and any glibc older than 2.35 build from source. The Windows binary
runs, but executing a *document* there additionally needs a POSIX `sh`,
because the local executor spawns one per cell; `--version`, `--help`, and
parsing work without one — which is exactly what its smoke test checks. And
on Windows the Cloud Canopy executor can only reach an agent over its mesh
`host:port` address: `CANOPY_AGENT` set to a unix socket path fails at first
use with a message naming that alternative, because Windows has no unix
domain sockets. The first two are in `docs/users/install.md`, where someone
about to download would look; the third is in
`docs/specs/freeform/canopy-integration.md`, where an operator configuring
canopy would.

That last point is a build constraint, not a nicety. `tokio::net::UnixStream`
does not exist on Windows, so before it was `cfg`-gated it was the single
symbol preventing the whole `hick` binary from building for
`x86_64-pc-windows-msvc` — every other crate in the tree, including the C
dependencies, compiled there.

**The example documents must be portable, not merely Linux-correct.** Each
archive ships `examples/` and the README tells a stranger to run
`hick test examples/`, so an example whose expectation encodes GNU
coreutils behaviour turns a working download into a failing one on macOS.
`hick:expect match="exact"` compares bytes, and BSD `wc -l` right-aligns its
count in eight columns where GNU `wc -l` does not — so cells are written to
normalize such output (`| tr -d ' '`) and say in prose why.

The build must not require a container-based cross-compilation harness, a
self-hosted machine, or a paid runner class. It uses stock GitHub-hosted
runners, one per architecture, and installs nothing beyond NASM on Windows.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: partially verified — see caveats
- Evidence:
  - `scripts/dist.sh` builds `-p hickory-cli` for one target and stages the
    binary, `README.md`, `LICENSE`, and `examples/` into
    `dist/hickory-<version>-<target>/` before archiving, then writes a
    `.sha256` beside the archive. The binary path it reads
    (`target/<triple>/release/hick[.exe]`) follows from
    `[[bin]] name = "hick"` in `crates/hickory-cli/Cargo.toml`, and the
    script fails loudly, naming that manifest, if the file is absent.
  - Run end to end on this machine for `x86_64-unknown-linux-gnu`
    (`./scripts/dist.sh x86_64-unknown-linux-gnu 0.0.0-localtest`): 1m11s,
    producing a 4.0 MB tarball around a 9.2 MB stripped binary, plus a
    `.sha256` that `sha256sum -c` accepts. Unpacked into a directory outside
    the repository, that binary reported `hick 0.0.0-localtest` and then
    `hick test examples/` exited 0 with all three example documents `ok` —
    run entirely against the archive's own copy of `examples/`.
  - `.github/workflows/release-build.yml` builds all five targets by calling
    that same script — the same entry point `just dist` uses, so CI cannot
    package differently from a maintainer reproducing it.
  - Its `clean-machine` job is the direct test of this guarantee: it downloads
    only the x86_64 Linux artifact onto a runner with no checkout, runs
    `rustup self uninstall` and asserts `cargo` is gone from `PATH`, then
    unpacks the tarball and runs `hick test examples/`. It runs on
    `ubuntu-latest` while the artifact is built on `ubuntu-22.04`, so it also
    catches a binary tied to its build image.
  - Version reporting: `crates/hickory-cli/src/main.rs` prefers
    `option_env!("HICKORY_VERSION")` over `CARGO_PKG_VERSION`, and
    `scripts/dist.sh` exports it. Verified locally — a plain build reports
    `hick 0.1.0`, one with `HICKORY_VERSION=9.9.9-test` reports that.
  - Why the matrix has no cross-compilation: `cargo tree -p hickory-cli`
    shows `ring`, `libsodium-sys` (autotools, not the `cc` crate),
    `zstd-sys`, `bzip2-sys`, and `lzma-sys` all compiling C. Each Linux
    target is therefore built on a runner of its own architecture.
  - **The matrix has now run for real** (Unstable Release run
    `31445166789`, commit `9d329eb`), which settles most of the previous
    round's caveats. `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
    and `x86_64-apple-darwin` all built and passed; arm64 Linux runners *are*
    available to this repository, and `libsodium-sys`'s autotools `--host`
    handling for `x86_64-apple-darwin` and `ring`'s NASM requirement on MSVC
    both turned out to be non-issues — the Windows leg compiled the entire
    dependency tree, including every C dependency, before it failed.
  - The two legs that failed in that run, and what changed here:
    - `x86_64-pc-windows-msvc` failed to build:
      `error[E0433]: cannot find UnixStream in net`, one error, in
      `hickory-executor-canopy`. Fixed by splitting the unix-socket dialer
      into `connect_unix_socket` in
      `crates/hickory-executor-canopy/src/executor.rs`, with a `#[cfg(unix)]`
      implementation and a `#[cfg(not(unix))]` one that `bail!`s naming
      `CANOPY_AGENT=<mesh host:port>` and `HICKORY_EXECUTOR=local` as the two
      ways out. The `Uri` import moved under the same gate. The TCP path is
      untouched and identical on every platform.
    - `aarch64-apple-darwin` built and its binary **ran** — the log shows
      `hick 0.1.0-unstable.9d329eb0` — then exited 3 on
      `examples/text-tools-tour.hick:69`: `expected "0", got "       0"`.
      That was BSD `wc -l`'s eight-column padding, i.e. a portability bug in
      the shipped example rather than anything about the release path. Fixed
      in `examples/text-tools-tour.hick` by piping through `tr -d ' '`, with
      the reason written into the document's own prose;
      `examples/text-tools-tour.md` regenerated with `just run`.
  - `.github/workflows/release-build.yml`'s smoke step now derives skip/run
    from `${{ matrix.target }}` vs `uname -m` rather than a hand-maintained
    `smoke: none` flag, and emits a `::notice` when it skips. This is what
    makes the per-target table above true by construction instead of by
    matrix bookkeeping.
- Caveats — what LLM review could NOT establish without a real run:
  - **Neither fix has run in CI.** Both are CI-only failure modes: the
    Windows build cannot be reproduced on this machine (`cargo check
    --target x86_64-pc-windows-msvc` dies in `libsodium-sys`'s `configure`
    with ``Invalid configuration `x86_64-pc-windows-msvc': OS `msvc' not
    recognized`` — cross-compiling it from Linux needs an MSVC toolchain that
    is not present and is not worth installing). What *was* verified locally:
    inverting the two `cfg` attributes and running
    `cargo check -p hickory-executor-canopy` compiles the `not(unix)` arm
    clean, with no warnings and no unused imports. That the rest of the crate
    and `hickory-cli` are Windows-clean rests on the CI log above (the whole
    tree compiled up to this one crate) plus a grep showing the only other
    `std::os::unix` uses in the workspace —
    `crates/hickory-cli/src/init.rs:234` and `:485` — are already inside
    `#[cfg(unix)]` blocks.
  - **`x86_64-apple-darwin` has still never been executed anywhere**, and
    under the new smoke gate it never will be on the current runner images.
    Its row in the table above says so; if this matters, the fix is an
    Intel macOS runner, not a change to the gate.
  - The macOS `hick test examples/` path is now proven only for
    `text-tools-tour.hick`, which is all the smoke test runs. The other two
    example documents need `python3`/`polars`/`duckdb`, and their macOS
    behaviour is unobserved — the `clean-machine` job runs the full
    `examples/` tree on Linux only. A grep for the usual GNU-vs-BSD
    divergences (`sed -i`, `date -d`, `stat -c`, `readlink -f`, `grep -P`,
    `sort -V`, `base64 -w`) across `examples/*.hick` found none, which is
    evidence and not proof.
  - The archive was unpacked and run on the machine that built it — which has
    a Rust toolchain, a C compiler, `duckdb`, and `uv` already. Nothing in the
    run used them, but "no toolchain present" was not actually demonstrated.
    That is what the `clean-machine` job exists to establish, and it has not
    yet run: the run above never reached it, because it `needs: [build]` and
    two legs failed.
- **2026-08-13, `hick-lsp` added to the archive.** `scripts/dist.sh` now
  builds `-p hickory-cli -p hick-lsp` in one `cargo build` and stages both
  binaries, failing loudly and naming both manifests if either is absent;
  `scripts/install.sh` installs `hick-lsp` beside `hick` and, for an older
  release that has no such file, installs `hick` and says what is missing
  rather than failing. Verified locally for
  `x86_64-unknown-linux-gnu`: the archive contains `hick` and `hick-lsp`,
  and `printf '' | ./hick-lsp` exits 0 from the unpacked directory. The
  Windows leg checks presence only (`test -f hick-lsp.exe`), because the
  stdin-EOF trick is not worth trusting through Git Bash's pipe emulation.
  Not yet observed in CI on any target.
- Test coverage: the `clean-machine` job in
  `.github/workflows/release-build.yml` is the executable form of this
  guarantee, and runs on every Unstable and Stable Release.
