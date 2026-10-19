# Host context observations are immutable and changes are source-backed

Given a retained host context session,
when Rust captures a saved file, an explicitly supplied editor snapshot or an
existing exact read receipt, or validates a derivation,
then it retains the exact returned UTF-8 bytes, excerpt digest, source identity,
whole-source revision hash and half-open byte range. Later observations never
replace those nodes. Capture records do not assert model delivery or exposure.

Given an old passage and a full current capture of the same source,
when their revisions differ,
then comparison reports an unchanged passage despite changes elsewhere, a unique
moved passage, changed text, or ambiguous correspondence. Duplicate matches,
including overlapping matches, are never resolved using old line numbers.
Capture-time uniqueness evidence is retained too: a unique current match cannot
resolve duplicates in the original snapshot. Missing uniqueness evidence remains
ambiguous rather than selecting a surviving copy. A partial current capture
cannot establish unique correspondence or absence in
omitted bytes and returns `cannot_check`. Comparison mappings index node bytes;
add the capture's source-span start to obtain source coordinates.

Given registered sources and retained references,
when the host polls changes,
then it hashes the supported source heads independently of Git and watchers,
returns ordered bounded pages, and distinguishes unavailable/deleted locators
from sources it cannot check. Dependency changes invalidate derived currentness
without changing derivation history. Restart, branch mismatch and event-retention
expiry refuse old cursors; explicit reconciliation returns fresh currentness and
a new cursor. Before reconciliation, reopened sources are not called unchanged.

Saved-file and editor identities remain separate even for equal bytes. A host
relocation preserves identity and historical locators; an unexplained missing
path is unavailable, not a guessed rename. Instruction metadata retains the
captured node, authority role, order and directory scope. Recomputing applicability
for another task location does not require instruction file edits and does not
promote ordinary captured material into instructions.

A retained session uses an exclusive writer lock and atomic, synced replacements.
Its 64 MiB runtime storage quota refuses new data rather than evicting history;
the event window retains at most 1,024 events. Large host imports use explicit
UTF-8-safe chunks with original source coordinates. References and idempotent
results survive restart; conflicting keys and reference replacement are refused.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified: store/capability witnesses executed; live UI editor
  capture integration reviewed in code, without a browser/model claim.
- Originating requirement: [#49](https://github.com/LoumTechnologies/hickory-docs/issues/49),
  parent [#47](https://github.com/LoumTechnologies/hickory-docs/issues/47), and
  `docs/specs/freeform/live-agent-context.md`; source/runtime slice of the v1
  contract established by #48.
- Implementation evidence: `crates/hick-context-store/src/lib.rs` (`Store`,
  `capture`, `import_read`, `import_chunks`, `derive`, `compare`, `reconcile`,
  `changes`, `currentness`, `instruction`, `applicable`, `Files`);
  `src/host.rs` (`Service`, `Editors`, capability negotiation and recovery).
  `crates/hickory-cli/src/python_agent.rs` installs the capability service,
  imports native read receipts and captures supplied editor JSON.
  `serve/acp/read_receipt.rs::persist` imports exact ACP read evidence.
  `hickory-agent/src/tools/read_file.rs` computes coordinates at the original read.
- Test coverage: `crates/hick-context-store/tests/lineage.rs` exercises real
  filesystem changes, CRLF/Unicode, overlapping reads and duplicate matches,
  instructions and task movement, distinct editor state, explicit rename,
  deletion/unavailability, original/current duplicate ambiguity, transitive
  currentness, restart/replay, expired cursors,
  quotas/chunks, capability operations and workspace/symlink confinement.
- Assumptions: adapters are host-owned and attest snapshot hashes/coordinates;
  a hash proves bytes, not identity, trust, semantic support or understanding.
  The filesystem has no global snapshot transaction. Currentness describes the
  last completed reconciliation; another edit can occur immediately afterward.
  A host rename notification is an identity witness; equal file content is not.
- Alternatives and falsifiers: old-line matching and first-duplicate matching
  would misidentify inserted/moved passages; the Unicode/duplicate witnesses
  challenge those approaches. Any historical-node mutation, forged exact
  derivation or disk fallback for a missing editor invalidates this guarantee.
- Exclusions and unsupported coverage: Rust supplies capabilities and evidence;
  Python still chooses inclusion, refresh, budget and policy (#50). Provider
  request/delivery manifests are not implemented by this slice. External
  adapters remain opaque about what their providers receive. Generated output
  receipts preserve exact observed bytes but report unavailable freshness until
  their provider is installed; this does not invent a second weave/source-map
  engine. Arbitrary external sources can implement `SourceAdapter` but have no
  implicit filesystem or network authority. Text-only runtime derivations reuse
  the protocol validator; producer-backed summaries/model/tool outputs need the
  authenticated production/request receipts of #50 and are refused here.
  Binary artifact storage, automatic rename discovery, UI refresh and push
  notifications are not provided. Instruction applicability describes retained
  captured material, not discovery or automatic injection of AGENTS.md.
- Evidence dependencies: inspected parent revision `ff01e02`; the exact source
  snapshots below and the issue handoff commit identify this implementation.
  Later changes to these dependencies make the recorded checks stale.
- Executed checks (2026-10-18): `cargo test -p hick-context-store --test lineage`
  passed 10 tests; `cargo test -p hickory-agent --lib tools::read_file::tests`
  passed 1 test; `cargo check -p hickory-cli --lib` passed;
  `cargo clippy -p hick-context-store --lib -- -D warnings` passed.
  `cargo test -p hickory-cli --lib serve::acp::read_receipt::` passed 2 tests,
  including persisted original-duplicate ambiguity. Touched-file formatting and
  staged whitespace checks passed. These commands used
  `CARGO_TARGET_DIR=target/context-lineage`; native checks reused the already built
  matching libsodium static archive via `SODIUM_LIB_DIR`, avoiding a second C
  dependency self-test run. No broad project regression suite was run; detached checks were not monitored.
- Human judgment: not recorded. No browser/model usability claim is made.

Implementation/test snapshot SHA-256:

```text
7476cb0889349ad4f4ce1a37ce8b930df6612e14720d18f104ec5505cf10f550  crates/hick-context-store/src/lib.rs
5932c70fa864e621d4e8da8bf3641015c90e8672ca8edc359f97c908676e3c4c  crates/hick-context-store/src/host.rs
f9aa128a4097db852e50ce90ad260647ab342d852bd6bd024a30309ac424d693  crates/hick-context-store/tests/lineage.rs
e51fdef00a2050107130de9b330179dfd6cc9c7a7d03103ad9b12f3d8e5325ee  crates/hickory-cli/src/python_agent.rs
dbd09885a90038c87cc54c46f8f9e9e4a9b661c5431a9306af67a9ef3f7ccd67  crates/hickory-cli/src/serve/python_agent.rs
147baeb5cfaac99a38adec38c30c164ca9463c1f7d0110692b4fa7b02857f414  crates/hickory-cli/src/serve/acp/read_receipt.rs
563564681d8c86c5adabf606f15f0f181857eef93c5877da1c9e3dd65587049e  crates/hickory-agent/src/tools/read_file.rs
10a60571712a7352b6d760b79e68d00456e29ac6b7a16cb8f2c3851152f2d541  crates/hickory-agent/src/tools/receipts.rs
```
