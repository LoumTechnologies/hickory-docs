# Stages write forward

A pipeline is a chain of documents connected by `hick:upstream`. Each link in
that chain is a **stage**: one document, plus the files it generates.

## Every stage can do everything

A stage is not a phase in a fixed lifecycle, and the capabilities are not
rationed by position. Any stage may declare containers, run cells, pin
expectations, generate files, hold diagrams, and fail the build.

This matters because the obvious mental model is wrong. It is tempting to
think requirements are prose, implementation is code, and only the last stage
executes — so execution drifts toward the end and the early documents become
unverifiable narration. But a requirement is exactly the kind of claim worth
executing. "The website isn't working yet" is a statement someone wrote down;
a cell that checks it is a statement that tells you the day it stops being
true. A meeting note recording "the vendor's API returns ISO dates" can hold a
cell that fetches one. Those are the claims most likely to rot silently,
because nothing downstream restates them.

So: a stage that only describes is a choice its author made, never a limit the
tool imposed.

## Except backwards

The one rule is directional. **A stage may read anything upstream and write
nothing there.**

Reading upstream is the point of the chain: a fragment is declared once and
pasted wherever it is needed, so a decision has exactly one home. Writing
upstream would destroy that. If implementation could rewrite a requirement,
the requirement would no longer be the record of what was asked for — it would
be a record of what was eventually built, which is the failure every design
document already suffers from and the reason nobody trusts them.

Concretely, for any stage:

- Its generated files land in its own space, never over a file an earlier
  stage owns.
- Its cells may read upstream files; a cell that writes to one is a defect,
  not a feature. Two stages producing one file means the second silently wins,
  and which is second depends on execution order.
- A disagreement with an upstream fact is fixed **upstream, by a person or an
  agent editing that document** — deliberately, in the place the fact lives.
  That is a different act from a downstream stage overwriting it as a side
  effect of running, and it is the act the agent tools already support
  (`docs/guarantees/agent/the-editable-set-is-the-pipeline-closure.md`).

The distinction is authorship, not permission: an agent may edit any document
in the closure because a person asked it to. A *cell* may not, because nobody
asked — it just ran.

## What this implies for the last stage

The terminal stage of a chain is usually generated source code, and it is
different in kind from the ones before it: everything upstream *describes*,
while the last stage *is the thing described*. That is the boundary, not
"where the executable cells are" — cells are everywhere.

It is also the only stage whose coverage is partial. Every byte of a `.hick`
document belongs to the chain; a real source tree contains files hick
generated and files nobody generated, and the ones with no upstream are
exactly the parts of the system this pipeline cannot account for. A view of
that stage should show the whole tree and mark which is which, because "what
is not covered" is the most useful thing it can tell you.
