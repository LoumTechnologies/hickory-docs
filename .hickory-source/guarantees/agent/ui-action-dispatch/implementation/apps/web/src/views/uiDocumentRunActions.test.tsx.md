<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiDocumentRunActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-14d5d529a63c"></a>
## `apps/web/src/views/uiDocumentRunActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-14d5d529a63c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-14d5d529a63c-expression_statement">
import { act, cleanup, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { api } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useDocumentRunActions } from "./uiDocumentRunActions";
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });
</storycode:copy>
<a id="f-14d5d529a63c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-14d5d529a63c-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("refuses unsaved or concurrently changed source and queues the shared check with its digest", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  let text = "# current\n";
  const doc = vi.spyOn(api, "doc").mockImplementation(async () => ({ source: "# saved\n" } as never));
  const run = vi.fn().mockResolvedValue("run-real"), verify = vi.fn().mockResolvedValue("check-real");
  renderHook(() => useDocumentRunActions("doc", () => text, null, run, verify), { wrapper });
  let n = 0;
  const invoke = async (label = "Verify saved document") => {
    const action = registry.observe(registry.groups().groups[0]).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: {}, request_id: String(++n) }); }); return result;
  };
  expect(await invoke()).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); expect(verify).not.toHaveBeenCalled();
  doc.mockImplementation(async () => { const saved = text; text = "# changed during read\n"; return { source: saved } as never; });
  expect(await invoke()).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); expect(verify).not.toHaveBeenCalled();
  doc.mockImplementation(async () => ({ source: text } as never));
  expect(await invoke()).toMatchObject({ status: "result", outcome: { status: "queued", job: "check-real", value: { document: "doc", purpose: "document-check" } } });
  const expected = [...new Uint8Array(await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(text)))].map(b => b.toString(16).padStart(2, "0")).join("");
  expect(verify).toHaveBeenCalledWith(expected); expect(run).not.toHaveBeenCalled();
  expect(await invoke("Run saved document")).toMatchObject({ status: "result", outcome: { status: "queued", job: "run-real", value: { document: "doc", purpose: "document-run" } } });
});
</storycode:copy>

### Assemble `apps/web/src/views/uiDocumentRunActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiDocumentRunActions.test.tsx"><storycode:paste select="#f-14d5d529a63c-expression_statement"/><storycode:paste select="#f-14d5d529a63c-expression_statement-2"/></storycode:file>

</storycode:doc>
