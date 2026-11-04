# Evaluate the idea

For researchers deciding whether explicit, typed composition deserves investment.

First run the advertised app contract against both hosts:

```sh
python3 test_http.py
EXPLICIT_TEST_HOST=Baseline python3 test_http.py
```

The second spelling is POSIX shell syntax; on Windows set EXPLICIT_TEST_HOST in
the shell before running the same Python command. Both arms must pass before
interpreting runtime numbers.

## Compiler feedback

```sh
python3 evaluation/compiler_matrix.py --output target/compiler.json
```

The matrix compiles one valid consumer, then 12 selected API mistakes against the
real built libraries, requiring each expected diagnostic. Corresponding untyped
control APIs compile the same categories of mistaken call. Two surviving controls
(null suppression and wrong CORS configuration) must also compile. The selected
cases are an executable catalog of constraints, not a random sample of real web
bugs, a comparison to all ASP.NET analyzers, or a general bug-prevention rate.

## Hosting overhead

```sh
python3 evaluation/host_benchmark.py --rounds 5 --requests 400 --concurrency 4 --output target/hosts.json
```

Both arms use the same typed application, SQLite driver, security, response
adapter, logs, data seed and runtime. One invokes IHttpApplication directly; the
other uses a normal ASP.NET terminal delegate. Release builds, warmup, randomized
arm order, source hashes, per-request latencies, status errors, concurrency and
sample counts are retained. Report medians per round and tail latency beside
throughput. Local load generation competes with the server for resources, and
short runs are noisy. This comparison isolates hosting overhead; it does not
compare type safety to conventional MVC, measure maximum capacity, or establish
AI productivity. A small/no runtime difference is a valid result.

## Matched AI implementation trials

Do not claim a token/time saving until this has actual independent observations.
Freeze both starting repositories, toolchain, model/version, prompts, time budget,
network access and acceptance checks. For each task, randomize arm order and run
fresh sessions without sharing the other arm’s solution. Compare this framework
to a separately implemented conventional ASP.NET app meeting the same behavior,
not the host-only Baseline executable. Both arms need the same documentation
quality or a declared documentation confound.

Use at least these task families: add a protected aggregate; add a validated
field; add a read-only export; diagnose a CORS refusal; add an owner-scoped API;
fix a stale edit; fix an HTML-encoding defect; add a session-dependent page.
Prewrite acceptance and adversarial checks; the implementing agent must not be
the only correctness judge. Repeat across tasks and seeds. A pilot of 8 tasks ×
3 runs × 2 arms gives 24 paired observations, not broad statistical certainty.

Retain task IDs, source hashes, arm/order/seed, model, wall time, compile attempts,
compiler diagnostics, tool calls, actual input/output token counters, test outcomes,
reviewer acceptance and failures/timeouts. Missing token counters stay unknown,
never zero. Compute paired time/token differences only for independently accepted
solutions, and report failure rate across all attempts alongside them. Include
framework/adapter/documentation construction and maintenance cost separately.
A useful initial decision rule is equal or better acceptance/safety with at least
20% lower median accepted-task time or tokens; retain uncertainty and negative
results. That threshold is a proposed investment criterion, not a measured result.

If testing Hickory’s literate interface itself, use its in-app human opt-in
measurement workflow. Do not attribute framework gains to literate presentation
or vice versa without controlling which factor changed. No extra agent trials
have been launched by this implementation.
