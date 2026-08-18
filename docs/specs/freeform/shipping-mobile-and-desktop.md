# Shipping mobile and desktop: one frontend, one core, two shells

*Status: design of record for how this product is distributed and how much of
it is shared. Adopted 2026-08-18. Closes the open edge in `notes-ide.md` —
mobile distribution is decided — and **amends**
`.instructions/continuous-delivery-downloadable.md`, which assumed a GitHub
release and a one-line installer were the whole delivery path.*

The decision: **iOS and Android through the App Store and Play Store; desktop
as a `.dmg`, an `AppImage`, and an `.msi` you download.**

## Can all the source be shared?

The frontend, yes — completely. The Rust, mostly, and the part that cannot be
shared is blocked by the operating systems rather than by Tauri.

This was measured rather than assumed. Compiling against a real mobile target
on this machine:

| Crate | `cargo check --target aarch64-linux-android` |
|---|---|
| `hick-lang` | compiles, with no Android NDK installed at all |
| `hick-transcript` | compiles, no NDK |
| `hick-flow` | compiles, no NDK |
| `hick-condition` | compiles, no NDK |
| `hick-case` | compiles, no NDK |
| `hick-store` | compiles, no NDK — **but cannot run** (see below) |
| `hick-merge` | needs the NDK |
| `hick-structure` | needs the NDK (tree-sitter grammars are C) |
| `hick-token` | needs the NDK (libsodium) |
| `hick-literate` | needs the NDK — it pulls both of the above |

Five of the ten crates a notes IDE leans on are portable today, untouched, with
no Android toolchain present at all: the parser, the document model, transcript
derivation, the reactive graph, and conditionals.

**`hick-store` is the sixth, and it is a trap worth naming**, because it is the
one that makes this table less useful than it looks. It compiles for Android and
would fail the moment it ran: `git_backend.rs` shells out to the `git`
executable through `tokio::process::Command`, which compiles on every target and
exists on almost none of them.

**Compiling is necessary and not sufficient.** A crate that spawns a process
cross-compiles perfectly and dies at runtime on a platform that forbids
spawning. So the portable set is defined by two properties, and only the first
one a compiler can check:

1. it cross-compiles, and
2. it never spawns a process.

Audited for the second: `hick-lang`, `hick-transcript`, `hick-flow`,
`hick-condition`, and `hick-case` spawn nothing. `hick-store` spawns `git`.
`crates/hickory-cli/tests/portability.rs` keeps that true.

So the language, the document model, and transcript derivation are portable
with *zero* work. What the weave path drags in is more interesting than
whether it compiles:

- `libsodium-sys` ← `macaroon` ← **`hick-token`**, the capability tokens minted
  for containers.
- `zstd-sys` ← `zip`.
- C grammars ← **`hick-structure`**, the tree-sitter parsers behind structural
  navigation.

All are cross-compilable with the NDK, so none is a wall. But capability tokens
exist to serve **execution**, and a phone does not execute — which means the
phone would pay, in build complexity and binary size, for a capability it can
never have. (`hick-structure` is a different case: navigating code structure is
something a reader might genuinely want on a phone, so its C grammars are a cost
worth paying rather than one to design out.) That is a portability wart worth fixing when there is a mobile shell to
motivate it, and not before.

## What genuinely cannot be shared, and why

None of this is a Tauri limitation. Tauri v2 builds the same Rust for iOS and
Android; the platforms are what say no.

| Not on mobile | Why |
|---|---|
| `hickory-executor` / `LocalExecutor` | iOS does not permit a sandboxed app to `fork`/`exec`. Android can, but you cannot ship `python` and expect a document's cells to run. |
| `hick-term` | A PTY is a process. |
| `hick-lsp`, `hick-dap` | Both spawn language servers and debug adapters. |
| The `notify` inbox watcher | iOS has no arbitrary filesystem to watch: an app sees its own container plus documents the user grants through the picker, held as security-scoped bookmarks. Android's scoped storage is the same shape. |
| `git` the binary | There isn't one. Sync needs a pure-Rust implementation (already named in `notes-ide.md`). |

This is why `notes-ide.md` says the phone reads and captures and never
executes, and why `hick weave` — render from cached transcripts, mark never-run
blocks as never-run — is the mobile story. That was written before this was
measured, and the measurement agrees with it.

### The constraint and the store policy point the same way

App Review's guideline on executable code is the rule most likely to reject a
tool whose premise is *running the documents you write*. It does not apply
here, because on mobile the app cannot execute anything anyway.

The limitation that makes the phone a lesser client is the same limitation that
makes it shippable. Worth knowing before anyone proposes a clever way around
the first one.

## Git on a phone, which sync depends on

Notes sync through the user's own git remote (`notes-ide.md`), so git has to
work on a device with no git. Three findings, and the first is the one that
resizes the job.

**There is no remote git anywhere in this codebase today, on any platform.**
Every `git` invocation in the repository is local plumbing or reporting:

| Where | What it runs | Needed on a phone? |
|---|---|---|
| `hick-store/git_backend.rs` | `git init --bare` and object plumbing for `.hick/git/` | No — this is the internal version store, **not** the user's repository |
| `hickory-cli/agent_lineage.rs` | `git blame`, `git config` | Yes, eventually — this is the authorship half of `provenance-and-standing.md` |
| `hickory-cli/init.rs` | `rev-parse`, hook paths | No — `hick init` is a CLI act |
| `hick-term/src/git.rs` | `worktree add` | No — terminals are desktop-only |

