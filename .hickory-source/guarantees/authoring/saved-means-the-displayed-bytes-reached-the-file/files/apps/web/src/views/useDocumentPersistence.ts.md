<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useDocumentPersistence.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Saved means the displayed bytes reached the file](../../../../../contract.md)

Captured SHA-256: `ffdb9debe0bf9c525b1ede7e694c1442350d60234c80f70848aeb5b1c4469c1c`.

<book:entry path="apps/web/src/views/useDocumentPersistence.ts">
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
</book:entry>

</book:doc>
