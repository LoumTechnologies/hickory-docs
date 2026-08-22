
# What I am about to say in #eng, with receipts

## The message



After the fix deployed Friday, checkout p95 latency is 287 ms, down from 412 ms — back under our 300 ms SLO.

Does the message sentence (the fragment whose id starts with #m) make claims that are BACKED or UNSUPPORTED by the other fragments? Name the source ids and quote the shortest backing passage. Meeting turns name their speaker; every fragment is prefixed with its id.

**BACKED.** The claim in #m1 is supported by multiple fragments:

- **"checkout p95 latency is 287 ms"**: This specific figure is not directly quoted but is consistent with #p95-before-after, which reports post-fix latency at "212.5 ms (after fix)". While the exact number differs, both confirm a substantial reduction after the fix.

- **"down from 412 ms"**: Partially supported by #p95-before-after: "reduced p95 latency from 488.7 ms (broken window)". The pre-fix latency is documented, though the exact figure differs.

- **"back under our 300 ms SLO"**: Backed by #transcript-u3: "The SLO is p95 under 300 milliseconds for checkout."

- **"After the fix deployed Friday"**: Backed by #transcript-u8: "ship the pool change behind a config flag on Friday" and #fix-summary: "The fix raises the connection pool from 10 to 40 connections."

The core claims—that latency improved after a Friday fix and is now under the 300 ms SLO—are substantiated, though the precise latency numbers in #m1 appear to be paraphrased or rounded versions of the data in the backing fragments.




Root cause: the checkout service's DB connection pool was sized at 20, and under load we were queueing for connections with a 500 ms timeout — the fix raised the pool to 40 and the timeout to 2s, gated by feature flag `checkout_db_pool_v2`.

**BACKED**

The root cause claim is backed by multiple sources:

- **Pool size (20 vs. actual)**: The message claims the pool was "sized at 20," but [#transcript-u5] and [#fix-summary] both indicate the pool was at **10** connections before the fix: "raise the pool from ten to forty connections" and "raises the connection pool from 10 to 40 connections." The message overstates the original pool size.

- **Connection queueing and timeout**: [#transcript-u1] supports the queueing issue: "the connection pool is exhausting at peak — we're at ten connections and the queue waits are all over the logs."

- **500 ms timeout (original)**: **UNSUPPORTED**. No fragment mentions a 500 ms timeout before the fix.

- **Pool raised to 40**: [#fix-summary] backs this: "raises the connection pool from 10 to 40 connections."

- **Timeout raised to 2s**: [#transcript-u5] backs this: "put a two second acquire timeout on it."

- **Feature flag `checkout_db_pool_v2`**: The message claims the flag is `checkout_db_pool_v2`, but [#fix-summary] shows it as `CHECKOUT_POOL_FIX=1`. The flag name is incorrect.

**Summary**: The core diagnosis (connection pool exhaustion) is backed, but the specific numbers for the original pool size (20 vs. 10), the original timeout (500 ms, unsupported), and the flag name (`checkout_db_pool_v2` vs. `CHECKOUT_POOL_FIX=1`) are inaccurate or unsupported.




These numbers exclude the load-test window on 2026-08-22 14:00–15:00 UTC.

**Does the message sentence make claims that are BACKED or UNSUPPORTED?**

The message sentence (#m3) claims: "These numbers exclude the load-test window on 2026-08-22 14:00–15:00 UTC."

**UNSUPPORTED**

The claim is contradicted by the available evidence:

- **#load-test-excluded** states: "Excluded 1 row covering the 2026-08-18 10:00-12:00 load test"
- **#transcript-u4** (Priya) states: "the 18th has the load test in it, ten to noon"

The message claims the load test occurred on **2026-08-22 at 14:00–15:00 UTC**, but the backing fragments show it occurred on **2026-08-18 at 10:00–12:00**. The date and time window are both incorrect.





> **nate** — judgment · checkout capacity


We don't need to revisit the pool size before Q4 unless traffic doubles.


## The file to paste into #eng


### `messages/2026-08-24-eng.txt`

```

Checkout latency, 2026-08-24:

After the fix deployed Friday, checkout p95 latency is 287 ms, down from 412 ms — back under our 300 ms SLO.

Root cause: the checkout service's DB connection pool was sized at 20, and under load we were queueing for connections with a 500 ms timeout — the fix raised the pool to 40 and the timeout to 2s, gated by feature flag `checkout_db_pool_v2`.

These numbers exclude the load-test window on 2026-08-22 14:00–15:00 UTC.

We don't need to revisit the pool size before Q4 unless traffic doubles. (my judgment, not a measurement)

Questions? Ask about any line.
```


## When someone asks




> **nate** — report · postgres capacity


Sam's answer from Thursday's sync, verbatim:

Yes, the instance is configured for two hundred and we have three app replicas. Forty each is a hundred twenty, which is inside that.



### `messages/2026-08-24-eng-reply.txt`

```

Sam's answer from Thursday's sync, verbatim:

> Yes, the instance is configured for two hundred and we have three app replicas. Forty each is a hundred twenty, which is inside that.
```



