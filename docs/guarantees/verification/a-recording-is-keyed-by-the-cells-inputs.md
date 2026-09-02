# A Recording Is Keyed By What The Cell Read, Not Only By What It Says

Given a document whose exec cell reads a file the document itself assembles,
when that file changes but the cell's command text does not, then the cell is
**not** answered from its recording: it re-executes, and the document reports
what its current code produces.

The failure this exists to prevent is silent. A document assembles
`analysis.py`, a cell runs `python3 analysis.py`, an author edits the analysis,
and the command text is byte-identical before and after. A cache keyed on the
command alone serves the old recording, and the document publishes numbers
produced by code it no longer contains — with no error, no drift report, and
nothing about the output that looks wrong.

Two terms in the key carry this:

1. **The cell's mounted inputs.** Every volume mounted into the cell is
   digested — path and bytes, sorted, length-prefixed — and folded into the
   key. A `<hick:file>` product is seeded into an input volume before the run,
   so it becomes an input in exactly the way the document's author already
   thinks it is one.
2. **The cell's predecessors' keys.** Invalidation is transitive without
   hashing anything twice: if an upstream cell's inputs changed, its key
   changed, so every cell downstream of it keys differently too. Container
   state, volume flow, copy/paste, forks, and agent barriers are all carried
   by this one term, because the flow DAG already models each of them as an
   edge.

The key is computed **once per cell, before the cell runs**, and reused for
both the lookup and the store. Sampling it after execution would key the
recording on the volume contents the cell just produced rather than the ones it
read, and a lookup and a store that computed their keys separately could
disagree — recording every cell under a key nothing ever looks up.

The weave path computes the same key, seeding the same input volumes even
though it never executes. Otherwise a woven document could not find any of the
recordings a run had just written, and every cell would report never-run
immediately after a successful `hick run`.

## Boundary

**`.hick-cache/` and `.git/` are excluded from the digest.** An input volume
declared `input="."` seeds from the project directory, which contains the
recordings themselves; without the exclusion each run would write a recording,
changing the volume, changing the key, writing a new recording — a cache that
never hits and grows a file per run.

**A file the cell reads from outside any declared volume is not in the key.**
The cache sees declared inputs, which is the same boundary the flow DAG has:
undeclared reads are invisible to both. This is part of why `--cache` is opt-in.

**A cold start records one entry it will never reuse.** The first run happens
before the document's assembled file exists on disk, so it reads a different
input set than every run after it. One wasted entry per cold start, and no
growth per run after that.

**A changed input takes one extra run to settle.** The assembled file is
written to disk by the run that produces it, so the first run after a document
change seeds the volume from the previous version on disk. The cell still
executes against the correct current code — staging happens before execution —
but the *key* stabilises one run later.


> **Amended 2026-09-02.** A third term is *excluded*: the documents' own
> unstable products. The weave target carries the cell's transcript and a
> `hick:file` a cell fills carries a run's output, so a cell that mounted the
> directory holding them keyed its recording on bytes its own run changes.
> Every run changed the key, no recording was ever found again, and the next
> weave wrote `[never run]` over recorded output. Those paths are now skipped
> in `mounted_inputs_digest` (`unstable_outputs`, matched exactly and by
> tail for a volume seeded from a subdirectory). A `hick:file` assembled from
> literal text is still in the key, which is this guarantee's whole point;
> `a_documents_own_unstable_products_are_not_in_the_key` covers both halves.
> Proven on `examples/grand-tour.hick`: one `hick run`, then two weaves in a
> row found the recording, and `git status` stayed clean.

---

Last LLM verification:
- Date: 2026-08-12
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-literate/src/cache.rs::exec_cache_key` takes
  `input_digest` and `upstream_keys` alongside image, capabilities, command,
  and secret names; `inputs_digest` sorts by path and length-prefixes both path
  and body so a shifted boundary cannot collide.
  `crates/hick-literate/src/lib.rs::mounted_inputs_digest` digests the unpacked
  entries of each mounted volume (not the tar, which carries mtimes and would
  give stable inputs a new key every run), skipping `is_run_artifact` paths.
  `upstream_keys` reads `FlowDag::predecessors` in sorted id order and
  substitutes a placeholder for an unkeyed predecessor so "present but unkeyed"
  never digests as "absent". The live pipeline computes `exec_key` once before
  the cell runs, inserts it into `keys_by_exec`, and reuses it at the store
  site; `run_pipeline_weave` seeds input volumes from `CacheConfig::project_dir`
  and computes the identical key.
- Test coverage: `crates/hickory-cli/tests/cache_inputs.rs` —
  `a_changed_input_file_re_executes_the_cell_that_reads_it` drives the real
  binary, changes only the data inside an assembled file, and asserts both that
  the recording was not used and that the weave reports the new result;
  `an_unchanged_document_settles_back_onto_its_recording` asserts repeat hits
  and that the recording set stops growing. Unit tests in `cache.rs` cover each
  key term in isolation (`cache_key_changes_with_inputs`,
  `cache_key_changes_with_upstream`) and the digest's ordering, collision, and
  empty-vs-present properties.
- Caveat requiring review: the `.git/` exclusion is reasoned, not tested — no
  fixture commits mid-run to prove a commit would otherwise invalidate a cell.
