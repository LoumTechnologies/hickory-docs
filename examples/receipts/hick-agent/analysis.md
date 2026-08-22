---
date: 2026-08-24
---



## Context

Priya's warning about the load-test window:

I have the hourly latency export for August 10th through the 21st in the analytics bucket. I can compute before and after for whatever window you pick. One warning: the 18th has the load test in it, ten to noon, so anything that averages over the 18th will be wrong unless you exclude that window.




## Analysis

Baseline (2026-08-10 to 2026-08-12): 202.6 ms
Broken window (2026-08-12 to 2026-08-21 12:00): 488.7 ms
After fix (2026-08-21 12:00 to 2026-08-24): 212.5 ms
Excluded 1 rows (load-test window 2026-08-18 10:00-12:00)



## Findings






# Checkout latency: before and after the pool fix

