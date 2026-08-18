# Can a phone do the phone's job?

`notes-ide.md` says the phone **reads and captures and never executes**, and
that `hick weave` — render from cached transcripts, mark never-run blocks as
never-run — is the mobile story. Everything supporting that had been established
by cross-compiling, which proves a crate *links* and says nothing about whether
it *works* inside an app container.

This probe runs the story on the operating system it is a claim about.

## What it checks

Ten assertions, not ten print statements — a probe that "ran fine" must not be
able to mean "printed something plausible":

1. the app container is writable at all;
2. a real `.hick` document parses (`hick-lang`);
3. the block model builds with nothing ever having run
   (`hick-literate::render::build_block_model` — the shared path behind both
   the CLI's `--json` and the app's render endpoint);
4. prose renders to HTML;
5. **an unrun cell is marked `never-run`** rather than shown as fresh — the
   load-bearing one, because a reader on a phone must never see a stale output
   presented as current;
6. a cached transcript renders that same cell as `ok`;
7. the cached output reaches the rendered block;
8. a real VTT transcript derives speaker turns (`hick-transcript`);
9. the speakers are the people who actually spoke;
10. nothing was executed to produce any of it.

## Re-running it

Needs a macOS host with Xcode. Built for a **simulator** target, bundled as a
real `.app`, installed, and launched — deliberately not `simctl spawn`, which
would sit outside the app sandbox and so could not answer the question.

```sh
cd experiments/ios-core
# Intel Macs run an x86_64 simulator; Apple Silicon uses aarch64-apple-ios-sim.
cargo build --release --target x86_64-apple-ios

# For a real device, pin the deployment target — without it the SDK default
# (currently 26.2) is baked in and the binary loads on nothing older:
#   IPHONEOS_DEPLOYMENT_TARGET=16.0 cargo build --release --target aarch64-apple-ios

APP=/tmp/CoreProbe.app
mkdir -p "$APP" && cp target/x86_64-apple-ios/release/ios-core-probe "$APP/CoreProbe"
# Info.plist: CFBundleExecutable=CoreProbe, CFBundleIdentifier=com.hickorydocs.coreprobe,
# CFBundlePackageType=APPL, CFBundleSupportedPlatforms=[iPhoneSimulator].
codesign --force --sign - "$APP"          # ad-hoc is enough for a simulator

DEV=$(xcrun simctl list devices available | sed -n 's/.*(\([0-9A-F-]\{36\}\)) (Shutdown).*/\1/p' | head -1)
xcrun simctl boot "$DEV"
xcrun simctl install "$DEV" "$APP"
xcrun simctl launch --console-pty "$DEV" com.hickorydocs.coreprobe

# The report is also written inside the container, so the sandbox write is
# confirmed from outside the sandbox rather than on the app's own word:
cat "$(xcrun simctl get_app_container "$DEV" com.hickorydocs.coreprobe data)/Documents/core-report.txt"
```

## What it found, 2026-08-18

Ten passes, no failures, on an iPhone 17 Pro simulator (iOS 26.1) hosted by
macOS 15.7.7. Written up in
`docs/specs/freeform/shipping-mobile-and-desktop.md`.

## What a simulator cannot tell you

The probe also spawns `/bin/echo`, and it **succeeds**. A simulator app is a
macOS process wearing an iOS runtime, so `posix_spawn` works there and does not
on a device. The no-spawn constraint that defines the portable set
(`crates/hickory-cli/tests/portability.rs`) is therefore **not** settled here,
and the probe says so in its own output rather than leaving a reader to infer
it. Only a provisioned build on real hardware settles that one.

`hick-literate` is linked here on purpose even though it is **not** in the
portable set: the question was whether the mobile story is reachable at all, and
`build_block_model` is where the never-run mark is made. Linking it is a
measurement, not a proposal for what `apps/mobile/` should depend on.
