# The History Lens Reads The Repository As A Story

Given a folder under version control, when its history is opened as a story
(the Git pane's *Read as a story*, or *History as a story* from the command
bar), then the commits are drawn **oldest first** as cards: an ordinary
commit as prose (its subject, its body, its files behind one click), a
commit whose trailers carry a recipe (`Hick-Recipe`, `Hick-Image`,
`Hick-Output`) as a **cell** whose command is shown and whose output is the
commit's own diff, and the working tree as the last card. The past folds by
default and the view opens at the tail. The publication floor is drawn as a
line between the records and the drafts. A recipe card, once expanded, says
which of its files a later commit changed and in which commit — or that
none has.

The view says on its face that it is a lens: it exists on disk nowhere, it
cannot be saved, and every card is read-only. A folder that is not a
repository says so rather than failing.

A recipe is a **declared** claim, in the commit's own words. Nothing has
replayed it, so the card is drawn *unrecorded* and says "no evidence of
drift". The word "reproducible" does not appear on a recipe card.

This is step 2 of `docs/specs/freeform/lenses.md`. It answers the complaint
that started that spec: a scaffold whose output vanished the moment one of
its files was edited. Here the output is the commit's tree, which is
permanent, and *edited since* is a separate fact on the same card.

## Boundary

Read-only. No reword, reorder, drop, replay or commit happens from this
view yet; those are later steps and are gated by the floor drawn here.
`edited_since` names the **first** later commit to touch each file, not
every one. A merge commit is drawn as an ordinary card with its combined
diff; the diverged surface for merges is a later step.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/git.rs` — `Recipe`, `recipe_of`
  (trailers in the last paragraph only, per `git interpret-trailers`),
  `Commit::recipe`, the `commit` handler on `GET /api/git/commit?sha=` and
  `edited_since` (one `git log --reverse --name-only <sha>..HEAD -- files`);
  `crates/hickory-cli/src/serve/mod.rs` routes `GET`/`POST /git/commit`
  together. In the app: `apps/web/src/views/HistoryLens.tsx` (`storyOrder`,
  `floorIndex`, `OPEN_TAIL`, `CommitCard`, `CommitDetail`, `TailCard`),
  `apps/web/src/views/workspaceState.ts` (`STORY_TAB`, `openStoryTab`),
  `apps/web/src/views/WorkspaceView.tsx` (the tab body and the command-bar
  entry), `apps/web/src/views/GitPane.tsx` (`onOpenStory`).
- Test coverage: `crates/hickory-cli/src/serve/git.rs::a_recipe_is_read_from_the_last_paragraphs_trailers`;
  `crates/hickory-cli/tests/git_ops.rs::a_commit_reads_as_a_card_with_its_recipe_and_what_was_edited_since`
  (a real repository over HTTP: the recipe on the log, the diff and
  `edited_since` on the commit, a non-hex id refused with 400, an unknown
  one with 404); `apps/web/src/views/HistoryLens.test.tsx` (order, the
  floor line, the recipe cell and its wording, the diff and edited-since on
  expand, folding, the not-a-repository answer).
- Caveats: the lens is exercised in jsdom, not in a real browser. Nothing
  yet writes a recipe commit (step 3), so the only recipe commits are ones
  written by hand or by tests.
