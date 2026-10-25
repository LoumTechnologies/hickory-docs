# Profiles keep their measured scope

Given a captured or imported performance profile, function costs retain the
collector's units, measured interval, per-thread/profile scope and attribution.
Sampling observations are not calls. Inclusive costs overlap and are never summed
as independent costs. A frame's source line anchors a function; it is not a
measurement of that line's execution time. Missing samples do not prove zero cost.

Source identity, labels, configuration, provider/version and original reports are
retained with each run. Imported reports without original identity stay unknown.
Inline annotations require captured bytes matching the editor and admission
hashes. Browser anchors additionally require matching executed JavaScript bytes
or a provider source map whose original source contents match the saved source.
Unresolved/native functions remain visible without invented document locations.

The existing confined Python provider keeps its current permissions. Explicit
project-owned profiler commands run with the user's local permissions, as coverage
commands do. Browser collection samples the selected V8 page isolate; it does not
claim other workers, backend processes, GPU work or wall-clock workflow latency.

---

Last LLM verification:

- Date: 2026-10-25
- Reviewer: Codex
- Result: partially verified
- Requirement: the user's request for performance profiling inline in literate
  documents, continuing automated and human/AI manual coverage workflows;
  existing #65 function semantics remain authoritative for cProfile.
- Evidence: `crates/hick-profile`, `crates/hickory-cli/src/performance`, shared
  `source_mapping.rs`, and `apps/web/src/performance`.
- Executed checks so far: `cargo test -p hick-profile --test metrics`: 7 passed.
  Sampled weights, recursive inclusive de-duplication, evented self time,
  microsecond V8 deltas, missing time weights, cProfile call counts and malformed
  reports have independent numeric witnesses.
- Assumptions: provider weights represent its declared metric; measurements
  include sampling noise and profiling overhead. Native commands and source maps
  are project/provider inputs, not tamper-resistant attestation.
- Challenges/falsifier: displaying samples as calls, merging independent thread
  costs into one duration, attaching transpiled JavaScript to guessed TypeScript
  lines, or presenting absent samples as a speedup would violate this contract.
- Dependencies: the implementation files named above in this change. Changes to
  parsing, source binding or rendering require new focused verification.
- Human judgment: not recorded.
