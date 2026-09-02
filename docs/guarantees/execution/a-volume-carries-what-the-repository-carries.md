# A Volume Carries What The Repository Carries

Given a `<hick:volume input="…">` seeded from a directory, when a cell mounts
it, then the volume holds exactly the files the project's own `.gitignore`
would let git carry. Build output — `obj/`, `bin/`, `__pycache__`, a
`node_modules` a tool dropped — never reaches the cell. Hidden files are
**kept**, because `.editorconfig` and `.dockerignore` are inputs a build
reads, and a directory with no repository around it keeps everything, there
being nothing that says otherwise.

## Why

This is not about tidiness in the container. A cell's recording is keyed by
the digest of what is mounted
(`a-recording-is-keyed-by-the-cells-inputs.md`), so anything in a mounted
directory that churns makes every recording of that cell unfindable — and a
weave that cannot find a recording writes `[never run]` over the output of a
run that really happened, which is the exact failure
`a-weave-without-a-recording-keeps-the-artifact.md` was written about.

Build output churns by definition. Both halves of this were observed rather
than imagined: mounting `src/Warehouse.Web` after a `dotnet build` swept an
entire `obj/` into the key, and running a Python generator once left a
`__pycache__` beside its script, so the next weave of the document that had
just run it reported the cell as never run.

The filter is the repository's own, not a list of directory names this
product maintains. A hand-maintained list of "things that are build output"
is the shape of bug this codebase has already shipped four times, and every
project already states the answer in a file git reads.

## The other half

The same key makes a cell that mounts **its own unstable output** unstable,
which no ignore rule can fix: mounting `.` in a document that weaves markdown
beside itself puts that markdown — which carries the cell's transcript —
inside the key of the cell that produces it. That is a warning at run time
(`self_mounting_warnings`), not an error, and it says the fix in one line:
mount what the cell reads, not the folder it lives in.

Only **unstable** outputs count, and getting that boundary right is the whole
difficulty. A directory holding a `hick:file` the document assembles from
literal text is not a hazard — it is the central move of literate programming,
a cell running a script its own document wrote, and those bytes are identical
on every run. The first version of this warning did not make the distinction
and fired on three of the warehouse's five documents, which is how a warning
stops being read. Two kinds are unstable: the **weave target**, because
running the cell changes it, and a **`hick:file` fed by a cell**, because its
bytes are a run's output.

## The digest half

The rule above covers what a volume holds when it is **seeded** from a host
directory. It says nothing about the SEPARATE moment a cell's cache-key
**digest** is computed over whatever a volume currently holds — and until
2026-09-01, that computation used a hardcoded two-name allowlist
(`.hick-cache`, `.git`) with no `.gitignore` awareness at all. The gap showed
up specifically when a cell's own real, in-memory output (extracted after it
runs — not read from host disk, and not covered by `seed_from_directory`'s
filter at all) entered a volume another cell shares: a `dotnet build` cell's
`obj/` — full of non-deterministic timestamps and absolute paths in its
`*.json` files — destabilized the digest, and therefore the cache key, of
every cell sharing that mount, even though the same `.gitignore` was already
correctly keeping `obj/` off host disk.

The fix applies the exact same filter to the digest computation:
`mounted_inputs_digest` (`crates/hick-literate/src/lib.rs`) now checks each
candidate path against `volume_state::gitignored` — the same
git-authoritative function `seed_from_directory`'s sibling `hick ingest` path
uses (moved into `hick-literate` so both could share it rather than
reimplementing gitignore matching a third time) — before folding it into the
digest. Paths are checked as they appear WITHIN the volume, not reconstructed
to their true path relative to the project root; this is an approximation,
exact for the unanchored patterns (`bin/`, `obj/`, `__pycache__`) that are
the actual, observed failure mode, but not necessarily precise for a
gitignore rule anchored to the true repository root. A failure to check (no
`git`, no repository) folds into the digest rather than silently meaning
"nothing is ignored" — matching the existing unreadable-volume branch, and
`gitignored`'s own contract of never staying quiet about "could not check"
versus "checked, found nothing".


> **Amended 2026-09-02.** The warning about mounting a document's own unstable
> output (`self_mounting_warnings`) is gone, because the hazard it warned
> about is gone: a cell's input digest now leaves out the documents' own
> unstable products — every weave target and every `hick:file` a cell fills
> (`hick_literate::unstable_outputs`). A cell that mounts `.` no longer keys
> its recording on its own transcript, so the recording is findable on the
> next weave, which is what the warning was trying to get the author to
> arrange by hand. The `.gitignore` filter this guarantee is about is
> unchanged and still does the other half of the job.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified (digest half); the 2026-08-28 verification below still
  stands for the seed half
- Evidence: `crates/hick-literate/src/lib.rs`'s `mounted_inputs_digest`,
  consulting `volume_state::gitignored` inside its per-volume loop before
  folding each candidate path into the digest.
- Test coverage: `mounted_inputs_digest_excludes_gitignored_paths` and
  `mounted_inputs_digest_outside_a_repository_falls_back_to_the_fixed_exclusions`
  in `crates/hick-literate/src/lib.rs` — a real git repository with a
  `build/` `.gitignore` rule; the digest of two volume states differing only
  in a gitignored path's content is identical, while the digest of two
  states differing in a non-ignored path still differs (the sanity check
  that rules out "this test passes by digesting nothing"). Verified the fix
  is load-bearing by temporarily disabling the filter and confirming the
  first test fails with two different digests, then restoring it. Outside a
  repository, nothing beyond `.hick-cache`/`.git` is filtered, matching the
  seed-time fallback.
- Caveat requiring LLM review: an EARLIER attempt at an end-to-end
  integration test (two cells sharing a volume across separate `hick run`
  invocations) gave a false failure unrelated to this fix — it tripped
  `self_mounting_warnings`' own hazard (`input="." output="."` sweeping up
  the document's own weave target) and separately ran into the DAG's
  upstream-key propagation, where a cell that depends on another correctly
  re-keys when its predecessor's key changes, for reasons that have nothing
  to do with gitignore filtering. Both are real, separate mechanisms; the
  unit-level test above isolates the one property this fix actually changed.
  Nobody has yet reproduced the ORIGINAL fork's exact `dotnet build`
  scenario as an automated test — that finding remains dogfooding evidence
  (`.../scratchpad/tutorial-hick/friction-notes.md`, point 6), not CI
  coverage.
