<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Shared print requests do not attest physical output

For: Authors editing executable documents and the files those documents produce.

Given a focused printable buffer, when the human or agent requests Print, both
invoke the same full-buffer print callback. The title comes from the active tab
in the focused pane, rather than another pane's active tab. An absent editor is
unavailable; a platform callback error remains an error. A returned callback does
not establish that a dialog appeared, was confirmed rather than cancelled, or
produced physical output. Printer selection is browser/OS-owned, beyond what the
page's print API reports; the registered command states this limitation before
invocation and returns its actual callback result afterward.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/print-requests-do-not-attest-physical-output.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-9df0c3e5b86d"></a>
## `apps/web/src/lib/printing.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Shared print requests do not attest physical output](contract.md).

<a id="f-9df0c3e5b86d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9df0c3e5b86d-expression_statement">
// Guarantee: docs/guarantees/authoring/print-requests-do-not-attest-physical-output.md
import { afterEach, describe, expect, it } from "vitest";
import { printText, printTitleFor } from "./printing";

afterEach(() => {
  document.getElementById("hickory-print-sheet")?.remove();
  document.title = "";
});

</storycode:copy>
<a id="f-9df0c3e5b86d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9df0c3e5b86d-expression_statement-2">
describe("printing a buffer", () => {
  it("prints the WHOLE text, not the screenful CodeMirror has rendered", () => {
    // The bug this module exists to avoid: a virtualised editor prints the
    // viewport and the paper looks fine.
    let printed = "";
    const win = {
      document,
      print: () => {
        printed = document.getElementById("hickory-print-sheet")?.textContent ?? "";
      },
    } as unknown as Window;
    const text = Array.from({ length: 500 }, (_, i) => `line ${i}`).join("\n");
    printText({ title: "notes.md", text }, win);
    expect(printed).toContain("line 0");
    expect(printed).toContain("line 499");
  });

  it("names the page after the file, because that names the saved PDF", () => {
    let titleAtPrint = "";
    const win = {
      document,
      print: () => {
        titleAtPrint = document.title;
      },
    } as unknown as Window;
    document.title = "Hickory Docs";
    printText({ title: "quarterly.md", text: "x" }, win);
    expect(titleAtPrint).toBe("quarterly.md");
    expect(document.title).toBe("Hickory Docs");
  });

  it("puts the page back even when printing throws", () => {
    // A webview with printing disabled must not leave the app wearing a
    // printout's title and a hidden element full of somebody's notes.
    const win = {
      document,
      print: () => {
        throw new Error("no printer");
      },
    } as unknown as Window;
    document.title = "Hickory Docs";
    expect(() => printText({ title: "n.md", text: "secret" }, win)).toThrow();
    expect(document.getElementById("hickory-print-sheet")).toBeNull();
    expect(document.title).toBe("Hickory Docs");
  });

  it("writes the text as text, so a note about scripts stays one", () => {
    let html = "";
    const win = {
      document,
      print: () => {
        html = document.getElementById("hickory-print-sheet")?.innerHTML ?? "";
      },
    } as unknown as Window;
    printText({ title: "t", text: "<script>alert(1)</script>" }, win);
    expect(html).toContain("&lt;script&gt;");
  });
});

