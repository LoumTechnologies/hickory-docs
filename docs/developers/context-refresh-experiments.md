# Replaying context refresh policies locally

From the repository root:

```sh
uv run --project apps/agent python -m hickory_agent_worker.refresh_experiment \
  --output /tmp/hickory-refresh-results.json --repetitions 3
```

This writes 144 local replay records: eight scenarios × three encodings × two
schedules × three repetitions. Every record includes the exact validated first
and refreshed request manifests. Request IDs vary intentionally; pair by task,
repetition and scheduling. No remote provider is contacted.

Observed October 18, 2026: all 144 deterministic exposure/invalidation witnesses
passed. These results establish harness operation, not model correctness or a
preferred refresh policy. Cached/uncached/input/output tokens, actual cost,
stale reasoning/edit attempts and user effort are unavailable and remain null.
Byte counts are not token counts. Local assembly latency is not model latency.
Manual scheduling records one simulated intervention, not measured human effort.

`RefreshPolicy` is injectable through `MemoryContext(policy=...)` and supports
`queue_update(Update(...))`. Use `assemble` to project a new candidate while
retaining original history; no historical delivery receipt is rewritten. Generic
worker notices remain compatible with the existing policy interface. The queue
drains once per boundary; later edits remain for a subsequent request.

`compare_observations(cases, driver)` provides an independent paired dimension
for coordinates versus annotations/handles. Use the same log, debug or profiling
task and refresh policy in both arms; report behavioral success, mapping/tool
errors, correction requests, stale-handle refusals and user effort separately
from tokens/latency. Missing driver measurements remain null. This extension
does not itself claim an observed usability improvement.

Real model comparisons require a separately configured task/model driver and
explicit spending budget. Preserve its request and delivery evidence, observed
usage and behavioral task checks alongside these records; do not treat source
presence as proof that a model followed the changed requirement.
