import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { Doc } from "../api/types";

export type SyncState = "idle" | "editing" | "saved" | "failed";

/** Saved means the current text was acknowledged on disk. Rendering is a
 * separate operation: a syntax error must never turn a successful save into
 * an unsaved buffer. Ignore replies for text superseded while waiting. */
export function useDocumentPersistence(docId: string, doc: Doc | null, source: string | null, confirmed: (doc: Doc) => void) {
  const [savedSource, setSavedSource] = useState<string | null>(null);
  const [syncState, setSyncState] = useState<SyncState>("idle");
  const confirmedRef = useRef(confirmed); confirmedRef.current = confirmed;
  useEffect(() => {
    if (doc && savedSource === null) setSavedSource(doc.source);
  }, [doc, savedSource]);
  useEffect(() => {
    if (source === null || !doc) return;
    if (source === doc.source) {
      setSavedSource(source);
      setSyncState(current => current === "idle" ? "idle" : "saved");
      return;
    }
    setSyncState("editing");
    let live = true;
    let attempts = 0;
    let timer: ReturnType<typeof setTimeout>;
    const check = async () => {
      try {
        const stored = await api.doc(docId);
        if (!live) return;
        if (stored.source !== source) {
          if (++attempts >= 30) setSyncState("failed");
          else timer = setTimeout(check, 300);
          return;
        }
        setSavedSource(source); setSyncState("saved"); confirmedRef.current(stored);
      } catch {
        if (live) setSyncState("failed");
      }
    };
    timer = setTimeout(check, 1200);
    return () => { live = false; clearTimeout(timer); };
  }, [docId, source, doc?.source]); // eslint-disable-line react-hooks/exhaustive-deps
  return { savedSource, setSavedSource, syncState, setSyncState };
}
