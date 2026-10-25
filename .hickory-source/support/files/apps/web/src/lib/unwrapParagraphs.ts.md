<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/unwrapParagraphs.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Markdown Paragraphs Unwrap By Default](../../../../../../guarantees/authoring/paragraphs-unwrap-by-default/contract.md)

Captured SHA-256: `431321eb01571bcbd2495fc54bb8bb1425e5354436506407092495a93d9778d8`.

<book:entry path="apps/web/src/lib/unwrapParagraphs.ts">
/** Editing preference, persisted per browser profile like word navigation. */
export const UNWRAP_PARAGRAPHS_KEY = "hickory.unwrapParagraphs";

export function loadUnwrapParagraphs(): boolean {
  return typeof localStorage === "undefined" ||
    localStorage.getItem(UNWRAP_PARAGRAPHS_KEY) !== "false";
}

export function saveUnwrapParagraphs(on: boolean): void {
  localStorage.setItem(UNWRAP_PARAGRAPHS_KEY, String(on));
}
</book:entry>

</book:doc>
