<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/router.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-415811f3ab59"></a>
## `apps/web/src/router.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [New Document Creates Another Unnamed Buffer](../../../../guarantees/authoring/new-document-is-an-act/contract.md), [The Lineage Browser Shows One Column Per Stage](../../../../guarantees/authoring/the-lineage-browser-shows-one-column-per-stage/contract.md), [Typed Text Is Prose, Not A Transcript](../../../../guarantees/authoring/typed-text-is-prose-not-a-transcript/contract.md).

<a id="f-415811f3ab59-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-415811f3ab59-expression_statement">
import { describe, expect, it } from "vitest";
import { parseRoute } from "./router";

describe("routes", () => {
  it("routes the bare domain to the landing decision, never a chooser", () => {
    expect(parseRoute("")).toEqual({ name: "landing" });
    expect(parseRoute("#")).toEqual({ name: "landing" });
    expect(parseRoute("#/")).toEqual({ name: "landing" });
  });

  it("routes a document", () => {
    expect(parseRoute("#/docs/abc")).toEqual({ name: "doc", id: "abc" });
  });

  it("lands the retired documents-list route like any stale bookmark", () => {
    // The folder's files are a pane now (FolderTreePane), not a page.
    expect(parseRoute("#/documents")).toEqual({ name: "landing" });
  });

  it("routes the in-memory untitled document", () => {
    expect(parseRoute("#/new")).toEqual({ name: "new" });
  });

  it("routes a deliberately folderless window", () => {
    expect(parseRoute("#/blank")).toEqual({ name: "blank" });
  });

  it("routes settings (LLM API keys)", () => {
    expect(parseRoute("#/settings")).toEqual({ name: "settings" });
  });

  it("routes retired hosted-product paths to the landing decision", () => {
    // These routes no longer exist (local-only.md); a stale bookmark should
    // land somewhere useful rather than on a blank screen.
    expect(parseRoute("#/login")).toEqual({ name: "landing" });
    expect(parseRoute("#/verify?token=abc")).toEqual({ name: "landing" });
  });
});

</storycode:copy>
<a id="f-415811f3ab59-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-415811f3ab59-expression_statement-2">
describe("the lineage route", () => {
  it("is project-scoped, because a chain crosses documents", () => {
    expect(parseRoute("#/projects/abc123/lineage")).toEqual({ name: "lineage", id: "abc123" });
  });
});

</storycode:copy>
<a id="f-415811f3ab59-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-415811f3ab59-expression_statement-3">
describe("the scratchpad route", () => {
  it("is its own route, so it can be linked and bookmarked", () => {
    expect(parseRoute("#/scratchpad")).toEqual({ name: "scratchpad" });
  });
});

</storycode:copy>
<a id="f-415811f3ab59-expression_statement-4"></a>
### `expression_statement`

Protects docs/guarantees/authoring/new-document-is-an-act.md: asking for a
new document works however many times, including when the address already
says #/new — which is exactly when `navigate("/new")` alone did nothing.

<storycode:copy id="f-415811f3ab59-expression_statement-4">
// Protects docs/guarantees/authoring/new-document-is-an-act.md: asking for a
// new document works however many times, including when the address already
// says #/new — which is exactly when `navigate("/new")` alone did nothing.
describe("newDocument", () => {
  it("fires the event even when the hash is already #/new", async () => {
    const { NEW_DOCUMENT_EVENT, newDocument } = await import("./router");
    location.hash = "#/new";
    let fired = 0;
    const listener = () => fired++;
    window.addEventListener(NEW_DOCUMENT_EVENT, listener);
    newDocument();
    newDocument();
    window.removeEventListener(NEW_DOCUMENT_EVENT, listener);
    expect(fired).toBe(2);
    expect(location.hash).toBe("#/new");
  });
});


</storycode:copy>
<a id="f-415811f3ab59-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-415811f3ab59-expression_statement-5">
it("routes a selected file without opening a folder", () => {
  expect(parseRoute("#/file/%2Ftmp%2FMy%20note.md")).toEqual({ name: "new", file: "/tmp/My note.md" });
});
</storycode:copy>

### Assemble `apps/web/src/router.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/router.test.ts"><storycode:paste select="#f-415811f3ab59-expression_statement"/><storycode:paste select="#f-415811f3ab59-expression_statement-2"/><storycode:paste select="#f-415811f3ab59-expression_statement-3"/><storycode:paste select="#f-415811f3ab59-expression_statement-4"/><storycode:paste select="#f-415811f3ab59-expression_statement-5"/></storycode:file>

</storycode:doc>
