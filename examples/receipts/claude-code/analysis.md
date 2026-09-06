---
date: 2026-08-24
---

# Checkout latency: before and after the pool fix



Priya's export, `data/checkout-latency.csv`, holds one row per four-hour
bucket from 2026-08-10 to 2026-08-23: the bucket's p95 checkout latency in
milliseconds and its request count. The windows below are the ones the
meeting asked for (the deploy on the 12th, the fix at noon on Friday the
21st), and the load-test window Priya warned about is excluded — the cell
says how many rows it dropped so the exclusion is visible rather than
silent.

I have the hourly latency export for August 10th through the 21st in the analytics bucket. I can compute before and after for whatever window you pick. One warning: the 18th has the load test in it, ten to noon, so anything that averages over the 18th will be wrong unless you exclude that window.

baseline  (08-10 .. 08-12)        p95 203 ms over 12 buckets
broken    (08-12 .. 08-21 12:00)  p95 489 ms over 56 buckets
after fix (08-21 12:00 .. 08-24)  p95 212 ms over 15 buckets
excluded: 1 bucket(s) in the 2026-08-18 load-test window



## Findings

The cell above is what computes the numbers; the fragments below are what
the rest of the pipeline can quote. They are typed, not pasted from the cell
— an exec's output is not selectable — so the `hick:expect` above is what
keeps them honest: change the data and the cell fails before anyone reads a
stale finding.

Checkout p95 was 489 ms between the 2026-08-12 deploy and the fix at noon on 2026-08-21, and 212 ms from the fix through 2026-08-23 (mean of four-hour p95 buckets, load-test window excluded).

The 2026-08-18 10:00–12:00 load-test bucket is excluded from every number here, as Priya asked.

Before the 2026-08-12 deploy the same measure was 203 ms, so the fix returned checkout to where it was, not somewhere new.
