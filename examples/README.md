# Examples

Local execution runs cells against your **host** toolchain — `image=` is
recorded and ignored (see the note in the top-level README). So an example
needs whatever its cells invoke to be installed locally.

| Example | Needs | What it shows |
|---|---|---|
| `text-tools-tour.hick` | nothing beyond a POSIX shell (`sort`, `awk`) | The core loop: cells, pinned expectations, weaving |
| `bootstrap-ci.hick` | `python3` (standard library only) | A small statistical paper that computes its own figure and verifies its own numbers |
| `grand-tour.hick` | `python3` with `polars`, the `duckdb` CLI; optionally `Rscript` with `ggplot2` | Literate weaving with lineage, polyglot cells, generated interactive artifacts, feature gates |

Start here — it works everywhere:

```sh
hickory run examples/text-tools-tour.hick
hickory check examples/text-tools-tour.hick
```

`hickory check examples/` runs all three and needs the full set above. The R
chapter of the grand tour is behind a feature flag and stays off unless you
ask for it:

```sh
hickory run examples/grand-tour.hick --features with-r
```

Every `.md` and generated artifact beside these documents is committed output.
`hickory check` re-executes and fails if any of it has drifted, which is how
these examples stay honest.
