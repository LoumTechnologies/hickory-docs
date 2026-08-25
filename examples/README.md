# Examples

Local execution runs cells against your **host** toolchain — `image=` is
recorded and ignored (see the note in the top-level README). So an example
needs whatever its cells invoke to be installed locally.

| Example | Needs | What it shows |
|---|---|---|
| `text-tools-tour.hick` | nothing beyond a POSIX shell (`sort`, `awk`, `wc`, `tr`) | The core loop: cells, pinned expectations, weaving |
| `bootstrap-ci.hick` | `python3` (standard library only) | A small statistical paper that computes its own figure and verifies its own numbers |
| `grand-tour.hick` | `python3` with `polars`, the `duckdb` CLI; optionally `Rscript` with `ggplot2` | Literate weaving with lineage, polyglot cells, generated interactive artifacts, feature gates |
| `scaffolded-service.hick` | nothing beyond a POSIX shell | Owning what a generator wrote: `hick ingest --from`, the gitignore as the filter, and editing bytes you did not type |
| `architecture-that-draws-itself.hick` | `hick` itself on the PATH | A diagram that fails when it lies: `hick diagram` deduces the topology (AI-free), an expect pins it, and a `renderer="graph"` diagram derives its picture from the pinned fragment |

Start here — it works everywhere:

```sh
hick run examples/text-tools-tour.hick
hick test examples/text-tools-tour.hick
```

`scaffolded-service.hick` is the one to read if you have ever run
`dotnet new` or `npm create` and then had to change four lines in what it
wrote. Its `service/` tree is committed output like everything else here, and
the `<hick:ingested>` block in it was written by:

```sh
hick ingest --from '#scaffold' examples/scaffolded-service.hick
```

`hick test examples/` runs all of them and needs the full set above. The R
chapter of the grand tour is behind a feature flag and stays off unless you
ask for it:

```sh
hick run examples/grand-tour.hick --features with-r
```

Every `.md` and generated artifact beside these documents is committed output.
`hick test` re-executes and fails if any of it has drifted, which is how
these examples stay honest.

## `ai-transform.hick`

A passage WRITTEN by a model, VERIFIED like everything else: `hick:transform`
records what it read (`select=`), the instruction, and a fingerprint of both
(`from=`). `hick refresh` (the only command that calls a model) writes the
passage; `hick test` checks the fingerprint offline and free — edit a fact
and the summary is flagged stale until refreshed. Needs a provider key only
to refresh, never to verify.

## `planned-messages.hick`

Messages you are about to send, drafted as generated files assembled from
named fact blocks — so `hick lineage` (and the app's ribbons) answer "where
did that number come from?" byte-for-byte. Also shows that a document may
bind ANY prefix to the hickory namespace: this one reads as `<slack:copy>`,
`<slack:file>`.

## `receipts/`

A Slack message with receipts, built twice (by Claude Code and by the
built-in agent): meeting → analysis → fix → message, each sentence a fragment
with a model-written check under it, the Slack text a generated file whose
lineage runs back to the meeting turn. See `receipts/README.md` and
`docs/specs/freeform/receipts-for-a-message.md`. Needs `python3` (standard
library) to run; verifies with no key.
