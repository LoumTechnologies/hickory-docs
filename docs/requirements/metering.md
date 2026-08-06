
# Metering — requirements

Written against the decisions of 6 Aug and the billing domain model. The
term and the decisions below are imported, not restated: if either changes
upstream, this document changes with it and `hickory check` fails until the
generated artifact is regenerated.




## Vocabulary


**Execution minute** — wall-clock time a document's cells spend inside an
executor, measured from container start to the last cell's exit. Not CPU
time, and not billed when a run is served from cache without starting a
container.


## Decisions being implemented


The free plan (**Open**) includes 300 execution minutes per month and
1 private project. Public projects are unlimited.



Exceeding the monthly execution-minute allowance is a **hard stop**: the
run is refused before any container starts, with the plan's limit and the
upgrade path named in the error.


## Requirements






**R1.** A run that would exceed the caller's remaining monthly allowance is
refused with HTTP 403 before any container is started. The error names the
plan, its allowance, and the upgrade path.



**R2.** Allowances reset at 00:00 UTC on the first of each calendar month.


## Allowance table

These are the numbers R1 enforces, written once. Because a paste copies
bytes verbatim, the canonical form has to be the *machine* form — the prose
below renders the same bytes rather than a second, prettier copy that could
drift from it.



```json
    "open": 300,
    "pro": 2000,
    "team": 10000,
    "business": 30000

```


### `plan-limits.json`

```json
{
  "_generated_by": "docs/requirements/metering.hick — edit the requirement, not this file",
  "exec_minutes_month": {
    "open": 300,
    "pro": 2000,
    "team": 10000,
    "business": 30000

  }
}
```

