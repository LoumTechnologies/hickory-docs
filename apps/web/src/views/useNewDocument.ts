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
