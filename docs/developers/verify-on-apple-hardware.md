# Prompt: verify the Apple-only claims

> **Run on 2026-08-18** on a MacBook Pro (MacBookPro15,1, Intel Core i7, macOS
> 15.7.7, Xcode 26.3, iOS 26.2 SDK, iOS 26.1 simulator). Seven of the eight items
> were settled; item 3's push landed but a simulator cannot settle the no-spawn
> constraint it also probes. Results live where the conventions put them —
> `docs/specs/freeform/shipping-mobile-and-desktop.md` (tables and open edges),
> the caveat sections of
> `docs/guarantees/authoring/ingest-keeps-the-original-bytes.md`,
> `docs/guarantees/terminal/a-session-appears-where-it-is-working.md`, and
> `docs/guarantees/release/the-desktop-app-is-its-own-download.md`, plus
> `.instructions/continuous-delivery-downloadable.md` for signing.
>
> **What it changed, not just confirmed:** the macOS download-origin parser was
> wrong and is fixed; macOS's shells emit no OSC 7 to a truthful terminal, so
> terminals-in-the-file-tree runs on its fallback there; `scripts/dist-desktop.sh`
> could not build a `.dmg` off CI and now can.
>
> **Still owed, and only a device or an Apple account can settle it:** that a
> sandboxed app on real hardware cannot spawn a process; that a notarized,
> signed build passes Gatekeeper; and anything requiring a `Developer ID
> Application` or `Apple Distribution` identity, neither of which exists yet.
> The prompt below is kept as-is so it can be re-run.

Everything in this repository was built and tested on Linux. Several claims
about macOS and iOS were therefore **written down but unproven**, and each was
recorded as a caveat in a guarantee or an open edge in a spec.

Paste the prompt below into Claude Code on the MacBook, in a checkout of this
repository. It is written to be self-contained: it names each unverified claim,
where the claim lives, and how to settle it.

---

## The prompt

