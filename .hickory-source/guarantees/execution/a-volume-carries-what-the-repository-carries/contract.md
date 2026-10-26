<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Apply repository exclusions when seeding execution inputs

For: Authors running documents on their own machine and reviewers assessing their side effects.

Mount the repository as a volume. Ignored build outputs must not become accidental inputs, and a cell's own transcript must not make its next recording unreachable.

## What must happen

Repository volume seeding applies the repository's intended ignore filtering. Recording input digests additionally exclude the document's unstable generated products and weave targets. Literal authored inputs remain inputs. Removing the old self-mount warning does not remove ignore filtering or authorize a blanket exclusion of everything under an output directory.

## Limits of the evidence

Volume input filtering and executor confinement are separate boundaries. This does not police arbitrary filesystem writes by an unconfined process.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep the original ignore-filter promise and fold in the later removal of the obsolete self-mount warning. Delegate exact digest policy to the recording-key contract.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-07bf5ad2717c"></a>
## `apps/web/src/lib/scaffold.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Apply repository exclusions when seeding execution inputs](contract.md).

<a id="f-07bf5ad2717c-FieldValues"></a>
### `FieldValues`

The New Project form's own logic: what a field starts at, and which of them
end up on the command line.
The rule the whole file exists for: **a field the person did not change is
not written**. `dotnet new console` and `dotnet new console --framework
net10.0 --langVersion latest --no-restore false` scaffold the same project,
and only one of them is a command somebody can read a year later and see
what the decisions were. So the command carries the decisions and nothing
else — which is also what a person typing it by hand would do.
Nothing here renders the commit message. That is the server's
`commit_message`, shown through `POST /api/scaffold/preview`, so what the
dialog displays and what gets committed are the same function rather than
two that agree today.

<storycode:copy id="f-07bf5ad2717c-FieldValues">
// The New Project form's own logic: what a field starts at, and which of them
// end up on the command line.
//
// The rule the whole file exists for: **a field the person did not change is
// not written**. `dotnet new console` and `dotnet new console --framework
// net10.0 --langVersion latest --no-restore false` scaffold the same project,
// and only one of them is a command somebody can read a year later and see
// what the decisions were. So the command carries the decisions and nothing
// else — which is also what a person typing it by hand would do.
//
// Nothing here renders the commit message. That is the server's
// `commit_message`, shown through `POST /api/scaffold/preview`, so what the
// dialog displays and what gets committed are the same function rather than
// two that agree today.

import type { ChosenOption, TemplateOption, ScaffoldTemplate } from "../api/types";

/** Every field's value, by flag. `""` means "left to dotnet". */
export type FieldValues = Record<string, string>;

</storycode:copy>
<a id="f-07bf5ad2717c-NO_RESTORE"></a>
### `NO_RESTORE`

Restore is a build step, not a scaffold step.
Off by default, and not as tidiness: a restore fills `obj/` inside the
cell's output volume, and everything in that volume is what the ingest
considers. The project's `.gitignore` normally takes it back out — see
`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`
— but a notes folder that has never held a .NET project has no reason to
ignore `obj/`, and then five NuGet caches land in the document. Not
generating them is the answer that does not depend on a file being right.

<storycode:copy id="f-07bf5ad2717c-NO_RESTORE">
/**
 * Restore is a build step, not a scaffold step.
 *
 * Off by default, and not as tidiness: a restore fills `obj/` inside the
 * cell's output volume, and everything in that volume is what the ingest
 * considers. The project's `.gitignore` normally takes it back out — see
 * `docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`
 * — but a notes folder that has never held a .NET project has no reason to
 * ignore `obj/`, and then five NuGet caches land in the document. Not
 * generating them is the answer that does not depend on a file being right.
 */
export const NO_RESTORE = "--no-restore";

</storycode:copy>
<a id="f-07bf5ad2717c-initialValues"></a>
### `initialValues`

What each field starts at: the template's own default, so the form opens
showing what would happen if you pressed the button immediately.

<storycode:copy id="f-07bf5ad2717c-initialValues">
/** What each field starts at: the template's own default, so the form opens
 * showing what would happen if you pressed the button immediately. */
export function initialValues(options: TemplateOption[]): FieldValues {
  const values: FieldValues = {};
  for (const option of options) {
    values[option.flag] =
      option.flag === NO_RESTORE ? "true" : (option.default ?? "");
  }
  return values;
}

</storycode:copy>
<a id="f-07bf5ad2717c-unchanged"></a>
### `unchanged`

Whether a field is at the value `dotnet` would use anyway.

<storycode:copy id="f-07bf5ad2717c-unchanged">
/** Whether a field is at the value `dotnet` would use anyway. */
function unchanged(option: TemplateOption, value: string): boolean {
  if (option.kind === "bool") {
    return (value || "false").toLowerCase() === (option.default ?? "false").toLowerCase();
  }
  return value === "" || value === option.default;
}

