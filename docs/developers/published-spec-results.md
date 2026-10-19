# Publish executable-spec verification

`publications/selected.json` is the explicit public-material allowlist. Each entry
approves its output and pins the reviewed source, dependency and asset hashes.
Do not add private sessions, credentials or unreviewed outputs. Review changed
inputs before updating `reviewed_inputs`; do not automatically refresh that field.

Generate the same artifacts locally with:

```sh
cargo build -p hickory-cli --bin hick
python3 scripts/publish-spec-results.py --hick target/debug/hick --output /tmp/spec-results
```

The command runs the existing native weave/test flow in an isolated temporary
copy of only the selected inputs. It writes `index.json` and per-document snapshots
on verification failure as well as success, then returns nonzero for failed,
stale, unavailable or not-run evidence. Source/dependency hashes and exact command
transcripts identify the scope. Requirement links use explicit zero-based indices
into the resulting execution-cell list; absent/unasserted witnesses are unavailable.

CI retains these files as `executable-spec-results-<commit>`. The existing site
workflow generates them into the static site's `publications/` directory and
publishes even failed verification evidence before reporting the failed check.
Open `snapshot.html?index=publications/index.json` on that host. Individual results
use the existing snapshot viewer and need no local engine.

Document recording/replay identities and keys come from CLI evidence when
available. Other cache-hit origins remain unavailable; executing a verification
command does not establish that every cell ran live. Passing deterministic budget
allocation checks does not establish real network timing or total prose coverage.

Monitor detached regressions yourself with `just checks-status` or GitHub Actions.
Generation/hosting is wired here; a successful local result is not proof that a
new site revision has deployed.
