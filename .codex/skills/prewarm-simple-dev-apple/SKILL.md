---
name: prewarm-simple-dev-apple
description: Prepare Hickory Docs on macOS for a developer who will soon run `just simple-dev apple`. Refresh master, warm the Rust and Swift build caches, and check for a desktop preview that could block the manual launch.
---

# Prewarm Simple Dev Apple

Work in the Hickory Docs checkout and shell environment the developer will launch
from. Build the artifacts there and leave opening the app to the developer.

## Update the launch checkout

1. Check the current branch and `git status --short --untracked-files=normal`.
   On a clean checkout, switch to `master` if needed and run
   `git pull --ff-only origin master`. If another worktree already has `master`,
   identify it with `git worktree list` and use it only if it is the intended
   launch checkout.
2. If the checkout has local changes, preserve them. Fetch with
   `git fetch origin master`. Continue if this is already `master` and HEAD
   matches `origin/master`. Otherwise report the blocker before touching the
   branch or working files. Verify HEAD equals `origin/master` before building.
   If local commits keep the revisions apart, report that blocker.

## Build the same artifacts the launch uses

Use the existing build directories. Cargo's dev stamp includes the full commit
and appends `+` when there are staged, modified, or untracked files, just as
`scripts/dev-supervisor.rs` does. Run from the repository root in Bash:

```bash
set -euo pipefail
just _dev-supervisor
commit=$(git rev-parse --verify HEAD)
status=$(git --no-optional-locks status --porcelain=v1 --untracked-files=normal)
test "$(git rev-parse --verify HEAD)" = "$commit"
export HICKORY_DEV_BUILD=1
export HICKORY_DEV_BUILD_INPUT="$commit${status:++}"
cargo build --locked -p hickory-cli --example simple_dev
```

If the default showcase at `.dev/simple-ui/code-review` is missing and the user
will use it, also build `cargo build --locked -p hickory-cli --bin hick` with
those same exports. Run `python3 examples/simple-ui/prepare.py --record` if the
destination is still absent. Leave existing workspaces in place.

In parallel with the Rust build, run `just simple-build apple` to prepare the
native resources and warm SwiftPM's `target/apple-simple-spm` cache. Honor an
explicit `DEVELOPER_DIR`. Otherwise use `xcode-select -p` if it points to full
Xcode. If it points to Command Line Tools, match the launcher's fallback by
selecting the lexicographically last `/Applications/Xcode*/Contents/Developer`
with an `usr/bin/xcodebuild` and setting `DEVELOPER_DIR` for the Swift recipe.
Report a missing full Xcode installation if neither path works.

Wait for both builds to finish and confirm the checkout still has the same
revision and dirty state. If helpful, repeat the same builds to verify cache
reuse. If Cargo is waiting for a build lock, identify the existing build and
leave it running. Report any stage that failed or is still blocked.

## Hand off the manual launch

Check `lsof target/dev-desktop.lock` for an existing desktop supervisor. Keep
the preview open unless the developer asks you to stop it. If the lock is held,
or their Codex Stop hook can queue an automatic preview, give them
`just dev-stop && just simple-dev apple` to run when ready. Otherwise give them
`just simple-dev apple`. Report the branch, commit, and completed builds.
