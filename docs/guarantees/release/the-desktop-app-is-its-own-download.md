# The Desktop App Is Its Own Download

Given a person who wants a window rather than a terminal, when they go to the
releases page, then they find an installer in their platform's own shape —
`.dmg` on macOS, `.msi` on Windows, `.deb` and `.AppImage` on Linux — built
from the same commit and carrying the same version as the CLI archives beside
it, and installing it requires no Rust toolchain, no Node, and no checkout.

The two downloads are separate on purpose. A GUI application on macOS is a
bundle you drag, not a binary you put on `PATH`; a CLI user should not have to
download a webview shell to run `hick test` in CI. Making them one artifact
would make both worse.

They are **one product** underneath, and that is what stops the split from
becoming a fork: `apps/desktop` runs `hickory_cli::serve` in its own process,
so weaving, lineage, and carrying an output edit back into its document have
exactly one implementation. The app's editor answers language questions
through the same `hick-lsp` an editor uses, driven in-process — see
[the-notebook-asks-the-same-questions-an-editor-does](../editor-intelligence/the-notebook-asks-the-same-questions-an-editor-does.md).
Nothing in it reaches the network: no telemetry, no update check, no account.

**Asset names are a contract, as they are for the CLI.** Tauri names bundles
from `productName` — `Hickory Docs_0.1.0_amd64.deb`, with a space — so
`scripts/dist-desktop.sh` renames every bundle to one shape,
`hickory-docs-<version>-<target>.<ext>`, and writes a `.sha256` beside it.
Changing that shape breaks the download links in
`docs/users/install-desktop.md`; change them together.

**The bundles are unsigned, and the docs say so.** Gatekeeper and SmartScreen
will both interrupt the first launch. `docs/users/install-desktop.md` gives the
exact click-through for each, and points at the checksum as the verification
that does exist today. Claiming a smooth first launch we cannot deliver would
be worse than naming the friction.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified for Linux; the other three targets are built by the same
  script but have not been produced anywhere yet
- Evidence:
  - `scripts/dist-desktop.sh` builds via `cargo tauri build`, collects every
    installable bundle, renames it to `hickory-docs-<version>-<target>.<ext>`,
    and writes checksums. It fails loudly when `cargo-tauri` is absent, when
    the bundle directory does not exist, and when the build produced no
    installable bundle at all.
  - Run end to end on this machine
    (`TAURI_BUNDLES=deb ./scripts/dist-desktop.sh 0.1.0-local`): a 10.7 MB
    `hickory-docs-0.1.0-local-x86_64-unknown-linux-gnu.deb`, `sha256sum -c`
    accepts its checksum file, and `dpkg -c` shows
    `usr/bin/hickory-desktop` (31 MB) plus a `.desktop` entry. The binary
    launched against a real project directory and stayed up.
  - `.github/workflows/release-build.yml` gained a `desktop` matrix job
    (Linux x86_64, both macOS architectures, Windows) calling that same
    script, and both channels' publish jobs now collect
    `{hick-*,hickory-docs-desktop-*}` so the bundles attach to the same
    release as the CLI archives.
  - **A real bug this found:** `tauri.conf.json`'s `beforeBuildCommand` and
    `beforeDevCommand` pointed at `../../web`. Tauri runs those from the *app*
    directory (`apps/desktop`), not from `src-tauri/`, so they resolved to
    `<repo>/web` — which does not exist. The app had therefore never been
    built by anyone. Fixed to `../web`; `frontendDist` stays `../../web/dist`,
    which IS resolved relative to the config file.
- Caveats — what LLM review could NOT establish:
  - **Only the `.deb` has been produced.** `.AppImage`, `.dmg`, and `.msi`
    have never been built here — Linux is this machine's only platform, and
    the CI job that builds the rest has not run. macOS bundling in particular
    is where an unsigned build most often surprises.
  - **No window has been observed.** The binary was launched with a project
    directory and did not exit before it was killed, which says the engine
    started; nothing here confirms what was rendered, and there is no
    automated UI test.
  - `cargo install tauri-cli` in CI is an uncached ~4 minute compile per
    platform per release. It works; it is not fast.
  - Nothing verifies the version stamped into the bundle equals the version in
    the asset name, the way the CLI's `hick --version` smoke test does. The
    app has no `--version` flag to ask.
- Test coverage: none automated. This guarantee rests on the release job
  running and on the manual `.deb` verification above — which is the same
  standing as `a-download-runs-without-a-rust-toolchain.md` had before its
  first real matrix run.
