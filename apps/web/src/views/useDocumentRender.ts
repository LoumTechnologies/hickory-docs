import { useCallback, useEffect, useRef } from "react";
import { request } from "../api/client";
import type { Block, RenderResponse } from "../api/types";

/** A source-backed projection. Save acknowledgement and render validity are
 * independent: invalid syntax can remain saved without hiding its editor. */
export function useDocumentRender(document: string, currentSource: () => string | null,
  accept: (blocks: Block[]) => void, failed: (error: string | null) => void) {
  const latest = useRef({ currentSource, accept, failed }); latest.current = { currentSource, accept, failed };
  const sequence = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refreshRender = useCallback(async (expectedSource?: string) => {
    if (timer.current !== null) { clearTimeout(timer.current); timer.current = null; }
    const revision = ++sequence.current;
    const requested = expectedSource ?? latest.current.currentSource();
    try {
      // A save must not join a pre-save api.render already in flight.
      const render = await request<RenderResponse & { document: string; source_hash: string }>("GET", `/api/docs/${document}/render`);
      const source = requested ?? latest.current.currentSource();
      if (render.document !== document || typeof render.source_hash !== "string") throw new Error("The document render has no current source receipt. Refresh the document.");
      if (source !== null) {
        const hash = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
        const expected = [...new Uint8Array(hash)].map(byte => byte.toString(16).padStart(2, "0")).join("");
        if (render.source_hash.replace(/^sha256:/, "") !== expected || latest.current.currentSource() !== source) return;
      }
      if (revision !== sequence.current) return;
      latest.current.accept(render.blocks); latest.current.failed(null);
    } catch (error) {
      if (revision === sequence.current && (requested === null || latest.current.currentSource() === requested)) {
        latest.current.failed(error instanceof Error ? error.message : String(error));
      }
    }
  }, [document]);
  const scheduleRender = useCallback(() => {
    if (timer.current === null) timer.current = setTimeout(() => { timer.current = null; void refreshRender(); }, 150);
  }, [refreshRender]);
  useEffect(() => () => { ++sequence.current; if (timer.current !== null) clearTimeout(timer.current); }, [document]);
  return { refreshRender, scheduleRender };
}