> You are on a MacBook Pro, in a checkout of the hickory-docs repository. Read
> `AGENTS.md` first, then `docs/specs/freeform/shipping-mobile-and-desktop.md`.
>
> Every claim below was written on a Linux machine and is unverified on Apple
> hardware. Your job is to **settle them by measurement**, and to correct the
> documents where the measurement disagrees — not to re-assert what is already
> written. A claim you cannot test is a claim you report as untested; do not
> substitute reasoning for a result.
>
> Work in priority order and stop to tell me if an early result invalidates a
> later task.
>
> ### Priority 1 — the mobile plan depends on these
>
> **1. Does the portable core cross-compile for iOS?** The Android equivalent
> passed with no NDK; iOS has never been tried.
>
> ```sh
> rustup target add aarch64-apple-ios aarch64-apple-ios-sim
> for c in hick-lang hick-transcript hick-flow hick-condition hick-case; do
>   cargo check -p $c --target aarch64-apple-ios
> done
> ```
>
> Report per crate. `crates/hickory-cli/tests/portability.rs` holds the list and
> the rule; if a crate fails, say why before changing anything.
>
> **2. Does `git2` cross-compile for iOS, with TLS?** Sync needs push, and
> `experiments/git-libraries/` established that `git2` is the only candidate of
> the three that can push at all. It needs libgit2 and OpenSSL, both C.
>
> ```sh
> cd experiments/git-libraries/git2
> cargo check --target aarch64-apple-ios
> ```
>
> If OpenSSL is the blocker, try the `vendored-openssl` feature of `git2` and
> report whether that resolves it. Read `experiments/git-libraries/README.md`
> before changing the harness — the trap it documents is real, and a compile
> check that quietly drops the feature under discussion is worse than no check.
>
> **3. Does `git2` actually push, from a simulator, over HTTPS with a token?**
> This is the one that matters most and the one nothing has proven. Reading the
> API established that the function exists; it did not establish that a push
> succeeds from inside the iOS sandbox.
>
> Build the smallest thing that answers it. Suggested order:
> - a Rust integration test run on the simulator (`cargo-dinghy` or an XCTest
>   host), pushing to a scratch repository on a remote you control over
>   **HTTPS with a personal access token** — never SSH, never a credential
>   helper, per the constraint in the spec;
> - failing that, a throwaway Tauri iOS app with one button.
>
> Report: did the push land, what did the remote's ref look like afterwards, and
> what (if anything) the sandbox refused. If it does not work, that changes the
> sync design and I want to know immediately.
>
> ### Priority 2 — shipped features resting on unverified assumptions
>
> **4. Does the macOS download-origin reader work?** `read_origin_attribute` in
> `crates/hickory-cli/src/ingest.rs` reads
> `com.apple.metadata:kMDItemWhereFroms` and scans a URL out of what is actually
> a binary plist. That heuristic has never run against a real file. Test it for
> real: download something with Safari **and** with Chrome into a notes folder's
> `inbox/`, run `hick ingest`, and check whether the note's frontmatter gained a
> `source-url:` — and that the query string was dropped, which is a security
> property, not a cosmetic one. Also confirm `xattr -p
> com.apple.metadata:kMDItemWhereFroms <file>` shows what the code expects.
> Fix the parser if the heuristic fails; a real plist parse is acceptable if it
> is needed.
>
> **5. Does macOS's default zsh emit OSC 7?** The whole
> terminals-in-the-file-tree feature rests on it — see
> `docs/guarantees/terminal/a-session-appears-where-it-is-working.md`, whose
> caveat says no shell was ever driven. Open a terminal in the app, `cd` into a
> subdirectory, and see whether the session's row moves to that directory in the
> file tree and whether `cwd_is_live` is true. If zsh does not emit it, say so —
> the fallback is the start directory, which is honest but means the feature
> barely works on the platform most likely to run this app.
>
> **6. Does the macOS desktop bundle actually run?** `scripts/dist-desktop.sh`
> produces a `.dmg` that CI has never opened. Build it, mount it, launch the
> app, open a folder, and run a document. Then report what Gatekeeper does to an
> unsigned build — verbatim, including the exact dialog — because
> `shipping-mobile-and-desktop.md` claims that reads to a new user as malware and
> I want that claim either confirmed or corrected.
>
> ### Priority 3 — the distribution mechanics
>
> **7. Is the Tauri v2 iOS build viable here?** With Xcode installed, try
> `cargo tauri ios init` against `apps/desktop/src-tauri` (which already declares
> `crate-type = ["staticlib", "cdylib", "rlib"]`, the shape iOS needs). Expect it
> to fail: that crate links `hickory-cli`, which pulls the executor, terminals,
> LSP, and DAP — none of which can exist on iOS. **Do not fix this by threading
> `#[cfg(mobile)]` through it.** Report the failure and what it tells us about
> the engine split the spec calls for. The split is a separate, deliberate piece
> of work.
>
> **8. What exactly does signing need?** Without submitting anything, report
> what is required to notarize a `.dmg` and to sign an iOS build: which Apple
> account tier, which identities, and which of those can live in CI secrets.
> `continuous-delivery-downloadable.md` claims the only secret is
> `GITHUB_TOKEN`; that is already known to be wrong and should be corrected with
> specifics.
>
> ### How to report
>
> Follow this repository's conventions rather than writing a summary file:
>
> - Correct the **caveat sections** of the affected guarantees with what you
>   actually observed, dated, naming the hardware and OS version.
> - Correct the **open edges** in `docs/specs/freeform/shipping-mobile-and-desktop.md`.
> - Where a claim turns out to be wrong, change the claim — do not soften it.
> - Keep each change and its evidence in one commit, and run the full suite plus
>   `cargo clippy --workspace --all-targets` before committing.
> - Tell me plainly which items you could not test and why.

---

## Why each of these was unverifiable on Linux

| Claim | Where it lives | Why Linux cannot settle it |
|---|---|---|
| The portable crates build for iOS | `crates/hickory-cli/tests/portability.rs` | `aarch64-apple-ios` needs the Apple SDK |
| `git2` builds and pushes on iOS | `experiments/git-libraries/README.md` | Same, plus the simulator |
| `kMDItemWhereFroms` parsing | `crates/hickory-cli/src/ingest.rs` | The attribute only exists on a file a macOS browser downloaded |
| zsh emits OSC 7 | `crates/hick-term/src/screen.rs` | Tests feed bytes directly; no shell was driven |
| The `.dmg` runs, and Gatekeeper's behaviour | `scripts/dist-desktop.sh` | Neither exists off macOS |
| Notarization and signing requirements | `.instructions/continuous-delivery-downloadable.md` | Apple's tooling and account model |
