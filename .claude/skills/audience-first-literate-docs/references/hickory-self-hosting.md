# Maintaining Hickory Docs itself

Apply this reference only in the Hickory Docs repository, whose entry is
`project.md` and whose maintained source is under `.hickory-source/`.
Read its `AGENTS.md` and `docs/developers/literate-workflow.md` before editing.
Current requirements are the contract pages linked from the entry document;
captured documents under `docs/guarantees/` and `docs/specs/freeform/` are history.

Teach how to build Hickory Docs with Hickory Docs. Use
`python3 hickory-dogfood/tools/behavior.py locate <output-path>` to find the
construction and every fragment. `hickory-dogfood/behavior-map.json` maps outputs
to source pages; `hickory-dogfood/narrative-map.json` records explanation gaps.
Inspect those gaps rather than inferring adequate explanation from ownership.
Do not rerun historical capture or reconciliation scripts as an editing workflow.

After editing source, regenerate with
`python3 hickory-dogfood/tools/behavior.py generate`, then run `check` and `audit`
through the same tool. Run only relevant focused behavior and safety tests;
leave broad regression checks and their status monitoring to the user. Commit
maintained Markdown and generated outputs together when a commit is requested.

When publishing Hickory's own literate source, let the rendered document carry the
reading experience. Use only a quiet file sidebar and breadcrumbs where needed;
avoid a separate welcome banner, document picker, related-links panel, publication
statistics or other presentation wrapped around the tutorial. Navigation and
scaffold folds must work without changing the source bytes.

