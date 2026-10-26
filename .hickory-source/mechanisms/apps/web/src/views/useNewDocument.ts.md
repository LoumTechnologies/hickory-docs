<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useNewDocument.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6ece107bbb2b"></a>
## `apps/web/src/views/useNewDocument.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md).

<a id="f-6ece107bbb2b-useNewDocument"></a>
### `useNewDocument`

Each explicit New request creates another unnamed buffer.

<storycode:copy id="f-6ece107bbb2b-useNewDocument">
// Each explicit New request creates another unnamed buffer.
import { useEffect } from "react";
import type { Dispatch, SetStateAction } from "react";

import { NEW_DOCUMENT_EVENT } from "../router";
import type { Layout } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";

export function useNewDocument(setLayout: Dispatch<SetStateAction<Layout>>): void {
  useEffect(() => {
    const onNewDocument = () => setLayout((current) => openUntitledTab(current, true));
    window.addEventListener(NEW_DOCUMENT_EVENT, onNewDocument);
    return () => window.removeEventListener(NEW_DOCUMENT_EVENT, onNewDocument);
  }, [setLayout]);
}
</storycode:copy>

### Assemble `apps/web/src/views/useNewDocument.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useNewDocument.ts"><storycode:paste select="#f-6ece107bbb2b-useNewDocument"/></storycode:file>

</storycode:doc>
