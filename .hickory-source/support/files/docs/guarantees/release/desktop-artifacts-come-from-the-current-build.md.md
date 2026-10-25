<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/release/desktop-artifacts-come-from-the-current-build.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `87b1a9df13f43f47e2cae18472f249f52c3b3da9fb84e0aa94699b0e162d081e`.

<book:entry path="docs/guarantees/release/desktop-artifacts-come-from-the-current-build.md">
# Desktop collection excludes unchanged cached installers

Given an isolated desktop bundle build, when `scripts/dist-desktop.sh` collects
installable files, then it selects only files whose existence or observed
identity changed since the invocation's pre-build snapshot. An unchanged cached
installer is never renamed as the current version merely because it remains in
the bundle directory. No changed installable file, or multiple changed files of
one format, refuses collection before replacing the previous dist selection.

This is a local cache freshness guard. File size, modification time, device/inode
and SHA-256 distinguish observed changes; they do not authenticate the producer,
attest the embedded version, establish reproducibility or coordinate simultaneous
builders in the same directory. Use an isolated worktree/output for a witness.
A failed compile/bundle does not become a successful package or launch claim.

## Verification record — 2026-10-22

Originating requirement: issue 72/#76 requires current revision-bound packaged
witnesses and truthful release/install status. Inspection found collection of
every cached bundle, including old formats not rebuilt by the current invocation.

LLM implementation review: partially verified for this bounded guard. Inspected
`dist-desktop.sh` and `desktop-bundle-inventory.mjs` before/after collection. The
helper ignores symlinks, inventories installable files, checks snapshot ownership,
rejects ambiguous formats and emits NUL-delimited paths. Hashing detects content
replacement even when other file metadata is preserved. Unchanged future-dated
files are excluded. This does not prove that a changed file came from the compiler.

Executed checks: three focused stub-build tests in
`scripts/tests/test_desktop_bundle_inventory.py` passed using the actual shell
collector and Node helper: fresh deb plus future-dated cached AppImage, no fresh
installer, and two fresh debs. The last two preserve the previous selection and
refuse. `bash -n`, `node --check` and whitespace checks passed. No compiler,
installer or upload is invoked by those tests. Current actual package/launch
verification remains pending. Human judgment: not supplied.

Dependencies (changes stale this review):
- `scripts/dist-desktop.sh` SHA-256: `60515552afc7adbde344d6ca56d98d1e1598705df6e4907fa441cb5f5fc5df63`.
- `scripts/desktop-bundle-inventory.mjs` SHA-256: `df5c5ade6e9f4814d3eaa67aaa934727baef8fa9276b4e4f37ba87563e9d9437`.
- `scripts/tests/test_desktop_bundle_inventory.py` SHA-256: `6dcd148ef12c0beadc593ee7dab622320f92f8c49ca9912c4c830b20c79e9f89`.
</book:entry>

</book:doc>
