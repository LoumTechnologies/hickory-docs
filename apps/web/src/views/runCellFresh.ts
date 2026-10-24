import { api, request } from "../api/client";
import type { RenderResponse } from "../api/types";

/** Resolve coordinates from a new saved-source receipt, not a retained card.
 * Never retry the action: a lost response may already have started execution. */
export async function runCellFresh(document: string, cell: string, currentSource: () => string | null) {
  const source = currentSource();
  if (source === null) throw new Error("The document is not loaded. Reopen it before running this cell.");
  const bytes = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  const expected = [...new Uint8Array(bytes)].map(byte => byte.toString(16).padStart(2, "0")).join("");
  // Deliberately bypass api.render's in-flight deduplication: an earlier
  // render may describe the source before the acknowledged save.
  const render = await request<RenderResponse & { document: string; source_hash: string }>("GET", `/api/docs/${document}/render`);
  if (render.document !== document || typeof render.source_hash !== "string"
      || render.source_hash.replace(/^sha256:/, "") !== expected || currentSource() !== source) {
    throw new Error("The document changed or has unsaved edits. Save it, then run this cell again.");
  }
  const block = render.blocks.find(block => block.kind === "exec" && block.id === cell);
  if (!block) throw new Error("This cell is no longer in the saved document. Refresh the document before running it.");
  return api.blockAction(document, block.span[0], "run", { expected_source_hash: render.source_hash });
}
