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
- Date: 2026-08-18 (macOS added; Linux verification 2026-08-13 unchanged)
- Reviewer: Claude (Opus 5)
- Result: verified for Linux **and macOS**; `.AppImage` and `.msi` are built by
  the same script and have not been produced anywhere yet
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
  - Run end to end on macOS 2026-08-18 (MacBookPro15,1, macOS 15.7.7, Xcode
    26.3, `cargo-tauri` 2.9.4): `scripts/dist-desktop.sh` produced a 15.5 MB
    `hickory-docs-0.1.0-x86_64-apple-darwin.dmg` with its `.sha256`. The image
    mounts, carries the `Applications` symlink and a volume icon, and the
    `Hickory Docs.app` inside it copies out and **runs**: the bundled executable
    starts the in-process engine on `127.0.0.1`, `GET /api/health` returns
    `{"db":false,"executor":"sandbox","ok":true}`, a `.hick` document dropped in
    `~/Documents/HickoryDocs` was picked up and woven automatically, and
    `POST /api/docs/<id>/run` executed its cell — `status: "ok"`, exit code 0,
    expectation matched. That is the "executes a real document" half of
    `a-download-runs-without-a-rust-toolchain.md`, met by the app rather than the
    CLI.
  - **A second real bug this found:** the `.dmg` step failed on the developer's
    Mac while succeeding in CI. Tauri passes `--skip-jenkins` to its bundled
    `bundle_dmg.sh` only when `CI=true`; otherwise the script drives Finder over
    AppleScript to lay out the disk-image window, and it died with
    `Finder got an error: AppleEvent timed out. (-1712)` *after* the `.app` had
    been built. `scripts/dist-desktop.sh` now sets `CI=true` for the
    `cargo tauri build` call, which both makes a local build work and makes it
    produce the same image CI does.
  - **A real bug this found:** `tauri.conf.json`'s `beforeBuildCommand` and
    `beforeDevCommand` pointed at `../../web`. Tauri runs those from the *app*
    directory (`apps/desktop`), not from `src-tauri/`, so they resolved to
    `<repo>/web` — which does not exist. The app had therefore never been
    built by anyone. Fixed to `../web`; `frontendDist` stays `../../web/dist`,
    which IS resolved relative to the config file.
- Caveats — what LLM review could NOT establish:
  - **`.AppImage` and `.msi` have never been produced.** Only `.deb` and `.dmg`
    have been built by hand, on Linux and macOS respectively; the CI job that
    builds the rest has not run.
  - **The unsigned `.dmg` is refused by Gatekeeper, as expected, and the
    refusal was measured.** `spctl -a -vvv -t exec` reports
    `rejected / source=no usable signature`. A copy carrying a real
    browser-written `com.apple.quarantine` and launched with `open` is created,
    App-Translocated to a randomised read-only path, and then **held without
    initialising** — no listening socket, no window — waiting on a user decision.
    The dialog is composable verbatim from the system's own localized strings in
    `CoreServicesUIAgent.app/Contents/Resources`:
    `Q_HEADLINE_SUNFISH_NOT_VERIFIED` → **“Hickory Docs” Not Opened**;
    `Q_DETAIL_CASPIAN_UNVERIFIED` → **Apple could not verify “Hickory Docs” is
    free of malware that may harm your Mac or compromise your privacy.**;
    buttons **Move to Trash** and **Done**. So the claim in
    `shipping-mobile-and-desktop.md` that this "reads to a new user as malware"
    is literal, and signing is confirmed as delivery work rather than polish.
  - **No window has been observed, on either platform.** On Linux the binary was
    launched and did not exit; on macOS the engine was driven over its own HTTP
    API, because the session it ran in could not present GUI windows at all
    (three separate applications launched with none). Both say the engine
    started. Neither says anything about what was rendered, and there is still no
    automated UI test.
  - **The Gatekeeper dialog was not photographed.** Its text above is read from
    Apple's own string tables and the launch was observed to be held, which is
    strong but is not the same as seeing it. Anyone with a normal desktop session
    can settle it in one double-click.
  - `cargo install tauri-cli` in CI is an uncached ~4 minute compile per
    platform per release. It works; it is not fast.
  - Nothing verifies the version stamped into the bundle equals the version in
    the asset name, the way the CLI's `hick --version` smoke test does. The
    app has no `--version` flag to ask.
- Test coverage: none automated. This guarantee rests on the release job
  running and on the manual `.deb` verification above — which is the same
  standing as `a-download-runs-without-a-rust-toolchain.md` had before its
  first real matrix run.
