// Answer every request for a new document, not only a change of address.
//
// The route effect opens an untitled buffer when the hash CHANGES to #/new;
// this opens one whenever `newDocument()` fires its event — including the
// second time, when the hash has not moved. `openUntitledTab` re-activates an
// existing Untitled tab rather than making a second, so the two firing
// together on a first request is harmless. See
// docs/guarantees/authoring/new-document-is-an-act.md.

import { useEffect } from "react";
import type { Dispatch, SetStateAction } from "react";

import { NEW_DOCUMENT_EVENT } from "../router";
import type { Layout } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";

export function useNewDocument(setLayout: Dispatch<SetStateAction<Layout>>): void {
  useEffect(() => {
    const onNewDocument = () => setLayout((current) => openUntitledTab(current));
    window.addEventListener(NEW_DOCUMENT_EVENT, onNewDocument);
    return () => window.removeEventListener(NEW_DOCUMENT_EVENT, onNewDocument);
  }, [setLayout]);
}
