# Python profiles keep function scope and measurement semantics

## Requirements

The explicit saved-Python execution provider supports stdlib cProfile inside the
existing cell sandbox. Profile mode disables trace hooks; it grants no agent
execution capability. Run/source IDs and SHA-256 source bytes use the existing
observation store and literate mapping. Metadata captures CPython version,
environment, monotonic interval and main-thread scope. Unsupported functions
remain unresolved with their reported file/name; no model guesses attribution.

Function aggregates retain total/primitive calls, self seconds excluding subcalls,
and cumulative seconds including subcalls. Recursive reentry is included in total
calls. Inclusive costs overlap and are never summed as independent costs. Timing
is variable and includes profiling overhead; source mapping is independently
line-level adapter-reported. No line-cost or per-invocation timing is inferred.
The optional function payload is additive to the shared profile schema; existing
records without it remain valid.

Expandable function hotspot tables, inline function tables, source jumps and
selected context reuse actual event IDs and immutable source locations. Expanded
or pinned profile scope selects context; collapsed scope does not auto-disclose
all captures. Capturing context is separate from provider delivery. Source edits
remove current inline annotations and disable current-source jumps, while the
retained historical evidence remains inspectable. Authored Markdown is unchanged.

## Verification and limits

`real_profile_optimize_rerun_preserves_correctness_and_aggregate_scope` runs the
real literate hotspot before/after a source optimization, validates its correctness
assertions each time, checks recursion and mapped/unresolved functions, and
reports three paired uninstrumented-control/profile durations per variant. No
wall-clock threshold gates correctness. The control uses the same sandbox and
source without trace/profiler instrumentation. The browser fixture identifies
itself as replay of selected events from an actual local capture, never a live
browser Python execution claim. Browser interaction and context checks use that
same captured evidence; native execution checks use actual CPython.

cProfile covers the main thread, not all threads/tasks/native internals. Output
hooks remain active but timing trace hooks are disabled. Instrumented application
code can modify its own interpreter; this is not tamper-resistant attestation.
Environment/version and measured results are recorded in the issue handoff.
No human usability study or fixed performance benefit is claimed. Profiling
archives can expose application values and source; sharing remains explicit.

LLM review: source/metric derivation uses existing native provider mappings and
stdlib measurements; function aggregation and source precision remain separate.

Executed on 2026-10-19: real native before/after experiment passed with three
pairs per variant; results and immutable source hashes are retained in
`examples/python-profile/measurements.md`. Four focused frontend tests passed.
Chromium, Firefox and WebKit each passed the serial captured-profile interaction
flow (source jump, inline table, exact selected event IDs and stale-source refusal).
The browser build also passed TypeScript. Observed numbers are fixture evidence,
not a model task-success or human usability measurement.
The real native HTTP profile/retention/workspace-escape test passed. This check
caught and corrected shutdown flush trying to infer a caller frame after capture
closure. A failed provider process now prevents a completed claim even if a
terminal artifact exists. Bounded unattributed process stderr remains separate
from source-linked evidence. Interpreter-shutdown/late-background writes after
the program's captured runpy scope are unsupported.
