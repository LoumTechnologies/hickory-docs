# Execution observations and source provenance

Status: proposed. Records the October 18, 2026 discussion; presentation and
retention defaults below are recommendations to validate, not settled policy.
Implementation tasks belong in GitHub issues, not an independent TODO list here.

## One relationship, multiple views

A runtime observation may be displayed beneath its originating statement in a
literate document, or in a log with ribbons/braces to its source. Both views must
consume the same observation identity and provenance mapping. Execution provenance
is distinct from source-byte derivation, model context and declared citations.

A captured stdout/stderr chunk alone has no trustworthy statement location.
Reuse the existing generated-file-to-document source mapping once a runtime or
instrumentation provides a location. Otherwise retain block-level or unknown
attribution and expose the precision; never guess from the last breakpoint or
matching output text. Trace support is explicit per runtime/provider.

## Event and invocation identity

Record run identity, captured source revision, executable block, stream/category,
event order, time where available, source location and precision, thread/task and
invocation identity where supplied. Preserve emitting location separately from
calling context: a logger wrapper and its caller answer different questions.

Output events are chunks, not necessarily lines. A write can contain multiple
lines; several statements can contribute to one displayed line. Preserve byte
contribution ranges when assembling a line. Partial lines, CRLF, multibyte text,
terminal control sequences and concurrent streams must not fabricate a single
origin or a total execution order stronger than what was observed.

Repeated calls are occurrences within a run, with per-invocation grouping and an
expandable timeline. Missing invocation IDs remain unknown. Text equality never
establishes that two events came from the same invocation.

Exceptions are typed observations, not an interpretation of stderr text. Preserve
reported throwing and catching sites separately where the runtime supports them.
A caught exception may produce no output; stderr may contain no exception.
Unhandled, caught and unknown handling states need distinct representation.

## Run history and document presentation

Recommended starting point: inline results select one run, normally the latest,
with explicit source revision and current/historical/stale/running/partial/failed
state. A newer failed run shows its failure and partial output; an older successful
run must not remain presented as current success. Editing source does not make an
old observation vanish or become evidence for the new bytes.

Earlier runs are available through a selector/history, explicit pinning and
comparison. Avoid interleaving all runs without labels. Define retention,
truncation, deletion, restart recovery and artifact-missing behavior rather than
using an evictable cache as the only durable evidence. Do not commit unlimited
logs automatically. The inline and log views select the same run/event set.

## Markdown persistence

Current behavior is not uniformly transient: `crates/hickory-cli/src/output.rs`
refreshes same-document derived `hick:output` regions for `show="output"` cells;
explicit recordings can be promoted into `hick:ingested` by
`crates/hickory-cli/src/ingest_recording.rs`. A transcript cache also exists.
Debugger observations and generated document outputs must remain distinguishable.

Recommended default for new live-debug observations: store run artifacts outside
authored Markdown and render them as a view. An explicit retain/pin/capture action
can promote selected observations into document-owned evidence. Reconcile this
with existing persisted outputs; do not silently remove that feature. Retained
results keep producing-run/source identity, stale state and verification scope.

## Context integration

Expanded observations can become agent context using the generic context-lineage
API in `live-agent-context.md`. Their lineage is run -> event -> captured code
revision -> document mapping. Expanding output does not turn it into human-authored
source or evidence that arbitrary prose is true. Include selected occurrences,
source/run identity, truncation and precision in a request manifest. Scrolling is
not a context-selection policy; expanded scope is stable until deliberately changed.

## Required witnesses

Exercise one statement printing multiple lines, multiple statements composing one
line, repeated calls, nested calls, loops, recursion, partial and failed runs,
concurrent writers, Unicode/CRLF and terminal escapes. Verify mappings through
literate fragments and source edits, distinguish wrapper/caller, retain unknown
locations, and prove that inline/log views refer to identical event IDs. Historical
observations must remain tied to their captured revision after source changes.

No new runtime tracing or run-history implementation was performed for this design.

## Foundation implementation (issue #54, October 18, 2026)

`crates/hick-observe` now defines the versioned evidence model, provider interface,
lossless output contributors and independent local run store. Its README describes
lifecycle, retention, pinning, deletion, recovery, coordinate conventions and host
integration. The executable `local_run` example exercises provider → durable run →
retrieved output. This is the storage/schema foundation, not language tracing or
inline/log UI integration; those remain the dependent issues. The historical
proposal above remains the record of presentation recommendations.
