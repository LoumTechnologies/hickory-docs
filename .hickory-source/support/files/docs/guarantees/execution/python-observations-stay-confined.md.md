<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/execution/python-observations-stay-confined.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `71b4d57939d3fd44b580da33a724b8e6d948194928d1a10d36afce1432330f1e`.

<book:entry path="docs/guarantees/execution/python-observations-stay-confined.md">
# Python observations stay confined and source bound

## Requirements

`hick observe FILE [--program main.py]` and the native Runtime observations
panel explicitly execute saved Python application code through the existing cell
sandbox. Missing sandbox support refuses execution. Network is disabled; no
agent shell/debug/verify capability is added. Unsaved editor content is not run.
Authored bytes are not changed. Native API paths must remain inside the workspace.

The reviewed CPython hook provider records actual text write, print, binary
buffer-write and flush contributions, thread/task identity and distinct traced
invocation IDs. Mapped frames and caller frames resolve through existing literate
mapping to immutable captured source. Mapping is line-level adapter-reported,
never a guess from output text. Literal source marker text is not interpreted as an annotation.
Throwing and unhandled exceptions are typed independently of stderr. Python
trace propagation is not mislabeled as another throw; catching sites are not
provided. Unmapped library/logging wrapper frames remain unknown; mapped caller
frames remain separate. Direct descriptor and native-library writes bypass these
hooks and are explicitly unsupported. This is CPython application execution,
not restricted Monty agent tracing.

Captures have a 1–120 second budget, 4096 event limit and 16 MiB artifact limit.
Timeout/interrupt cancels rather than completes; captured prefixes remain useful.
On Unix cancellation kills the provider process group. Partial/missing terminal
artifacts remain partial. Retained records use the shared run store, existing
history/inline/log views and selected context. Hiding the native panel does not
cancel its bounded request. Capture source is retained with its SHA-256.

## Assumptions and limits

Supported mechanisms are existing macOS Seatbelt and Linux bubblewrap cell
policies; Windows uses the existing AppContainer launcher when supported.
Platform policy limits remain those of the cell executor (including system
readability and permitted temporary storage), not a new VM isolation claim.
Runtime version is captured from `sys.version`; timing includes instrumentation
overhead. Trace/write hooks add workload-dependent cost and perturb scheduling;
no performance improvement is inferred. This reviewed instrumentation is not
tamper-resistant attestation against an application deliberately changing its
interpreter, hooks or writable scratch artifacts. No arbitrary-language support,
catching-site mapping, native write attribution or cross-thread causal ordering
is claimed. Output/raw artifacts can contain private application values and stay
local under the existing run retention policy.

## Verification

Focused tests in `observation::tests` execute real confined CPython, recursive
Unicode/binary output, threads, protected-home refusal, descriptor bypass,
literate source mapping, typed failure and timeout-prefix cancellation.
`user_python_capture_is_retained_and_refuses_workspace_escape` covers the real
native HTTP operation and shared store. Exact executed results and platform are
recorded in the issue handoff. Broad regressions are left to the user.

LLM review: implementation uses the existing sandbox wrapper and mapping instead
of an unrestricted interpreter fallback or agent capability. Human usability
judgment and cross-platform execution evidence are not recorded here.

Observed on macOS Seatbelt with CPython 3.9.6 (Clang 21): three paired
2,000-tiny-function-call trials measured untraced/traced seconds
`0.000371/0.117826`, `0.000356/0.116781`, `0.000357/0.118636`.
This call-heavy microbenchmark shows substantial hook overhead and is not an
application speed estimate. The identical correctness witness passed each pair.
Three focused real-provider tests passed; native HTTP capture/retention and
workspace-escape refusal passed. Cancellation retains the prefix; artifact reads
are bounded and Unix symlink-following is refused.
</book:entry>

</book:doc>