</storycode:copy>
<a id="f-07bf5ad2717c-chosenOptions"></a>
### `chosenOptions`

The flags the command carries: the ones whose fields say something other
than the default.
A bool that has to be turned **off** needs its value spelled out —
`--nullable false` — because the bare flag means "true" everywhere in
`dotnet new`. A bool turned on is written bare, which is the form a person
types.

<storycode:copy id="f-07bf5ad2717c-chosenOptions">
/**
 * The flags the command carries: the ones whose fields say something other
 * than the default.
 *
 * A bool that has to be turned **off** needs its value spelled out —
 * `--nullable false` — because the bare flag means "true" everywhere in
 * `dotnet new`. A bool turned on is written bare, which is the form a person
 * types.
 */
export function chosenOptions(
  options: TemplateOption[],
  values: FieldValues,
): ChosenOption[] {
  const chosen: ChosenOption[] = [];
  for (const option of options) {
    const value = values[option.flag] ?? "";
    if (unchanged(option, value)) continue;
    if (option.kind === "bool") {
      chosen.push(
        value.toLowerCase() === "true"
          ? { flag: option.flag }
          : { flag: option.flag, value: "false" },
      );
    } else {
      chosen.push({ flag: option.flag, value });
    }
  }
  return chosen;
}

</storycode:copy>
<a id="f-07bf5ad2717c-initialLanguage"></a>
### `initialLanguage`

The language to ask `dotnet` for.
A template with one language takes no `--language` at all — `gitignore` and
`editorconfig` have none — and passing one to it is an error rather than a
no-op, so "the only language" and "no language" have to stay apart.

<storycode:copy id="f-07bf5ad2717c-initialLanguage">
/**
 * The language to ask `dotnet` for.
 *
 * A template with one language takes no `--language` at all — `gitignore` and
 * `editorconfig` have none — and passing one to it is an error rather than a
 * no-op, so "the only language" and "no language" have to stay apart.
 */
export function initialLanguage(template: ScaffoldTemplate): string {
  return template.default_language ?? template.languages[0] ?? "";
}

</storycode:copy>
<a id="f-07bf5ad2717c-slug"></a>
### `slug`

`Greeter` -> `greeter`. Mirrors `scaffold::slug`, and is used only to
*suggest* — the person may type anything into either field.

<storycode:copy id="f-07bf5ad2717c-slug">
/** `Greeter` -> `greeter`. Mirrors `scaffold::slug`, and is used only to
 * *suggest* — the person may type anything into either field. */
