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

This was measured rather than assumed, on both mobile targets — Android on a
Linux machine, iOS on a MacBook Pro (MacBookPro15,1, macOS 15.7.7, Xcode 26.3,
iOS 26.2 SDK) on 2026-08-18:

| Crate | `--target aarch64-linux-android` | `--target aarch64-apple-ios` |
|---|---|---|
| `hick-lang` | compiles, with no Android NDK installed at all | compiles |
| `hick-transcript` | compiles, no NDK | compiles |
| `hick-flow` | compiles, no NDK | compiles |
| `hick-condition` | compiles, no NDK | compiles |
| `hick-case` | compiles, no NDK | compiles |
| `hick-store` | compiles, no NDK — **but cannot run** (see below) | compiles — same trap |
| `hick-merge` | needs the NDK | not checked |
| `hick-structure` | needs the NDK (tree-sitter grammars are C) | not checked |
| `hick-token` | needs the NDK (libsodium) | not checked |
| `hick-literate` | needs the NDK — it pulls both of the above | not checked |

Five of the ten crates a notes IDE leans on are portable today, untouched, with
no Android toolchain present at all: the parser, the document model, transcript
derivation, the reactive graph, and conditionals. **All five compile for iOS
too**, needing nothing beyond `rustup target add aarch64-apple-ios` and an Xcode
that was already installed — the iOS column cost five commands and found nothing.
The last four rows were not re-run for iOS: what they need is a C toolchain, and
Xcode is one, so the interesting question there is binary size rather than
whether it builds.

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

**The library choice was measured, not argued.** The harness lives in
`experiments/git-libraries/` with instructions for re-running it; results as of
2026-08-18:

| | Push to a remote | Spawns on the HTTPS path | Cross-compiles with TLS | Licence |
|---|---|---|---|---|
| `gix` 0.86 | **No.** Push *configuration* exists (`push_specs`, `Direction::Push`); the push *operation* does not — `src/remote/connection/` contains `fetch` and nothing else | No. Spawns only for SSH (`gix-transport`) and credential helpers (`gix-credentials`) | Needs C: rustls pulls `ring` or `aws-lc-sys` | MIT/Apache |
| `git2` 0.20 | **Yes** — `Remote::push`, and it was performed (below) | No. Spawns only for credential helpers (`cred.rs`) | Needs C: libgit2 and OpenSSL. **iOS needs `vendored-openssl`** (below) | MIT/Apache |
| `grit-lib` 0.5 | **No.** `push_local` moves objects between two local paths; a source comment puts `git://`, `http(s)`, and `ssh` push in "a later phase" | No. HTTPS is in-process via `ureq`; SSH, hooks, filters, and signing spawn (38 sites in the lib) | Needs C: `ring`, via `ureq` | MIT (`grit-lib`); **`grit-cli` is GPL-2.0 and unusable here** |

Four findings, in order of how much they change the decision.

**1. Only `git2` can push to a remote today.** This corrects an earlier reading
of grit's README, which lists "clone, fetch, pull, push" among the CLI's
capabilities; the library's own source says otherwise, in a `TODO` naming
exactly the missing pieces — "receive-pack handshake + report-status parsing +
credential helpers". Sync is push or it is nothing, so this decides it.

**2. The purity hope is false for all three.** Any HTTPS-capable build pulls C,
because Rust's TLS stacks do: `ring` and `aws-lc-sys` both ship C and assembly.
Swapping gix's rustls provider from aws-lc to ring changes which C, not whether.
Since this app already needs the NDK and the iOS SDK for tree-sitter, **"needs
C" is not a differentiator** — which removes the main reason to prefer a pure
implementation and leaves maturity and push as the only criteria that matter.

**3. Three independent confirmations of the same constraint.** Every candidate
keeps HTTPS in-process and shells out only for SSH and credential helpers. On
mobile that means: **HTTPS with a token, never SSH, never a credential helper**
— a conclusion reached earlier from one library's behaviour and now measured
across three.

**4. `grit-lib`'s 38 spawn sites are desktop concerns, not blockers.** Hooks,
clean/smudge filters, GPG signing, and `sh` — none of which a phone would run.
Worth knowing, not disqualifying, if it ever grows remote push.

So: **`git2`, behind a narrow trait**, the way `Executor` already isolates how a
cell runs. It is the only one that can do the job, and the trait is what makes
that reversible when gitoxide or grit lands push — which is the likelier future
than libgit2 going away.

### iOS, measured: one feature flag, and then a push that lands

**`git2` does not cross-compile for iOS out of the box, and the blocker is
OpenSSL.** `cargo check --target aarch64-apple-ios` fails in `openssl-sys`,
which looks for a system OpenSSL through `pkg-config` and is told
"pkg-config has not been configured to support cross-compilation". Nothing about
libgit2 is the problem; the TLS stack is.