Nothing clones, fetches, pushes, or merges. So sync is **a new capability**, not
a port of an existing one, and the phone is where it has to work first rather
than last.

**The library choice turns on push, not on purity.** `gix` is pure Rust and
would cross-compile with no C at all, which is the attractive property; its
weakness is exactly the operation sync cannot do without. `git2` (libgit2) has
mature push and fetch at the cost of C — which is a cost this build already pays
for tree-sitter and libsodium, so it buys less than it looks like it costs.
Recommendation: **whichever one can actually push, and prove it on a device
before building anything on top of it.**

Authentication should be **HTTPS with a token in the platform keychain**, not
SSH. There is no ssh-agent on a phone and no good place to put a private key, and
a token is a string a person can paste.

**The merge-driver design does not work on mobile, and that is a flaw in
`notes-ide.md`.** A git merge driver is a `.gitattributes` entry plus a config
line that invokes **a binary** — there is no binary on a phone, and no `git` to
read the config. So `hick-merge` has to be callable **in-process** by the sync
code, with the merge driver kept as the desktop and CLI spelling of the same
thing. One merge implementation, two ways of reaching it.

**Sync is not continuous on iOS.** An app does not run in the background at
will, so syncing happens when the app opens, when the user asks, and within
whatever background refresh the OS grants. A design that assumes a daemon will
not survive contact with the platform.

## The shape

```
apps/web/          one package
  index.html   →   the desktop editor
  site.html    →   hickorydocs.com
  mobile.html  →   the phone: read, capture, sync

apps/desktop/      Tauri shell, links the full engine
apps/mobile/       Tauri shell, links the portable core only
```

Two thin shells over one core, not one shell with branches through it. The
alternative — a single crate threaded with `#[cfg(mobile)]` — puts "this line
cannot exist on a phone" in a hundred places instead of one, and the compiler
only tells you which ones on the day you build for iOS.

`apps/desktop/src-tauri` already declares
`crate-type = ["staticlib", "cdylib", "rlib"]`, which is the shape Tauri's iOS
and Android builds need. The groundwork is there; what is missing is the split
of the engine, and the shell that consumes the smaller half.

## What the store decision costs

Stated plainly, because `continuous-delivery-downloadable.md` was written on the
assumption that none of it applied:

- **A paid relationship with two vendors.** Apple Developer Program, annually;
  Play Console, once. The first recurring cost this product has had.
- **A gate we cannot make green ourselves.** App Review is a human decision on
  someone else's schedule. "Push to `master`, a release appears" stops being
  true for mobile — the artifact is built automatically and *submitted*, and
  publication is someone else's verb.
- **Signing everywhere, not just the stores.** An unsigned `.dmg` gets a
  Gatekeeper refusal and an unsigned `.msi` gets a SmartScreen warning, and both
  read to a new user as "this is malware". Notarization uses the same Apple
  account the App Store does; Windows needs its own certificate.
- **A privacy disclosure for an app that collects nothing.** Both stores require
  the declaration regardless. The answer is "no data collected", and it stays
  true.
- **"Every platform we advertise is built and smoke-tested in CI" gets weaker.**
  CI can build and bundle for iOS; it cannot install to a device and run a
  document. The honest version for mobile is: built in CI, smoke-tested on a
  simulator, and verified by hand before submission.

## What does not change

- **Nothing in the product phones home.** No telemetry, no update check, no
  licence check. The stores report installs *to us* as a fact of being in a
  store; the app still says nothing to anyone. Same boundary as the marketing
  site's analytics in `local-only.md`, and worth keeping as precisely.
- **No server, no account, no payment.** The apps are free, and there is
  nothing to buy inside them. Sync is the user's own git remote.
- **`master` only, and the release channels stay.** Desktop delivery is
  unchanged: unstable on every push, stable on a human-chosen version bump.

## Open edges

- **Nothing has been built for a phone yet.** Everything above is a plan plus
  one compile check. No Xcode project, no Gradle project, no simulator run.
- **The `hick-token` dependency is unresolved.** Capability tokens drag
  libsodium into the weave path, which a phone should not be carrying. Fixing
  it means separating what weaving needs from what executing needs, inside
  `hick-literate`.
- **Blame does not work on a phone either.** `agent_lineage.rs` shells out to
  `git blame`, so the authorship half of `provenance-and-standing.md` is
  desktop-only until it goes through the same library sync will use. Nothing
  currently says so on the surfaces that show authorship.
- **Nothing was checked against an iOS target.** `aarch64-apple-ios` needs the
  Apple SDK, which this machine does not have, so the evidence above is Android
  standing in for both. The two agree on the constraint that matters — neither
  lets a sandboxed app spawn processes — but "it compiles for Android" is not
  "it compiles for iOS".
- **Store submission from CI is unproven.** Fastlane or `xcrun altool` from a
  GitHub runner needs signing identities in secrets — which is the first real
  secret this repository would hold, against
  `continuous-delivery-downloadable.md`'s claim that the only secret is
  `GITHUB_TOKEN`.
- **Two stores mean two review cadences and one version number.** Nothing here
  says what happens when iOS is approved and Android is not.