export function slug(name: string): string {
  const out = name
    .replace(/[^A-Za-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .toLowerCase();
  return out === "" ? "project" : out;
}

</storycode:copy>
<a id="f-07bf5ad2717c-grouped"></a>
### `grouped`

Templates under their first tag, in the order the catalogue lists them.
`dotnet new list` is alphabetical, which is the order a person who has used
the CLI already knows.

<storycode:copy id="f-07bf5ad2717c-grouped">
/** Templates under their first tag, in the order the catalogue lists them.
 * `dotnet new list` is alphabetical, which is the order a person who has used
 * the CLI already knows. */
export function grouped(
  templates: ScaffoldTemplate[],
): { group: string; templates: ScaffoldTemplate[] }[] {
  const byGroup = new Map<string, ScaffoldTemplate[]>();
  for (const template of templates) {
    const key = template.tags[0] ?? "Other";
    const list = byGroup.get(key);
    if (list) list.push(template);
    else byGroup.set(key, [template]);
  }
  return [...byGroup.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([group, list]) => ({ group, templates: list }));
}

</storycode:copy>
<a id="f-07bf5ad2717c-filterTemplates"></a>
### `filterTemplates`

Templates matching a query, over the name, the short names and the tags —
"api" should find `webapi` whether you know it by name or by tag.

<storycode:copy id="f-07bf5ad2717c-filterTemplates">
/** Templates matching a query, over the name, the short names and the tags —
 * "api" should find `webapi` whether you know it by name or by tag. */
export function filterTemplates(
  templates: ScaffoldTemplate[],
  query: string,
): ScaffoldTemplate[] {
  const q = query.trim().toLowerCase();
  if (q === "") return templates;
  return templates.filter((t) =>
    [t.name, ...t.short_names, ...t.tags].some((s) =>
      s.toLowerCase().includes(q),
    ),
  );
}

</storycode:copy>
<a id="f-07bf5ad2717c-problems"></a>
### `problems`

What the dialog cannot proceed without, keyed by field.

<storycode:copy id="f-07bf5ad2717c-problems">
/** What the dialog cannot proceed without, keyed by field. */
export function problems(name: string, output: string): Record<string, string> {
  const found: Record<string, string> = {};
  if (name.trim() === "")
    found.name = "A project needs a name — it becomes the root namespace and the assembly name.";
  const folder = output.trim().replace(/\/+$/, "");
  if (folder === "" || folder === ".")
    found.output =
      "Name a folder for the project's files. A scaffold is committed as its own tree, so it needs a folder of its own.";
  else if (folder.startsWith("/") || folder.split("/").includes(".."))
    found.output =
      "This is the folder's name, not a path. Where it goes is the Location above.";
  return found;
}

</storycode:copy>
<a id="f-07bf5ad2717c-joinPath"></a>
### `joinPath`

Join a location to the project's folder for display, on this machine's
own separator. Display only — the server does the real resolution, and it
is the only thing that knows which repository the result lands in.

<storycode:copy id="f-07bf5ad2717c-joinPath">
/** Join a location to the project's folder for display, on this machine's
 * own separator. Display only — the server does the real resolution, and it
 * is the only thing that knows which repository the result lands in. */
export function joinPath(location: string, folder: string, separator: string): string {
  const left = location.replace(/[/\\]+$/, "");
  const right = folder.trim().replace(/^[/\\]+|[/\\]+$/g, "");
  if (right === "") return left;
  return `${left}${separator}${right.replace(/\//g, separator)}`;
}
</storycode:copy>

### Assemble `apps/web/src/lib/scaffold.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/scaffold.ts"><storycode:paste select="#f-07bf5ad2717c-FieldValues"/><storycode:paste select="#f-07bf5ad2717c-NO_RESTORE"/><storycode:paste select="#f-07bf5ad2717c-initialValues"/><storycode:paste select="#f-07bf5ad2717c-unchanged"/><storycode:paste select="#f-07bf5ad2717c-chosenOptions"/><storycode:paste select="#f-07bf5ad2717c-initialLanguage"/><storycode:paste select="#f-07bf5ad2717c-slug"/><storycode:paste select="#f-07bf5ad2717c-grouped"/><storycode:paste select="#f-07bf5ad2717c-filterTemplates"/><storycode:paste select="#f-07bf5ad2717c-problems"/><storycode:paste select="#f-07bf5ad2717c-joinPath"/></storycode:file>

## Test the behavior: `a_sibling_documents_kept_recordings_are_not_in_the_key`

This is the maintained test fragment from `crates/hick-literate/src/lib/tests/tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-23d11bf7cb82-a_sibling_documents_kept_recordings_are_not_in_the_key"></a>
### `a_sibling_documents_kept_recordings_are_not_in_the_key`

<storycode:copy id="f-23d11bf7cb82-a_sibling_documents_kept_recordings_are_not_in_the_key">
use super::*;

// Protects docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md
// (the cache-digest half — `mounted_inputs_digest` must apply the same
// `.gitignore` filter `seed_from_directory` already applies at seed time).
#[test]
fn a_sibling_documents_kept_recordings_are_not_in_the_key() {
    let with = b"<h:doc>\n<h:exec container=\"c\">cmd<h:ingested key=\"k\" sha256=\"s\" at=\"d\">\nout\n</h:ingested></h:exec>\n</h:doc>\n";
    let without = b"<h:doc>\n<h:exec container=\"c\">cmd</h:exec>\n</h:doc>\n";
    assert_eq!(strip_kept_recordings(with), without.to_vec());
    assert_eq!(strip_kept_recordings(without), without.to_vec());
}

</storycode:copy>

## Test the behavior: `start`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_scaffold.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-5a76df8fd4d1-start"></a>
### `start`

A folder with one document in it, a git repository, and a `.gitignore`
that ignores what a .NET restore writes.
The `.gitignore` is not scene-setting: it is the ingest's filter, and
without it `obj/` lands in the document. See
`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`.

<storycode:copy id="f-5a76df8fd4d1-start">
/// A folder with one document in it, a git repository, and a `.gitignore`
/// that ignores what a .NET restore writes.
///
/// The `.gitignore` is not scene-setting: it is the ingest's filter, and
/// without it `obj/` lands in the document. See
/// `docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`.
async fn start() -> Session {
    start_inner(false).await
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/lib/scaffold.ts](contract.md#output-07bf5ad2717c)
- [crates/hick-blocks/src/lib.rs](../../../mechanisms/crates/hick-blocks/src/lib.rs.md#output-5260198b0a7b)
- [crates/hick-exec/src/dag.rs](../../../mechanisms/crates/hick-exec/src/dag.rs.md#output-e6f480388088)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)

### Tests and fixtures

- [crates/hick-literate/src/lib/tests/tests.rs](../../../mechanisms/crates/hick-literate/src/lib/tests/tests.rs.md#output-23d11bf7cb82)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/serve_scaffold.rs.md#output-5a76df8fd4d1)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
