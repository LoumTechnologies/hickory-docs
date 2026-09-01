# New Project Writes A Document Holding The Command It Ran, And The Bytes That Command Produced

Given a folder open in the app and a `dotnet new` template chosen in File →
New Project, when the project is created, then a `.hick` document is written
holding the exact `dotnet new` command as an `<hick:exec>` cell whose command
is a `<hick:copy id="scaffold">` child — and, when the scaffold is run, the
generator's output is ingested into that same cell as `<hick:file>` bytes the
document owns.

The document is `owning-what-a-scaffolder-wrote.md`'s shape and nothing new:

```
<hick:container name="sdk" image="mcr.microsoft.com/dotnet/sdk:10.0" />
<hick:volume name="project" output="greeter" />

<hick:exec container="sdk" mount="project:out">
<hick:copy id="scaffold">
dotnet new console -o out -n Greeter --language 'C#' --no-restore
</hick:copy>
<hick:ingested from="#scaffold" sha256="…" at="…" files="2" skipped="0">
<hick:file path="greeter/Program.cs">…</hick:file>
…
</hick:ingested>
</hick:exec>
```

This is a **door onto the existing verb**, not a second mechanism. `POST
/api/scaffold` writes the document and then calls the same
`ingest_from_exec` that `hick ingest --from '#scaffold'` calls, with the same
executor, the same gitignore filter, the same non-UTF-8 refusal, and the same
weave-verified write. Everything
`docs/guarantees/authoring/ingest-owns-what-a-scaffolder-wrote.md` promises is
promised here because it is the same code.

Parts of the guarantee:

- **`-o out` is the mount point, not the destination.** The command writes
  into the cell's mount; where the tree lands in the repository is the
  volume's `output=`. That separation is what lets the identical command run
  on this machine or in a container.
- **The document is bare.** No `<hick:doc>` wrapper, because nothing in it
  rebinds the namespace prefix — `docs/specs/freeform/bare-documents.md`.
- **The command carries the decisions and nothing else.** A field left at the
  template's default is not written. `dotnet new console` and `dotnet new
  console --framework net10.0 --langVersion latest --no-restore false`
  scaffold the same project, and only the first is a command somebody can read
  a year later and see what was chosen. A bool turned **on** is the bare flag;
  a bool turned **off** is spelled `--flag false`, because the bare flag means
  true everywhere in `dotnet new`.
- **`--no-restore` starts on, unlike `dotnet`'s own default.** A restore fills
  `obj/` inside the volume the ingest reads. The project's `.gitignore` is the
  filter that normally takes it back out, but a notes folder that has never
  held a .NET project has no reason to ignore `obj/` — and then five NuGet
  caches land in the document. Not generating them is the answer that does not
  depend on another file being right. It is a checkbox, and the dialog says
  why it differs.
- **The preview is the bytes.** The dialog shows the document it is about to
  write, rendered by the same function that writes it, reached over `POST
  /api/scaffold/preview`. There is no second renderer in TypeScript that could
  show a document different from the one created.
- **The files are on disk when the dialog closes.** An ingested volume is
  deliberately no longer flushed as a pipeline output — the document owns
  those bytes, and flushing a fresh run over them would overwrite the edits
  that ownership exists to protect — so after an ingest a **weave** is the
  only thing that puts the tree on disk. New Project does it. Without it the
  dialog finished with a document full of a project and a folder with no
  project in it, and whether the files appeared came down to whether a file
  watcher happened to be running.
- **A name that needs quoting gets it.** Arguments are quoted for the POSIX
  shell the cell runs under, so a project name with a space produces a cell
  that runs rather than one that fails on a word.

What this does NOT claim: nothing here builds or launches the project. A
compiled language needs a build between the weave and the launch, which is
`docs/specs/freeform/launching-what-a-document-builds.md` and is a separate
mechanism. New Project scaffolds and ingests; running the program is the
document's own business afterwards.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/scaffold.rs` — `scaffold_document`,
    `dotnet_new_command`, `shell_quote`, `SCAFFOLD_CELL`, `sdk_image`,
    `suggested_path` / `suggested_output`.
  - `crates/hickory-cli/src/serve/scaffold.rs` — `create` (write, then
    `crate::ingest_exec::ingest_from_exec` with the session's executor),
    `preview`, and the shared path rules via `api::new_doc_target`.
  - `apps/web/src/lib/scaffold.ts` — `chosenOptions` (only what changed),
    `initialValues` (`--no-restore` on), `slug`, `problems`.
  - `apps/web/src/components/NewProjectDialog.tsx` — the form, and the
    preview pane fed by `api.scaffoldPreview`.
  - `apps/desktop/src-tauri/src/lib.rs` — the `new-project` File item and its
    forwarding; `apps/web/src/lib/menuBridge.ts` — the action;
    `apps/web/src/App.tsx` — where the dialog is mounted.
  - Tests: `crates/hickory-cli/tests/serve_scaffold.rs`
    (`a_new_project_owns_what_dotnet_wrote` runs a real `dotnet new console`,
    asserts `greeter/Program.cs` is in the `.hick` file with the run's
    `sha256=`, that the file is on disk when the route answers, and that
    deleting the tree and weaving reproduces it;
    `the_preview_is_the_bytes_that_get_written` asserts the preview and the
    written file are byte-identical);
    `crates/hickory-cli/tests/scaffold_templates.rs`
    (`the_command_is_the_one_a_person_would_type`,
    `a_name_that_needs_quoting_gets_it`,
    `the_document_parses_and_holds_the_command`);
    `apps/web/src/lib/scaffold.test.ts` (what reaches the command line);
    `apps/web/src/components/NewProjectDialog.test.tsx`.
- Caveat requiring LLM review: the "only what changed" rule makes the command
  depend on this build's reading of each option's default. A default parsed
  wrongly would silently omit a flag the person set. The parse is pinned
  against checked-in help output for six templates, and a sweep over all 46
  templates on SDK 10.0.111 found no anomalies — but that is one SDK on one
  machine, and a future SDK could lay its help out differently. The failure
  mode is visible (the preview shows the command) rather than silent, which is
  the mitigation the design leans on.
