# Runtime observations retain their producing revision and occurrence

Given a host using `hick-observe`, when a provider supplies runtime evidence and
the host saves it, then retrieval preserves its run/event identity, captured source
bytes and digest, optional block/invocation/thread/task identities, emission and
caller locations, precision, and typed payload. Receive order makes no claim
about causality between threads. Missing emission metadata means unknown.

When output chunks are assembled for one stream/category, each displayed line
retains all contributing event IDs and byte ranges. One chunk can contribute to
several lines. Unicode bytes, CRLF, incomplete lines and terminal controls remain
lossless. Stderr alone never becomes an exception. Exceptions preserve reported
handling and separate throwing/catching sites. Logs, debugger observations and
profile measurements remain distinct payloads; profiles retain units, clock,
interval, scope and aggregation, without promising deterministic timing.

When a saved run is reopened after writer interruption, running becomes partial.
Completed, partial, failed and cancelled remain distinct; a failed newer run does
not replace historical evidence with success. Captured sources and saved event
prefixes cannot be rewritten. Missing and corrupt records are explicit retrieval
outcomes. Atomic saves leave either the previous or replacement record. The
exclusive writer lease prevents recovery from treating another active writer as
interrupted.

When the host requests retention, oldest unpinned terminal runs are deleted until
count/byte/age limits hold. Pins and running/corrupt records are exempt. Explicit
deletion can delete pinned runs. Event-budget loss sets truncation. New evidence
writes only `.hick/runs`, never Markdown, Git history or the transcript cache;
existing explicit `hick:output` and `hick:ingested` persistence is unchanged.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: GitHub #54 and #47; design
  `docs/specs/freeform/execution-observation-provenance.md` at c02ddde.
  Interpretation: Rust schema/provider/storage foundation; language providers and
  presentation are separately tracked by #55–#58, #64–#65.
- Evidence: `crates/hick-observe/src/lib.rs` (`Run`, `Observation`, `collect`,
  `output_lines`), `src/store.rs` (`LocalStore`), `examples/local_run.rs`.
- Test coverage: `src/tests.rs` exercises contributors, split Unicode/CRLF,
  terminal controls, repeated/recursive calls, interleaved writer identities,
  provider errors, truncation, status transitions, source edits, corrupt/missing
  records, writer leases, retention, pinning and Markdown isolation.
- Assumptions: host saves periodically and calls retention; default policy is
  100 unpinned terminal records / 256 MiB / 30 days. Source sizes are host bounded.
  Run/event IDs identify occurrences, not cache hits. Provider location metadata
  is reported evidence, not independently authenticated truth. Retention timestamps
  use host Unix milliseconds; profile clocks are explicitly separate.
- Alternatives: reusing the evictable transcript cache would lose history;
  automatic Markdown/Git persistence would alter authored evidence without intent.
- Witnesses/challenges: one line assembled from three writes including a split
  codepoint; one write spans multiple lines; interrupted run preserves partial
  output; changed source cannot overwrite captured bytes. Source-range validation
  rejects out-of-bounds attribution. Oldest retention cannot evict a pin.
- Exclusions and opaque coverage: no language instrumentation, source-map engine,
  UI selector, HTTP routes, agent policy or automatic production capture is added.
  Existing DAP/position mapping remains the provider integration point. Concurrent
  writes are tested as received events; real runtime scheduling is provider work.
  Tests simulate restart by releasing/reacquiring the lease; power-loss/fsync
  behavior is implementation-reviewed, not fault-injected. Unsaved observations
  are unrecoverable. No semantic correctness or benchmark timing claim is made.
- Falsifiers: changing saved source bytes, losing contributor identity, promoting
  stderr text to an exception, or evicting a pinned run would violate this contract.
- Dependencies: SHA-256 of `src/lib.rs`:
  `b8de97aa100f0e831350156843c93cfe582f185afa490821ced47a5e2823f8a3`;
  `src/store.rs`: `bf472d7923fe004fc0b696980e912825a8e32d0ee93a3f618d252dc9932d87e0`.
  Paths relative to `crates/hick-observe`; changes require review.
- Executed checks: `cargo test -p hick-observe` passed (6 tests);
  `cargo run -p hick-observe --example local_run` passed;
  `cargo clippy -p hick-observe --all-targets -- -D warnings` passed;
  `git diff --check` passed. Broad regression checks left to the user.
- Human judgment: not recorded.