</storycode:copy>
<a id="f-9df0c3e5b86d-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9df0c3e5b86d-expression_statement-3">
describe("what the printout is called", () => {
  it("uses the file's own name", () => {
    expect(printTitleFor("notes/2026/today.md")).toBe("today.md");
    expect(printTitleFor("README.md")).toBe("README.md");
  });

  it("falls back to the whole path rather than to nothing", () => {
    expect(printTitleFor("")).toBe("");
    expect(printTitleFor("/")).toBe("/");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/printing.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/printing.test.ts"><storycode:paste select="#f-9df0c3e5b86d-expression_statement"/><storycode:paste select="#f-9df0c3e5b86d-expression_statement-2"/><storycode:paste select="#f-9df0c3e5b86d-expression_statement-3"/></storycode:file>

<a id="output-b290fcb26ba2"></a>
## `apps/web/src/lib/printing.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Shared print requests do not attest physical output](contract.md).

<a id="f-b290fcb26ba2-HOST_ID"></a>
### `HOST_ID`

Printing, which is harder than `window.print()` for one specific reason.
CodeMirror virtualises: only the lines near the viewport are in the DOM at
all. Printing the window therefore prints the screenful someone happens to
be looking at, silently, with no indication that the other four hundred
lines were dropped — which is a worse failure than refusing to print, since
the paper looks fine.
So printing does not use the editor's DOM. The full text is written into a
dedicated element that only exists during the print, and `@media print` in
styles.css hides everything else on the page. What comes out is what the
buffer holds, all of it.
The document TITLE matters more than it looks: every browser and webview
puts it in the page header and uses it as the default filename for
"Save as PDF". Leaving it as the app's own title would name every printed
note "Hickory Docs".
Where the printable copy is written. Created on demand, removed after.

<storycode:copy id="f-b290fcb26ba2-HOST_ID">
// Printing, which is harder than `window.print()` for one specific reason.
//
// CodeMirror virtualises: only the lines near the viewport are in the DOM at
// all. Printing the window therefore prints the screenful someone happens to
// be looking at, silently, with no indication that the other four hundred
// lines were dropped — which is a worse failure than refusing to print, since
// the paper looks fine.
//
// So printing does not use the editor's DOM. The full text is written into a
// dedicated element that only exists during the print, and `@media print` in
// styles.css hides everything else on the page. What comes out is what the
// buffer holds, all of it.
//
// The document TITLE matters more than it looks: every browser and webview
// puts it in the page header and uses it as the default filename for
// "Save as PDF". Leaving it as the app's own title would name every printed
// note "Hickory Docs".

/** Where the printable copy is written. Created on demand, removed after. */
const HOST_ID = "hickory-print-sheet";

</storycode:copy>
<a id="f-b290fcb26ba2-PrintJob"></a>
### `PrintJob`

<storycode:copy id="f-b290fcb26ba2-PrintJob">
export interface PrintJob {
  /** Names the page header, and the PDF a reader saves. */
  title: string;
  /** The whole buffer. Not a viewport, not a selection. */
  text: string;
}

</storycode:copy>
<a id="f-b290fcb26ba2-printText"></a>
### `printText`

Print `job`, then put the page back exactly as it was.
`print()` is synchronous in every engine we ship on — it blocks until the
dialog is dismissed — so the cleanup below runs after the dialog closes.
The `finally` is not decoration: an engine that throws here (a headless
build, a webview with printing disabled) must not leave the app wearing a
printout's title and a hidden element full of somebody's notes.

<storycode:copy id="f-b290fcb26ba2-printText">
/**
 * Print `job`, then put the page back exactly as it was.
 *
 * `print()` is synchronous in every engine we ship on — it blocks until the
 * dialog is dismissed — so the cleanup below runs after the dialog closes.
 * The `finally` is not decoration: an engine that throws here (a headless
 * build, a webview with printing disabled) must not leave the app wearing a
 * printout's title and a hidden element full of somebody's notes.
 */
export function printText(job: PrintJob, win: Window = window): void {
  const doc = win.document;
  const previousTitle = doc.title;
  const host = doc.createElement("div");
  host.id = HOST_ID;
  host.className = "print-sheet";

  const heading = doc.createElement("h1");
  heading.className = "print-sheet__title";
  heading.textContent = job.title;
  const body = doc.createElement("pre");
  body.className = "print-sheet__body";
  // textContent, never innerHTML: this is somebody's file, and a note
  // containing `<script>` is a note about scripts.
  body.textContent = job.text;
  host.append(heading, body);
  doc.body.appendChild(host);
  win.document.title = job.title;

  try {
    win.print();
  } finally {
    host.remove();
    win.document.title = previousTitle;
  }
}

</storycode:copy>
<a id="f-b290fcb26ba2-printTitleFor"></a>
### `printTitleFor`

The last path segment, which is what a reader calls the file.

<storycode:copy id="f-b290fcb26ba2-printTitleFor">
/** The last path segment, which is what a reader calls the file. */
export function printTitleFor(path: string): string {
  const name = path.split("/").filter(Boolean).pop();
  return name && name.length > 0 ? name : path;
}
</storycode:copy>

### Assemble `apps/web/src/lib/printing.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/printing.ts"><storycode:paste select="#f-b290fcb26ba2-HOST_ID"/><storycode:paste select="#f-b290fcb26ba2-PrintJob"/><storycode:paste select="#f-b290fcb26ba2-printText"/><storycode:paste select="#f-b290fcb26ba2-printTitleFor"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/printing.ts](contract.md#output-b290fcb26ba2)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/uiWorkspaceMenuActions.ts](../../../mechanisms/apps/web/src/views/uiWorkspaceMenuActions.ts.md#output-d4b2f02b516e)
- [apps/web/src/views/workspaceMenuActions.ts](../../../mechanisms/apps/web/src/views/workspaceMenuActions.ts.md#output-b6d5b22ec427)

### Tests and fixtures

- [apps/web/src/lib/printing.test.ts](contract.md#output-9df0c3e5b86d)
- [apps/web/src/views/workspaceMenuActions.test.ts](../../../mechanisms/apps/web/src/views/workspaceMenuActions.ts.md#output-c4f5a89c7b3e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
