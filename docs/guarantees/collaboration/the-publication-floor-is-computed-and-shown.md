# The Publication Floor Is Computed On Every Read And Shown

Given a git repository open in the app, when the commit graph is read, then
the **publication floor** — `merge-base(HEAD, <published ref>)` — is computed
and reported alongside the commits, and every commit above it is marked a
draft with a sentence saying why.

The floor is what makes emission one-way: below it a commit is a **record**,
because someone else may be holding it, and nothing may re-produce one. Above
it is the **frontier**, which is derived — edit the document, re-emit, and
those commits are replaced rather than patched. Nothing here emits anything;
this is the fact, surfaced, and a person cannot respect a boundary they
cannot see.

Corollaries that are part of the guarantee:

- **The published ref is the branch's own upstream first.** `@{upstream}` is
  the ref *this* branch pushes to; falling straight to `origin/master` would
  call already-published commits on a pushed feature branch drafts.
  `origin/HEAD` and then `origin/master` / `origin/main` are the fallbacks,
  and which one was used is reported.
- **No published ref means everything is a draft**, said in those words: a
  repository with no remote-tracking branch has published nothing, and nobody
  else can be holding any of it.
- **A branch sharing no history with the published ref** is reported as that,
  not as "no floor".
- **Merging moves the floor**, and yesterday's rewritable commits become
  permanent — with nothing about any document changing at that moment. That is
  why the floor is computed on every read and stored nowhere: a recorded floor
  would be a claim the next fetch falsifies.
- **The mutable/immutable boundary lives in the commit graph, never in the
  document text.** No region of a document is ever frozen.
- **A folder that is not a repository has no floor and that is not an error.**
- **The wording is publication, never transport.** "Nobody else can be
  holding them" is the claim; "unpushed" would be a different and weaker one.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/floor.rs` — `compute`, `published_ref` (upstream →
    `origin/HEAD` → named), `Floor::unpublished`, `Floor::is_draft`,
    `draft_set`, and the summaries.
  - `crates/hickory-cli/src/serve/git.rs` — the floor computed in the same
    blocking task as the log, `Commit::draft`, and `floor` on the response.
  - `crates/hickory-cli/src/serve/history.rs` — `GET /api/git/floor`.
  - `apps/web/src/views/GitPane.tsx` — the floor bar and the per-row `draft`
    badge; `apps/web/src/api/types.ts` — `PublicationFloor`.
  - Tests: `crates/hickory-cli/tests/replay_and_floor.rs`
    (`with_nothing_published_every_commit_is_a_draft`,
    `the_floor_is_the_merge_base_with_the_published_ref`,
    `merging_moves_the_floor_and_yesterdays_drafts_become_records`,
    `a_folder_that_is_not_a_repository_has_no_floor_and_that_is_not_an_error`);
    `apps/web/src/views/GitPane.test.tsx` (the draft badge and the no-floor
    case).
- Caveat requiring LLM review: the floor is computed from a published ref,
  which assumes the only way a commit escapes is a push. A peer that attached
  over a fleet channel and pulled would hold a copy the floor does not know
  about. That is an open edge of `expression-and-log.md` and the fleet channel
  is not built, so nothing can exercise it yet.
