
# Planned Messages, With Receipts

A message you are ABOUT to send is a claim you will have to stand behind.
This document drafts messages as generated files assembled from named fact
blocks — so when someone asks "where did that number come from?", lineage
answers byte-for-byte: `hick lineage examples/planned-messages.hick
--output messages/2026-08-17-eng-standup.txt` points every sentence at the
exact block it came from, and the app draws the same answer as ribbons.

Note the namespace: this document uses the prefix `slack:` instead of
`hick:`. Any prefix bound to the hickory namespace works — a document about
Slack messages gets to read like one. The vocabulary is unchanged; the
prefix is yours.

## The facts, each from a named source

Each block records where the information came from in its own text, so the
receipt travels with the fact.

## The message to paste into #eng-standup


### `messages/2026-08-17-eng-standup.txt`

```
Standup, 2026-08-17:

p95 checkout latency dropped from 480ms to 210ms after the connection-pool fix (Grafana dashboard "checkout-slo", 2026-08-16 snapshot).

The release-pipeline incident (2026-08-14, three independent failures) is closed; fixes verified in CI run #412.

Remaining risk this week: the include-lineage migration touches every woven document; rollback is a git revert.

Questions welcome — every line above has a source; ask me and I will point at it.
```


## The shorter one for #leadership

Same facts, different audience — and the SAME provenance, because both
messages paste the same blocks. Editing a fact block updates every planned
message that cites it on the next weave.


### `messages/2026-08-17-leadership.txt`

```
Checkout is fast again and the release pipeline is green.

p95 checkout latency dropped from 480ms to 210ms after the connection-pool fix (Grafana dashboard "checkout-slo", 2026-08-16 snapshot).
```

