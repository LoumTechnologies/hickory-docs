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

/** Whether a field is at the value `dotnet` would use anyway. */
function unchanged(option: TemplateOption, value: string): boolean {
  if (option.kind === "bool") {
    return (value || "false").toLowerCase() === (option.default ?? "false").toLowerCase();
  }
  return value === "" || value === option.default;
}

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

/** `Greeter` -> `greeter`. Mirrors `scaffold::slug`, and is used only to
 * *suggest* — the person may type anything into either field. */
export function slug(name: string): string {
  const out = name
    .replace(/[^A-Za-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .toLowerCase();
  return out === "" ? "project" : out;
}

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
    found.output = "The folder goes inside the one you have open.";
  return found;
}
