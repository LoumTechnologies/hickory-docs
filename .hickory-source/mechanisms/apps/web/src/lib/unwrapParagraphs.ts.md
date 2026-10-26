<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/unwrapParagraphs.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-287fdbcc749d"></a>
## `apps/web/src/lib/unwrapParagraphs.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Paragraphs Unwrap By Default](../../../../../guarantees/authoring/paragraphs-unwrap-by-default/contract.md).

<a id="f-287fdbcc749d-UNWRAP_PARAGRAPHS_KEY"></a>
### `UNWRAP_PARAGRAPHS_KEY`

Editing preference, persisted per browser profile like word navigation.

<storycode:copy id="f-287fdbcc749d-UNWRAP_PARAGRAPHS_KEY">
/** Editing preference, persisted per browser profile like word navigation. */
export const UNWRAP_PARAGRAPHS_KEY = "hickory.unwrapParagraphs";

</storycode:copy>
<a id="f-287fdbcc749d-loadUnwrapParagraphs"></a>
### `loadUnwrapParagraphs`

<storycode:copy id="f-287fdbcc749d-loadUnwrapParagraphs">
export function loadUnwrapParagraphs(): boolean {
  return typeof localStorage === "undefined" ||
    localStorage.getItem(UNWRAP_PARAGRAPHS_KEY) !== "false";
}

</storycode:copy>
<a id="f-287fdbcc749d-saveUnwrapParagraphs"></a>
### `saveUnwrapParagraphs`

<storycode:copy id="f-287fdbcc749d-saveUnwrapParagraphs">
export function saveUnwrapParagraphs(on: boolean): void {
  localStorage.setItem(UNWRAP_PARAGRAPHS_KEY, String(on));
}
</storycode:copy>

### Assemble `apps/web/src/lib/unwrapParagraphs.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/unwrapParagraphs.ts"><storycode:paste select="#f-287fdbcc749d-UNWRAP_PARAGRAPHS_KEY"/><storycode:paste select="#f-287fdbcc749d-loadUnwrapParagraphs"/><storycode:paste select="#f-287fdbcc749d-saveUnwrapParagraphs"/></storycode:file>

</storycode:doc>
