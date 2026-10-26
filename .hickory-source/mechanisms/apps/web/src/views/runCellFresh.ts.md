<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/runCellFresh.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7e75d733b57f"></a>
## `apps/web/src/views/runCellFresh.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7e75d733b57f-runCellFresh"></a>
### `runCellFresh`

<storycode:copy id="f-7e75d733b57f-runCellFresh">
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
</storycode:copy>

### Assemble `apps/web/src/views/runCellFresh.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/runCellFresh.ts"><storycode:paste select="#f-7e75d733b57f-runCellFresh"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-cdf419f278b9"></a>
## `apps/web/src/views/runCellFresh.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-cdf419f278b9-source"></a>
### `source`

<storycode:copy id="f-cdf419f278b9-source">
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { api, installMockHandler } from "../api/client";
import { runCellFresh } from "./runCellFresh";

const source = '# Nonzero exit QA\n<hick:exec id="cell">exit 7</hick:exec>\n';
</storycode:copy>
<a id="f-cdf419f278b9-receipt"></a>
### `receipt`

<storycode:copy id="f-cdf419f278b9-receipt">
async function receipt(overrides = {}) {
  const digest = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  return { document: "doc", source_hash: `sha256:${[...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("")}`, blocks: [{ kind: "exec", id: "cell", span: [71, 99] }], ...overrides };
}
</storycode:copy>
<a id="f-cdf419f278b9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-cdf419f278b9-expression_statement">
afterEach(() => { installMockHandler(null as never); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

</storycode:copy>
<a id="f-cdf419f278b9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-cdf419f278b9-expression_statement-2">
it("bypasses an older in-flight render and runs the current cell byte with exact source identity", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const fresh = await receipt();
  let release!: (value: unknown) => void;
  let renders = 0;
  const handler = vi.fn(async (method: string, path: string, body?: unknown) => {
    if (method === "GET") {
      if (++renders === 1) return new Promise(resolve => { release = resolve; });
      return fresh;
    }
    expect(path).toBe("/api/docs/doc/blocks/71/run");
    expect(body).toEqual({ expected_source_hash: fresh.source_hash });
    return { outcome: "run", run_id: "new-run", cells: ["cell"] };
  });
  installMockHandler(handler);
  const older = api.render("doc");
  await expect(runCellFresh("doc", "cell", () => source)).resolves.toMatchObject({ run_id: "new-run" });
  expect(renders).toBe(2);
  release({ blocks: [{ kind: "exec", id: "cell", span: [73, 101] }] });
  await older;
  expect(handler.mock.calls.filter(([method]) => method === "POST")).toHaveLength(1);
});

</storycode:copy>
<a id="f-cdf419f278b9-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-cdf419f278b9-expression_statement-3">
it.each(["wrong document", "wrong source", "missing receipt", "removed cell", "editor changed"])("refuses %s before any action is dispatched", async reason => {
  vi.stubGlobal("crypto", webcrypto);
  let current = source;
  const fresh = await receipt(reason === "wrong document" ? { document: "other" }
    : reason === "wrong source" ? { source_hash: "sha256:older" }
    : reason === "missing receipt" ? { source_hash: undefined }
    : reason === "removed cell" ? { blocks: [] } : {});
  const handler = vi.fn(async () => {
    if (reason === "editor changed") current = source + "new edit";
    return fresh;
  });
  installMockHandler(handler);
  await expect(runCellFresh("doc", "cell", () => current)).rejects.toThrow(reason === "removed cell" ? "no longer" : "Save it");
  expect(handler).toHaveBeenCalledTimes(1);
});

</storycode:copy>
<a id="f-cdf419f278b9-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-cdf419f278b9-expression_statement-4">
it("never retries the action after an uncertain dispatched failure", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const fresh = await receipt();
  const handler = vi.fn(async (method: string) => {
    if (method === "GET") return fresh;
    throw new Error("connection lost after dispatch");
  });
  installMockHandler(handler);
  await expect(runCellFresh("doc", "cell", () => source)).rejects.toThrow("connection lost after dispatch");
  expect(handler.mock.calls.filter(([method]) => method === "POST")).toHaveLength(1);
});
</storycode:copy>

### Assemble `apps/web/src/views/runCellFresh.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/runCellFresh.test.ts"><storycode:paste select="#f-cdf419f278b9-source"/><storycode:paste select="#f-cdf419f278b9-receipt"/><storycode:paste select="#f-cdf419f278b9-expression_statement"/><storycode:paste select="#f-cdf419f278b9-expression_statement-2"/><storycode:paste select="#f-cdf419f278b9-expression_statement-3"/><storycode:paste select="#f-cdf419f278b9-expression_statement-4"/></storycode:file>


</storycode:doc>
