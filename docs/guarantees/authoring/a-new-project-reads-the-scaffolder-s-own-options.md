# New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains

Given a machine with a .NET SDK, when File → New Project is opened, then the
templates offered are exactly what `dotnet new list` reports on that machine,
and the fields shown for a chosen template are exactly the options `dotnet new
<template> --help` names — including templates from packages this product has
never heard of.

The alternative was a curated list of templates and flags, and it fails on the
day the SDK ships a new one: a dialog that knows about `webapi` and not
`mcpserver` is a dialog that has to be updated to keep telling the truth about
somebody else's tool.

Parts of the guarantee:

- **The option names come from `--help` and nowhere else.** The template
  engine keeps a JSON cache at
  `~/.templateengine/dotnetcli/<sdk>/templatecache.json` with every symbol,
  its type, its choices and its default. It is not usable for this: the names
  in it are the template's **symbol** names, and the mapping to CLI options
  lives in a `dotnetcli.host.json` inside a `.nupkg`. `dotnet new webapi
  --ExcludeLaunchSettings true` is rejected with "not a valid option", so the
  help text is the only source that names what a person may actually pass.
- **A wrapped value comes back exact.** Help is hard-wrapped to a width no
  environment variable moves. A line broken **at a space** keeps that space; a
  line broken **mid-token** does not. Concatenating is therefore the whole
  rule, and it is what returns
  `https://qualified.domain.name.b2clogin.` + `com/` as one URL rather than as
  one with a space in the middle.
- **The footer ends the section.** "To see help for other template languages
  (VB), use --language option:" is followed by an indented example command,
  which sits in the same column as a continued option name. Reading it as one
  turned `--nullable` into `--language` — a wrong flag in a generated command,
  not a cosmetic slip.
- **A template nobody has is refused by name.** `dotnet new nope --help` exits
  **zero** and prints "No templates or subcommands found matching" on stdout,
  so the exit code cannot be trusted; the presence of a `Usage:` line is what
  distinguishes a help page. Without that check the sentence came back as the
  template's title and the dialog drew an empty form for a template that does
  not exist.
- **`Enabled if:` is shown, never evaluated.** The template engine's condition
  (`UseMSTestSdk && (TestRunner == Microsoft.Testing.Platform)`) is displayed
  verbatim beside the field. Evaluating it would mean reimplementing another
  program's expression language inside a dialog, and a form that greys out the
  wrong field is worse than one that says what the condition is.
- **`dotnet` is asked in English.** `DOTNET_CLI_UI_LANGUAGE=en` is set on every
  invocation, because this parser reads the words `Type:` and `Default:`. The
  template listing's columns are read off the **rule line** rather than the
  header, so the layout rather than the language is what the table parse
  depends on.
- **Nothing phones home to ask.** `DOTNET_NOLOGO`, `DOTNET_CLI_TELEMETRY_OPTOUT`
  and `DOTNET_SKIP_FIRST_TIME_EXPERIENCE` are set: nothing in this product
  talks to anyone, and that includes the tools it asks questions of.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/scaffold.rs` — `catalog`, `template_detail` (and
    its `Usage:` check), `parse_template_list` with `columns`/`cell`,
    `parse_template_help` with its `Sink` and the indent rules, `dotnet()`'s
    pinned environment.
  - `crates/hickory-cli/src/serve/scaffold.rs` — `templates`, `options`, both
    on `spawn_blocking`.
  - `apps/web/src/components/NewProjectDialog.tsx` — the list, the per-option
    fields, and re-reading the options when the language changes.
  - Tests: `crates/hickory-cli/tests/scaffold_templates.rs`, against real help
    output checked in under `tests/fixtures/dotnet/` (captured from SDK
    10.0.111, 2026-09-01) — `the_listing_becomes_templates`,
    `a_simple_template_becomes_fields`, `a_wrapped_default_comes_back_whole`,
    `choices_keep_their_own_descriptions`,
    `a_condition_is_carried_not_evaluated`, `the_footer_ends_the_section`.
    `crates/hickory-cli/tests/serve_scaffold.rs` —
    `the_catalogue_answers_or_says_there_is_no_sdk` drives the live SDK when
    there is one.
- Caveat requiring LLM review: the fixtures are one SDK on one machine. A
  sweep over all 46 templates that SDK offers produced no malformed option,
  but the parse is bound to a layout Microsoft may change. A future SDK that
  laid help out differently would show wrong fields rather than fail loudly;
  the preview pane is what makes that visible before anything is written.
