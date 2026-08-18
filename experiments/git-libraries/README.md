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

# Check 3 — can it push to a remote?
#   Answered here by reading each API surface, which turned out to be enough:
#   two of the three do not expose the operation at all.
```

`aarch64-apple-ios` needs the Apple SDK and a macOS host, so Android stands in.
The two agree on the constraint that matters — neither lets a sandboxed app
spawn a process.

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

## The trap this harness exists for

`grit-lib`'s default build has **no HTTP transport at all** — `http-ureq` is
off by default. The first cross-compile run passed cleanly and was measuring a
library that cannot reach a remote. A compile check that does not enable the
feature under discussion is not evidence.
