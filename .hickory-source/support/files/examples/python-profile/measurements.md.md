<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/python-profile/measurements.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `9c6979365007870bbd676dcf0d5c7947eb50df26a1d746a8ad23ebcc1d2a6325`.

<book:entry path="examples/python-profile/measurements.md">
# Observed local profile experiment

On 2026-10-19, macOS 26.6 arm64 / CPython 3.9.6 / Clang 21, using the
existing Seatbelt cell sandbox without network. Three pairs per source variant
ran the same correctness assertions through the same runner, first without
trace/profiling instrumentation, then with stdlib cProfile (trace hooks disabled).
Durations are provider-monotonic seconds, main-thread scope. They include startup
inside the provider and output hooks; they exclude sandbox/process setup.

| Variant | Trial | Control elapsed | Profile elapsed | subtotal self seconds |
|---|---:|---:|---:|---:|
| Loop | 1 | .084156334 | .085222958 | .082915958 |
| Loop | 2 | .083365917 | .087740959 | .085318705 |
| Loop | 3 | .086384500 | .088970958 | .086445520 |
| Formula | 1 | .000355292 | .002379208 | .000025333 |
| Formula | 2 | .000338333 | .002301833 | .000023673 |
| Formula | 3 | .000350750 | .002433125 | .000025634 |

Every run asserted `15998000000` for 500 calls to `subtotal(8000)` and `45`
for recursive input 9. Profiles retained 500 total/primitive subtotal calls and
10 total / 1 primitive recursive calls. The algebraic rewrite removed the loop;
this observed fixture improvement is not a promised speedup. Profiling overhead
dominates the tiny formula variant, so compare control and instrumented results
separately. Inclusive costs overlap and must not be summed. Native/unmapped
functions remain unresolved; these are function aggregates, not per-line costs.

Reproduce the focused actual experiment (no timing threshold):

```sh
HICKORY_PROFILE_ARTIFACTS=/tmp/hickory-profile-artifacts cargo test -p hickory-cli --lib real_profile_optimize_rerun -- --nocapture --test-threads=1
```

In the native editor open `hotspot.md`, Runtime observations, choose generated
entry `hotspot.py`, enable cProfile and Run saved Python confined. Expand the
inline function table or hotspot table. Inspect/pin that evidence into context;
this is selection, not a request-delivery receipt. Optimize and save the source,
rerun correctness/profile, and compare retained runs. The old profile stays bound
to old source and current-source jumps are disabled after edits.

`apps/web/src/debug/fixtures/python-profile.json` is a selected-event replay of
the first actual baseline capture, including one unresolved native function. It
preserves measured values/event IDs/source bytes and explicitly marks incomplete
coverage. The browser acceptance host never claims browser Python execution.
No model task-success or human usability study is claimed in this experiment.

Baseline source SHA-256: `3b8c722898ea31c8691115b35fddabdc55082cd5e26aefcdd396257e8d95201d`.

Optimized source SHA-256: `118739b1123e9aad4a7cd397a827e7f60fefad2df401c8a5fee62e637f537bb8`.
</book:entry>

</book:doc>
