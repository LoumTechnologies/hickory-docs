
# Billing — domain model

The vocabulary this product bills in. Terms are tagged `.glossary`;
decisions arrive from `docs/decisions/` and are never restated here, so
there is exactly one place each of them is written down.

## Terms





## Decisions this model rests on



Billing meters **execution minutes**, not seats. Seats are a soft limit
that gates collaboration features, never the primary meter.



The free plan (**Open**) includes 300 execution minutes per month and
1 private project. Public projects are unlimited.



Exceeding the monthly execution-minute allowance is a **hard stop**: the
run is refused before any container starts, with the plan's limit and the
upgrade path named in the error.


## Why the distinction matters

"Allowance" and "quota" are the pair people confuse. A quota implies the
work waits; ours refuses. That is a product promise, not an implementation
detail — see the hard-stop decision above.
