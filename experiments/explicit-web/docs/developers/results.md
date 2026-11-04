# Observed results — 2026-11-03

For reviewers deciding what this implementation demonstrates.

The reference application is runnable and the reusable libraries build and pack.
It provides accounts, durable sessions, private CRUD, forms and JSON, validation,
CSRF/CORS decisions, HTML encoding, parameterized SQLite transactions, optimistic
versions, CSS assets, bounded transport and explicit error handling.

## Functional evidence

On macOS with .NET SDK 10.0.401:

- tools/check.py passed its file-size/syntax checks and compiler warnings-as-errors.
- Checks/Program.cs passed transaction rollback, Unicode/NUL text, empty/nonempty
  blobs, double parameters, retained-row lifetime, same-session capability joins,
  Secure-cookie construction, typed routes, local redirects and HTML encoding.
- All three HTTP contract groups passed on both hosts: complete app/safety,
  session expiry/throttling, and real transport exception/timeout/chunked-limit
  refusals. The app group covers forms and JSON, isolation, concurrency, stale
  edits, logout/replay, CORS, invalid input, assets and restart persistence.
- The dev launcher started on an allocated port and stopped its listener on
  Ctrl+C. Its bound origin was accepted; a spoofed Host with a foreign origin
  was refused. The ready and launcher-lock files were removed on normal exit.
- The standalone export materialized through the public Hickory CLI. The final
  reconstruction comparison checks its output bytes and executable modes against
  the maintained parent outputs; see the final source verification note.

The automated browser witness was attempted but did not reach an application
page. The installed driver first lacked its expected Chromium revision; installed
Chrome and a cached Chromium then aborted because this environment denied macOS
Mach-service registration. The computer-use browser host/native window was also
unavailable. Browser form behavior is therefore **unverified locally**. The Linux
CI browser job is authored but has not run here. Linux/Windows native driver and
remote CI results are likewise unverified. Passing HTTP forms is not a substitute
for a successful browser witness or a production security review.

## Compiler observations

[Raw compiler receipt](../../evidence/compiler-final.json): all 12 selected unsafe
API calls were rejected with their expected diagnostics, while their untyped
controls compiled. Two deliberately surviving controls—null suppression and
wrong CORS configuration—also compiled. This is a selected API-constraint catalog,
not a population-wide bug-prevention percentage or a comparison to every existing
C# web framework and analyzer.

## Host-only runtime observations

[Final raw run](../../evidence/hosts-final.json): Release builds, five rounds per
workload/arm, 400 requests per group, four concurrent connections. That is 2,000
measured requests per arm per workload, 8,000 total, with no HTTP status errors.
Both arms share the same app, typed libraries, database implementation and data.

| Workload | Direct median req/s | Conventional median req/s | Direct difference | Direct / conventional median p95 |
|---|---:|---:|---:|---:|
| Health | 6,377 | 6,632 | −3.8% | 1.07 / 1.06 ms |
| Authenticated task list | 1,327 | 1,261 | +5.2% | 6.20 / 9.34 ms |

These are local observations with load generation competing for resources. An
[earlier run](../../evidence/hosts-initial.json) on different source hashes showed
+1.0% and +12.4% throughput differences respectively; it is retained, not pooled
with the final run. The variation argues against a strong speed claim. This
comparison measures direct hosting versus a conventional terminal delegate; it
cannot isolate the cost of the type-oriented APIs or show AI productivity gains.

No matched AI task trials have run. No time/token saving is established. Follow
[evaluation.md](evaluation.md) for the proposed paired task study, independent
acceptance criteria, failure reporting and investment rule.
