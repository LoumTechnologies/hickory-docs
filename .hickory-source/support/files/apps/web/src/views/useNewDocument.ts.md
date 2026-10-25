<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useNewDocument.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)

Captured SHA-256: `10547acebf8b3900f6745357d59c9a1674eff7913f611e4b2db8a041c1cb5258`.

<book:entry path="apps/web/src/views/useNewDocument.ts">
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
</book:entry>

</book:doc>
