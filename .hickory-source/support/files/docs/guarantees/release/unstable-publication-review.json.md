<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/release/unstable-publication-review.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `c55c1346e1b0df3bec1d8eca68e0f570b15121da557f25da0e175d2b18cbc4a1`.

<book:entry path="docs/guarantees/release/unstable-publication-review.json">
{
  "date": "2026-10-25",
  "reviewer": "Codex",
  "review_status": "partially_verified",
  "base_revision": "988a8dab61bc349dfc2815c3d8c753a9e45a9ccc",
  "requirement": "Publish Apple Silicon Unstable DMGs from a Linux AI agent without GitHub Actions and offer signed in-app updates.",
  "decisions": [
    "Build committed source archives on an SSH Mac; keep signing credentials there and GitHub credentials on Linux.",
    "Replace the automatic release-deleting Unstable workflow with explicit manual publication.",
    "Retain immutable payload assets, publish unstable.json last, then move the rolling tag and edit release notes.",
    "Require consent before installing; preserve normal save/discard/detach handling by applying on next normal launch."
  ],
  "assumptions": [
    "One trusted, reachable Apple Silicon builder is shared by all publishers for this channel.",
    "Maintainer supplies SSH access, builder toolchains, signing credentials, and GitHub Contents write authorization.",
    "Installed app embeds the matching updater public key; the repository is publicly readable.",
    "Linux publisher clocks are sufficiently synchronized for increasing UTC timestamp versions."
  ],
  "exclusions": [
    "Stable release changes",
    "New CLI or non-Apple-Silicon release artifacts",
    "Forced relaunch",
    "Regression check monitoring"
  ],
  "executed_checks": [
    {
      "command": "just test-unstable-release",
      "result": "passed: 8 Python publishing/signature staging tests, 2 Rust policy tests, bash syntax checks"
    },
    {
      "command": "cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml --lib --offline",
      "result": "passed"
    },
    {
      "command": "temporary Linux harness compiling updater.rs with macOS guards removed",
      "result": "passed shared API/type check; does not compile macOS plugin internals"
    },
    {
      "command": "python3 scripts/release-unstable.py without HICKORY_MAC_BUILDER",
      "result": "refused with actionable configuration message before SSH or release writes"
    },
    {
      "command": "git diff --check",
      "result": "passed"
    }
  ],
  "failed_witness_and_correction": "A policy test using an all-decimal hash with leading zeros failed SemVer parsing. Generated versions now prefix hashes with g; the corrected test passed.",
  "unverified": [
    "Real Mac DMG build, code signing and notarization",
    "Real GitHub asset creation, replacement, and tag movement",
    "Native consent dialogs, invalid-signature rejection and bundle replacement on macOS",
    "Offline startup and concurrent folder-process behavior on a real Mac"
  ],
  "challenges": [
    "Separate builders can race because they do not share the lock.",
    "Manifest asset replacement is not atomic; a failed clobber can briefly or persistently remove it until republished.",
    "An abruptly killed publisher can leave the builder lock; recovery requires confirming no publish is active."
  ],
  "falsifier": "A payload failure still switching the manifest, a stable build initiating checks, an install without consent, or an accepted update failing on a real Mac would challenge the contract.",
  "human_judgment": "not_recorded",
  "dependencies_sha256": {
    "scripts/release-unstable.py": "0d37163859c3f17f573a956034f07722bebc32652e63c4f30e4e9263d3c8dbda",
    "scripts/build-unstable-macos.sh": "d5f0c12066eaf1fd1e1b5a0ec7e6c3e3b900663e46c3ebfde6b9a44c95d3b7ff",
    "scripts/dist-desktop.sh": "09bec96aee1da424531e9018b66a453879d90f63f33c4ae7828f53d9d2e20248",
    "scripts/tests/test_release_unstable.py": "7bd4920209782b10d81aef48d9c330c8609d0e5dccbdaab6c83ba7249121ee86",
    "apps/desktop/src-tauri/src/updater.rs": "13c161d8e53ffe38ba5021c50e8eebd357bbd554239c78ad4e9ec449a48b5979",
    "apps/desktop/src-tauri/src/lib.rs": "34521855d873130fa74f30c5f72185871a2891449405cb55a76361fa97bc10bc",
    "apps/desktop/src-tauri/Cargo.toml": "959f65ecc87286b6cc3d6d3d17f0ec16e934183a96410c7df3faf01f0efad3b1",
    "apps/desktop/src-tauri/Cargo.lock": "79c8e799087511a3a4a9703dc8d45ac7173f5e26ef79d2713c9870b7a19920ba",
    "apps/desktop/RELEASING.md": "bd6d53ce0372052c4d6147f5a02f33cf5e78e9b75640844ac55c81bdee901ca6",
    "justfile": "76f522bc3dfc0fb8d4c100589fbee7670debe4e0c383cc9b7d33d7fd20a1d180"
  },
  "removed_dependency": ".github/workflows/unstable-release.yml",
  "staleness": "Changes to any dependency invalidate this implementation review. Broader assumptions and macOS behavior remain unverified."
}
</book:entry>

</book:doc>
