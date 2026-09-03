# A New Project Uses The Toolchain The Language Uses

Given a machine with more than one scaffolder installed, when File → New
Project is opened, then it offers each one — `.NET`, `Python (uv)`,
`Rust (cargo)` — and every entry in that picker is a tool this machine
actually has, measured by running it, never a list written down. A machine
with one scaffolder draws no picker and looks exactly as it did.

Choosing one asks the server again and rebuilds the form from **that
scaffolder's own help**: `uv init`'s flags are `uv init`'s, and none of
`dotnet new`'s are carried across. What is created is a **recipe commit**
exactly as before — the same temporary index, the same `Hick-Recipe`,
`Hick-Image` and `Hick-Output` trailers, the same `output_matches` check in
the history lens — with the recipe naming the tool that actually ran.

**A scaffolder never makes a repository of its own.** `uv init` and
`cargo new` both run `git init` inside the project unless told not to, so
both are always given `--vcs none`, and `--vcs` is not offered as a form
field. The flag is on the recorded recipe as well as on the run, because a
replay without it would produce a different tree and the history lens would
report drift that is not there.

A scaffolder that is not installed is a **typed refusal** carrying
`missing: "<program>"`, and the dialog draws a screen from that field rather
than from the sentence. A toolchain id nothing serves is a `400` that names
what this build does know.

## Why

The rule File → New Project was built on is that **the templates and the form
fields are the scaffolder's own**, read from the tool rather than written down
beside it — so a template from a package this product never heard of gets a
form too. That rule was right and its reach was one tool wide: the reading was
`dotnet new`'s table, so nothing else could be offered at all. A Python
project today starts with `uv` and a Rust one with `cargo`, and neither was
reachable from a dialog whose whole point was to meet a scaffolder on its own
terms.

Almost everything downstream was already general — `scaffold_commit` takes a
command string and an image string and knows nothing about .NET — so what
actually differs between scaffolders turned out to be four things, and they
are now a table: how many templates there are, where the output path goes
(`-o <path>` against a positional), how the name is spelled (`-n` against
`--name`), and which help parser reads it.

The nesting rule was not in the design; it was found by running the command
this code generates and looking at what appeared. `uv init services/orders`
left a `.git` inside `services/orders`, which would have been committed into
the repository that contains it — a repository inside a repository, which
`a-new-project-is-a-recipe-commit.md` says in as many words must never
happen. It is the sharpest argument in this change for running the thing
rather than reading about it.

`uv init` and `cargo new` have no template catalogue: they are one command
whose shape is chosen by flags. Offering them as a **catalogue of one** rather
than as a second kind of thing means the dialog needs no second idea of what a
scaffolder is, and the template list simply has one row.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/toolchain.rs` — `Toolchain`, `Shape` (the
  four-way table), `forced` (`--vcs none`), `catalog`, `detail`, `argv`,
  `command`, `commit_message`, `installed`;
  `crates/hickory-cli/src/clap_help.rs` — the parser for uv and cargo;
  `crates/hickory-cli/src/serve/scaffold.rs` — `chosen`,
  `scaffold_error_for`, and the toolchain threaded through `templates`,
  `options`, `preview`, `create` and `Watch`;
  `crates/hickory-cli/src/scaffold_commit.rs` — `commit_scaffold` takes the
  toolchain and asks it for the message; `apps/web/src/components/NewProjectDialog.tsx`
  — `ToolchainPicker`, `NoToolScreen`, and the `key={toolchain}` reset.
- Test coverage: `crates/hickory-cli/tests/serve_scaffold.rs` —
  `a_uv_project_is_a_recipe_commit_like_any_other` runs a **real** `uv init`
  through the API and asserts the commit holds `orders/pyproject.toml`, holds
  no `.git`, leaves the person's own edit untouched, and carries the
  previewed line verbatim as `Hick-Recipe` with `output_matches` true;
  `the_catalogue_names_every_scaffolder_this_machine_has` (which asserts
  every offered id is really installed),
  `uvs_own_flags_are_the_form_and_the_ones_hick_owns_are_not`,
  `a_scaffolder_nobody_serves_is_refused_by_name`;
  `toolchain::tests` (15, including
  `a_scaffolder_never_makes_a_repository_of_its_own`,
  `a_chosen_option_cannot_override_that`, and
  `dotnets_message_is_byte_for_byte_what_it_was`);
  `clap_help::tests` (12) and `tests/clap_help_live.rs` (3);
  `NewProjectDialog.test.tsx` — "choosing which scaffolder" (4).
- Caveat requiring LLM review: `cargo new` is in the table and has the same
  tests at the unit level, but no end-to-end test scaffolds and commits a
  Rust package the way the uv one does — the two paths are the same code, so
  this is an argument from shared implementation rather than from evidence.
  Go (`go mod init`) and Node (`npm init`) are not offered at all; neither is
  clap-based, and `npm init` is interactive, which this dialog has no shape
  for.
