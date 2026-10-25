<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/developers/context-refresh-experiments.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `4c34e78a107fc9255641b3c090a682bf66d691dc0d462b8bd91a0cea3ecb4a6b`.

<book:entry path="docs/developers/context-refresh-experiments.md">
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
Manual scheduling records one scheduled manual update; measured human interventions
remain null unless a configured driver reports them.

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


The report now separates `exposure_success` from `behavioral_success`. Without a
driver, behavioral success and stale reasoning/edit counts are null. Fresh source
bytes in a request cannot establish that an agent used them. The five focused
harness tests include a deliberately stale driver that fails all 96 behavioral
witnesses despite successful exposure, and an isolated current-source driver
that passes all 96. Neither driver is a language model or a usability study.

To attach a local model/task adapter, pass an explicit JSON argv array:

```sh
PYTHONPATH=apps/agent/src apps/agent/.venv/bin/python \
  -m hickory_agent_worker.refresh_experiment --repetitions 3 \
  --driver-command '["/absolute/path/to/local-driver"]' \
  --output /tmp/hickory-behavioral-results.json
```

The driver runs in each isolated workspace and receives one JSON stdin record
containing `manifest`, `source` and `task`. It is invoked once before the edit
(`phase=plan`) and once after (`phase=act`). Return a JSON object with `answer`
(current source bytes, or null for deletion), optional `assumed_revision`,
`edit_attempts` (objects carrying `revision`), model/usage, tool calls, intervention
counts and reported cost. An absent answer, malformed response, timeout or failed
process is a failed behavioral witness, including deletion scenarios. The expected
answer is computed independently from the controlled source and is not supplied
as a task field. Unsaved content is supplied separately and stays off disk.

Both phase outputs and exact input manifests are retained. Usage/cost cover the
act phase and are driver declarations; elapsed local time is observed. This
adapter does not certify the driver's hidden provider requests. The built-in
harness never configures a paid provider, reads credentials, or sends telemetry.
Its driver task budget is zero: paid experiments require a separate explicitly
configured adapter enforcing an approved budget. This command is a local adapter
interface, not a mechanism for enforcing an arbitrary subprocess's network use.

Observed October 19, 2026: the default three-repetition run produced 144/144
exposure successes and 144 unavailable behavioral results. No refresh winner is
selected. Run the command above without `--driver-command` to reproduce that
coverage; request UUIDs, temporary paths and elapsed times vary.
</book:entry>

</book:doc>
