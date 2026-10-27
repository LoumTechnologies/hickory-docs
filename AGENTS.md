Post-commit checks are very thorough. Do not watch the logs; it uses too many tokens. Tell the user how to monitor / check status on such things.

Only run focused behavior and safety tests; leave broad regression tests and all regression-check status monitoring to the user.

Maintain code and tests in the owning entries under `.hickory-source/`, starting
from `project.md`. `hickory-dogfood/behavior-map.json` maps output paths to source
pages. Regenerate with `python3 hickory-dogfood/tools/behavior.py generate`, then
run `check` and `audit`. See `docs/developers/literate-workflow.md` for bootstrap.
Commit Markdown changes and their generated outputs together. Do not rerun the
one-time capture/reconciliation scripts or edit generated files alone.

Current requirements are the contract pages linked from `project.md`.
`docs/guarantees/` and `docs/specs/freeform/` retain captured documents as history;
the canonical contract pages distinguish retained, reconciled and historical text.

Always use the audience-first-literate-docs skill when modifying code or tests,
and for all maintained source changes. Interleave real
explanations, implementation and tests at reasoning boundaries. Current code uses
copy fragments and explicit file/paste assembly; raw entries preserve literal
assets and embedded document syntax. Shared code lives under named mechanisms.
Use `behavior.py locate <output-path>` to find its construction and every fragment.
A narrative owner is not proof of adequate explanation: inspect the explicit gaps
in `hickory-dogfood/narrative-map.json` and do not invent coverage or guarantees.