**`vendored-openssl` resolves it completely.** With
`--features git2/vendored-openssl`, the build compiles OpenSSL 3.6.3, libssh2,
and libgit2 1.9.6 from source for `arm64-apple-ios` — `lipo -info` reports
`arm64`, and `otool -l` reports `LC_BUILD_VERSION platform 2` (iOS), `minos
26.2`. So the answer is one feature flag and about a hundred seconds of C
compilation, not a wall. Note that `git2`'s default features also build libssh2,
which a phone will never use; dropping it is a size question for whoever builds
the mobile shell, not a blocker.

**The push was performed, not read about.** `experiments/git-libraries/ios-push/`
is a probe bundled as a real `.app`, installed on a booted iPhone 17 Pro
simulator (iOS 26.1) and launched — not run through `simctl spawn`, which would
sit outside the app sandbox. From inside the app's own container it made a
commit, pushed to a GitHub repository over **HTTPS with a personal access token**
(no SSH, no credential helper), and read the ref back over a second connection:

```
CONNECT: ok, remote advertised 1 ref(s)
PUSH-TRANSFER: 3/3 objects, 256 bytes
PUSH-STATUS: refs/heads/ios-probe-… accepted
VERIFY: remote refs/heads/ios-probe-… = c16e5a23… (local commit was c16e5a23…) — MATCH
```

Confirmed independently from the host with `git ls-remote` and the GitHub API:
the branch exists, the commit's tree holds the one file, and the sandbox refused
nothing. The credentials callback was offered exactly
`CredentialType(USER_PASS_PLAINTEXT)` and nothing else was needed. Sync can be
built on this.

Two things that probe cannot tell you, and both matter:

- **It is a simulator, so it does not test the no-spawn rule.** The probe also
  spawns `/bin/echo`, and it *succeeds* — a simulator app is a macOS process
  wearing an iOS runtime. The constraint that defines the portable set is still
  unverified on a device, and the probe prints that verdict itself rather than
  leaving a reader to infer it.
- **libgit2 does not go through `NSURLSession`.** It opens BSD sockets and speaks
  TLS through OpenSSL, so App Transport Security never sees the connection —
  which is why this worked with no `Info.plist` exception, and also means ATS is
  not protecting it. Worth knowing before someone concludes the platform vetted
  the traffic.

### The trap the harness exists for

`grit-lib`'s default build has **no HTTP transport at all**: `http-ureq` is off
by default. The first cross-compile run passed cleanly, with no C toolchain, and
was measuring a library that cannot reach a remote. A compile check that does
not enable the feature under discussion is not evidence — and that is the kind
of clean-looking result that would have survived all the way to a device.

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

**Measured on 2026-08-18, and the split is the whole job.**
`cargo tauri ios init` against `apps/desktop/src-tauri` **succeeds** — it writes
an Xcode project, plists, entitlements, a Podfile, and a launch storyboard under
`gen/apple/` (already gitignored), using an `xcodegen` and `cocoapods` that were
already installed. That is worth knowing precisely because it is misleading:
`init` generates scaffolding and compiles nothing.

`cargo build --lib --target aarch64-apple-ios` on that same crate **fails**, and
the wall is not where the table above would lead you to look:

```
error: could not compile `termios`  (28 errors)
error: could not compile `ioctl-rs` (12 errors)
   unresolved import `os::target`   — no iOS arm in termios
   cannot find value `TIOCMGET`     — ioctl-rs has no iOS
```

`hickory-cli → hick-term → portable-pty → serial → serial-unix → termios`.
The PTY's own dependencies have no iOS support at all, so **the compile aborts
before it can reach the executor, the LSP, or the DAP** — the other three things
that cannot exist on a phone. That is the argument against
`#[cfg(mobile)]` made concrete: threading it through would surface these one at
a time, each on a separate build, and the compiler would only ever name the first.

The fix is `apps/mobile/`, a shell that links the portable core and never
mentions `hickory-cli`. Not attempted here; it is the deliberate piece of work
the shape above describes.

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
  account the App Store does; Windows needs its own certificate — and Windows is
  the awkward one, because since 2023 that certificate's private key may not be a
  file at all, which makes the Windows channel a signing *service* rather than a
  secret. The full account-and-identity list, measured, is in
  `.instructions/continuous-delivery-downloadable.md`.
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

## Settling the Apple-only claims

Everything here was originally measured on Linux, and the claims that needed
Apple hardware were collected as a prompt in
`docs/developers/verify-on-apple-hardware.md`. **That prompt was run on
2026-08-18** on a MacBook Pro (MacBookPro15,1, Intel Core i7, macOS 15.7.7,
Xcode 26.3). What it settled is written into the sections above and into the
guarantees it touched; what it could not settle is in the open edges below.

The two results worth carrying forward, because they changed something:

- **The download-origin reader was wrong**, in a way only a real browser download
  could show. Fixed, and covered by tests against captured bytes —
  `docs/guarantees/authoring/ingest-keeps-the-original-bytes.md`.
- **macOS's shells do not emit OSC 7** to a terminal that answers truthfully
  about what it is, so terminals-in-the-file-tree degrades to its fallback on the
  platform most likely to run this app —
  `docs/guarantees/terminal/a-session-appears-where-it-is-working.md`.

## Open edges

Rewritten 2026-08-18 after `docs/developers/verify-on-apple-hardware.md` was run
on Apple hardware. What was settled has moved into the sections above; what is
below is what is still owed.

- **Nothing has been built for a phone yet.** Still true, and now precisely
  bounded: the portable core compiles for iOS, `git2` pushes from a simulator,
  and `cargo tauri ios init` writes an Xcode project — but no shell exists that
  links only the portable half, and the desktop shell cannot compile for iOS.
  There is still no Gradle project and no Android run of anything.
- **The no-spawn constraint is unverified on a device.** It is the property the
  whole portable set is defined by, and a simulator cannot test it: a simulator
  app *can* `posix_spawn`, which `experiments/git-libraries/ios-push` demonstrates
  by doing it. Settling this needs a provisioned build on real hardware.
- **The `hick-token` dependency is unresolved.** Capability tokens drag libsodium
  into the weave path, which a phone should not be carrying. Fixing it means
  separating what weaving needs from what executing needs, inside `hick-literate`.
- **Blame does not work on a phone either.** `agent_lineage.rs` shells out to
  `git blame`, so the authorship half of `provenance-and-standing.md` is
  desktop-only until it goes through the same library sync will use. Nothing
  currently says so on the surfaces that show authorship.
- **Sync's design is unbuilt, though its foundation is now proven.** `git2`
  pushed to a real remote from inside an app sandbox over HTTPS with a token. The
  trait that isolates it, the merge path called in-process, and the keychain the
  token lives in are all still to write.
- **The last four crates were never checked against iOS.** `hick-merge`,
  `hick-structure`, `hick-token`, and `hick-literate` need a C toolchain and
  Xcode is one, so the open question is binary size rather than feasibility — but
  it is open.
- **Store submission from CI is unproven, and the identities do not exist.** This
  machine holds one `Apple Development` certificate and no `Developer ID
  Application`, no `Apple Distribution`, and no provisioning profiles, so nothing
  here can be notarized or submitted today. What each one requires is now written
  down in `.instructions/continuous-delivery-downloadable.md`.
- **The Windows signing key cannot be a CI secret**, which is a bigger change to
  the delivery plan than the Apple side. See the same file.
- **Two stores mean two review cadences and one version number.** Nothing here
  says what happens when iOS is approved and Android is not.
- **The desktop `.dmg` was built and run, and the release script has a rough
  edge.** `scripts/dist-desktop.sh` produced
  `hickory-docs-0.1.0-x86_64-apple-darwin.dmg` (15.5 MB); it mounts, carries the
  `Applications` symlink, and the app inside runs a document successfully
  (`status: ok`, exit 0, expectation matched) through the same engine `hick up`
  uses. But the script **fails on a developer's Mac** and only succeeds when
  `CI=true`, because Tauri passes `--skip-jenkins` to `bundle_dmg.sh` only in
  that case, and without it the script drives Finder over AppleScript and times
  out. CI sets `CI=true`, so the release channel works; a local run does not,
  which is why nobody had noticed.
- **Gatekeeper's refusal is confirmed, and the wording is Apple's.** The
  unsigned bundle is rejected by `spctl` with `source=no usable signature`, and a
  quarantined copy launched by `open` is created, App-Translocated to a
  randomised read-only path, and then **held without ever initialising** — no
  sockets, no window, waiting on a user decision. The dialog's text could not be
  photographed on the machine this ran on, but it is composable verbatim from the
  system's own localized strings
  (`/System/Library/CoreServices/CoreServicesUIAgent.app/Contents/Resources`):
  headline `Q_HEADLINE_SUNFISH_NOT_VERIFIED` → **“Hickory Docs” Not Opened**,
  detail `Q_DETAIL_CASPIAN_UNVERIFIED` → **Apple could not verify “Hickory Docs”
  is free of malware that may harm your Mac or compromise your privacy.**,
  buttons **Move to Trash** and **Done**. The claim that an unsigned build "reads
  to a new user as malware" is therefore not a figure of speech: *malware* is the
  word Apple's own dialog uses, and the default button offers to bin it.
- **Nothing was seen in a running window.** The app was driven over its own HTTP
  API rather than clicked, because the session this ran in could not present GUI
  windows at all — three separate applications launched with none. How the app
  looks and feels on macOS is still unverified.
