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

---

Last LLM verification:
- Date: 2026-08-28
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-literate/src/volume_state.rs::seed_from_directory`
  walks with `ignore::WalkBuilder` (`hidden(false)`, `git_ignore(true)`,
  `git_exclude(true)`, `require_git(false)`, `parents(true)`) instead of
  `tar::Builder::append_dir_all`, which took the directory whole.
  Reproduced end to end in the `warehouse` project: with `__pycache__` on
  disk beside a generator, `hick test .` reported three documents drifted and
  their cells never run; with the filter in place the same folder is `ok`
  across all seven.
- Test coverage:
  `volume_state::tests::what_the_repository_ignores_never_reaches_the_volume`
  (ignored build output absent, hidden file present, ordinary source
  present); `volume_state::tests::seed_from_directory` still covers the
  no-`.gitignore` case, where everything is carried.
- Caveat requiring LLM review: `require_git(false)` means the rules apply
  even outside a git repository, which is what makes a `.gitignore` in a
  bare directory work — but it also means a project that deliberately
  gitignores a generated file it then mounts as input will find it missing.
  Nothing warns about that case yet.
