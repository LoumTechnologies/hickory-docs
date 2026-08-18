# Which Rust git library can sync notes?

Notes sync through the user's own git remote (`docs/specs/freeform/notes-ide.md`),
on a phone with no `git` binary. This directory is the harness that decides
which library does that, because the question is answerable by measurement and
the READMEs disagree with the code.

Three candidates, three checks. Each candidate is its own crate, outside the
root workspace, so one that fails to build cannot break `cargo check
--workspace`.

## Re-running it

```sh
# Check 1 — does anything on the path spawn a process?
#   A crate that shells out cross-compiles perfectly and dies on a device.
#   build.rs is excluded: build scripts run on a build machine, never on a phone.
python3 scripts/audit-spawns.py experiments/git-libraries/gix

# Check 2 — does it cross-compile for a phone?
rustup target add aarch64-linux-android
(cd experiments/git-libraries/gix && cargo check --target aarch64-linux-android)

# Check 2b — and for the other phone. Needs a macOS host and Xcode.
rustup target add aarch64-apple-ios
(cd experiments/git-libraries/git2 && cargo check --target aarch64-apple-ios)
#   Fails at `openssl-sys`, which has no cross-compiling pkg-config to find.
#   The feature that resolves it is the one to name explicitly:
(cd experiments/git-libraries/git2 && \
   cargo check --target aarch64-apple-ios --features git2/vendored-openssl)

# Check 3 — can it push to a remote?
#   Two of the three do not expose the operation at all, which settles the
#   comparison by reading. For the one that does, reading was not enough — see
#   ios-push/ below.
```

Android no longer stands in for iOS: both were measured. See
`docs/specs/freeform/shipping-mobile-and-desktop.md` for the tables.

## What it found, 2026-08-18

Full write-up in `docs/specs/freeform/shipping-mobile-and-desktop.md`. The
short version:

- **Only `git2` can push to a remote.** `gix` 0.86 has push *configuration* and
  no push *operation* — `src/remote/connection/` contains `fetch` and nothing
  else. `grit-lib` 0.5 has `push_local` (path to path) and a source comment
  putting `http(s)`/`ssh`/`git://` push in "a later phase".
- **The purity hope is false for all three.** Any HTTPS-capable build pulls C,
  because Rust TLS does: `ring` and `aws-lc-sys` both. Since this app needs the
  NDK and iOS SDK anyway for tree-sitter, "needs C" is not a differentiator.
- **All three keep HTTPS in-process and shell out only for SSH and credential
  helpers.** That is three independent confirmations of the same design
  constraint: on mobile, HTTPS with a token, never SSH, never a credential
  helper.

## `ios-push/` — the push, actually performed

`git2`'s `Remote::push` exists in the API. That is not the same as a push
landing from inside an app sandbox, so `ios-push/` is a probe that does it: one
commit in a repository inside the app's own container, pushed over **HTTPS with
a token** to a real remote, then read back over a second connection rather than
trusting the push's own report.

It is built for a **simulator** target, bundled as a real `.app`, installed, and
launched — deliberately not run through `simctl spawn`, which would sit outside
the app sandbox and so could not answer the question it exists for.

```sh
cd experiments/git-libraries/ios-push
# The simulator on an Intel Mac is x86_64; on Apple Silicon use aarch64-apple-ios-sim.
cargo build --release --target x86_64-apple-ios

APP=/tmp/PushProbe.app
mkdir -p "$APP" && cp target/x86_64-apple-ios/release/ios-push-probe "$APP/PushProbe"
# Info.plist: CFBundleExecutable=PushProbe, CFBundleIdentifier=com.hickorydocs.pushprobe,
# CFBundlePackageType=APPL, CFBundleSupportedPlatforms=[iPhoneSimulator].
codesign --force --sign - "$APP"          # ad-hoc is enough for a simulator

DEV=$(xcrun simctl list devices available | sed -n 's/.*(\([0-9A-F-]\{36\}\)) (Shutdown).*/\1/p' | head -1)
xcrun simctl boot "$DEV"
xcrun simctl install "$DEV" "$APP"

# Configuration arrives in the environment, so the token is never written to a
# file and never appears in the probe's output — it prints the length, not the value.
export SIMCTL_CHILD_PROBE_REMOTE=https://github.com/<you>/<scratch-repo>.git
export SIMCTL_CHILD_PROBE_USER=<your-login>
export SIMCTL_CHILD_PROBE_BRANCH=ios-probe-$(date +%s)
export SIMCTL_CHILD_PROBE_TOKEN=$(gh auth token)
xcrun simctl launch --console-pty "$DEV" com.hickorydocs.pushprobe
```

**What a simulator cannot tell you.** The probe also tries to spawn
`/bin/echo`, and it succeeds — a simulator app is a macOS process wearing an iOS
runtime, so `posix_spawn` works there and does not on a device. The no-spawn
constraint that defines the portable set is therefore *not* settled by this
harness, and the probe says so in its own output rather than leaving the reader
to infer it.

## The trap this harness exists for

`grit-lib`'s default build has **no HTTP transport at all** — `http-ureq` is
off by default. The first cross-compile run passed cleanly and was measuring a
library that cannot reach a remote. A compile check that does not enable the
feature under discussion is not evidence.
