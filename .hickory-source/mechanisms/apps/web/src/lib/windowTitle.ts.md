<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/windowTitle.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1e0e74b83588"></a>
## `apps/web/src/lib/windowTitle.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1e0e74b83588-TitleParts"></a>
### `TitleParts`

What the window calls itself, as one pure precedence rule:
1. the custom override from Settings (GET/PUT /api/settings/ui) — used
verbatim, because a person who typed a title meant that exact title;
2. else the open project folder's name (GET /api/files' `root`);
3. else the focused file's name;
4. else just the app.
This covers the in-window half (document.title). The native window title
the desktop shell sets at launch follows the same rule — see
apps/desktop/src-tauri/src/lib.rs — but does not update live: there is no
IPC from this page to the shell, deliberately.

<storycode:copy id="f-1e0e74b83588-TitleParts">
// What the window calls itself, as one pure precedence rule:
//
//   1. the custom override from Settings (GET/PUT /api/settings/ui) — used
//      verbatim, because a person who typed a title meant that exact title;
//   2. else the open project folder's name (GET /api/files' `root`);
//   3. else the focused file's name;
//   4. else just the app.
//
// This covers the in-window half (document.title). The native window title
// the desktop shell sets at launch follows the same rule — see
// apps/desktop/src-tauri/src/lib.rs — but does not update live: there is no
// IPC from this page to the shell, deliberately.

export interface TitleParts {
  /** The custom override, or null/absent when none is set. */
  custom?: string | null;
  /** The open folder, as the server names it (may be a path). */
  folder?: string | null;
  /** The focused file's display name ("Untitled" counts). */
  file?: string | null;
}

</storycode:copy>
<a id="f-1e0e74b83588-APP"></a>
### `APP`

<storycode:copy id="f-1e0e74b83588-APP">
const APP = "Hickory Docs";

</storycode:copy>
<a id="f-1e0e74b83588-folderName"></a>
### `folderName`

A folder value may arrive as a path; the title wants its last segment.

<storycode:copy id="f-1e0e74b83588-folderName">
/** A folder value may arrive as a path; the title wants its last segment. */
function folderName(folder: string): string {
  const name = folder.replace(/[/\\]+$/, "").split(/[/\\]/).pop() ?? folder;
  return name === "" ? folder : name;
}

</storycode:copy>
<a id="f-1e0e74b83588-windowTitle"></a>
### `windowTitle`

<storycode:copy id="f-1e0e74b83588-windowTitle">
export function windowTitle({ custom, folder, file }: TitleParts): string {
  const override = custom?.trim();
  if (override) return override;
  const project = folder?.trim();
  if (project) return `${folderName(project)} — ${APP}`;
  const name = file?.trim();
  if (name) return `${name} — ${APP}`;
  return APP;
}
</storycode:copy>

### Assemble `apps/web/src/lib/windowTitle.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/windowTitle.ts"><storycode:paste select="#f-1e0e74b83588-TitleParts"/><storycode:paste select="#f-1e0e74b83588-APP"/><storycode:paste select="#f-1e0e74b83588-folderName"/><storycode:paste select="#f-1e0e74b83588-windowTitle"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b2c3294caa9d"></a>
## `apps/web/src/lib/windowTitle.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b2c3294caa9d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b2c3294caa9d-expression_statement">
import { describe, expect, it } from "vitest";

import { windowTitle } from "./windowTitle";

describe("window title precedence", () => {
  it("uses the custom override verbatim when one is set", () => {
    expect(
      windowTitle({ custom: "My Lab Notebook", folder: "notes", file: "a.md" }),
    ).toBe("My Lab Notebook");
  });

  it("falls back to the project folder name when there is no override", () => {
    expect(windowTitle({ custom: null, folder: "notes", file: "a.md" })).toBe(
      "notes — Hickory Docs",
    );
  });

  it("reduces a folder path to its last segment", () => {
    expect(windowTitle({ folder: "/home/mock/notebook", file: "a.md" })).toBe(
      "notebook — Hickory Docs",
    );
    expect(windowTitle({ folder: "C:\\Users\\me\\notes" })).toBe("notes — Hickory Docs");
  });

  it("falls back to the focused file name when there is no folder either", () => {
    expect(windowTitle({ custom: "", folder: null, file: "weave-demo.md" })).toBe(
      "weave-demo.md — Hickory Docs",
    );
  });

  it("treats a blank override as unset, not as an empty title", () => {
    expect(windowTitle({ custom: "   ", folder: "notes" })).toBe("notes — Hickory Docs");
  });

  it("is just the app when nothing at all is known", () => {
    expect(windowTitle({})).toBe("Hickory Docs");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/windowTitle.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/windowTitle.test.ts"><storycode:paste select="#f-b2c3294caa9d-expression_statement"/></storycode:file>


</storycode:doc>
