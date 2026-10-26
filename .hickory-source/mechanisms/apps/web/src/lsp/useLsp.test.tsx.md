<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/useLsp.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2902d775e6f7"></a>
## `apps/web/src/lsp/useLsp.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-2902d775e6f7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2902d775e6f7-expression_statement">
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createLspChannel, encodeLspFrame, type JsonRpcMessage } from "./channel";
import type { Realtime } from "../api/realtime";
import { useLsp } from "./useLsp";

afterEach(() => { cleanup(); vi.useRealTimers(); });

</storycode:copy>
<a id="f-2902d775e6f7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2902d775e6f7-expression_statement-2">
it("flushes the newest source/version before immediate rename or code actions without sleeping or duplicate sync", async () => {
  vi.useFakeTimers();
  const sent: Array<{ method: string; params: any; id?: number }> = [];
  let providerSource = "", providerVersion = 0;
  const channel = createLspChannel({ send: frame => {
    const msg = JSON.parse(new TextDecoder().decode(frame.subarray(1)));
    sent.push(msg);
    if (msg.method === "textDocument/didOpen") { providerSource = msg.params.textDocument.text; providerVersion = msg.params.textDocument.version; }
    if (msg.method === "textDocument/didChange") { providerSource = msg.params.contentChanges[0].text; providerVersion = msg.params.textDocument.version; }
    if (msg.id) {
      const result = msg.method === "textDocument/prepareRename"
        ? { range: { start: { line: 0, character: providerSource.indexOf("square") }, end: { line: 0, character: providerSource.indexOf("square") + 6 } }, placeholder: "square" }
        : [];
      channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", id: msg.id, result } as JsonRpcMessage));
    }
  } });
  const realtime = { lsp: () => channel } as unknown as Realtime;
  const hook = renderHook(({ text }) => useLsp(realtime, "Application Support/main.cpp", text), { initialProps: { text: "int main() {}" } });
  const client = hook.result.current.client!;
  act(() => channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", method: "textDocument/publishDiagnostics", params: { uri: "hick:///Application Support/main.cpp", diagnostics: [{ range: { start: { line: 0, character: 4 }, end: { line: 0, character: 8 } }, message: "provider diagnostic" }] } })));
  expect(hook.result.current.diagnostics[0].message).toBe("provider diagnostic");
  hook.rerender({ text: "int square(int value);" });
  const prepared = await client.prepareRename(hook.result.current.uri, { line: 0, character: 5 });
  expect(prepared).toMatchObject({ start: { character: 4 }, end: { character: 10 } });
  expect(providerVersion).toBe(2);
  expect(sent.slice(-2).map(msg => msg.method)).toEqual(["textDocument/didChange", "textDocument/prepareRename"]);
  expect(providerSource).toBe("int square(int value);");
  await act(async () => { vi.advanceTimersByTime(300); });
  expect(sent.filter(msg => msg.method === "textDocument/didChange")).toHaveLength(1);
  hook.rerender({ text: "int square(int value) { return value }" });
  await client.codeActions(hook.result.current.uri, { start: { line: 0, character: 36 }, end: { line: 0, character: 36 } });
  expect(providerVersion).toBe(3);
  expect(sent.slice(-2).map(msg => msg.method)).toEqual(["textDocument/didChange", "textDocument/codeAction"]);
  hook.unmount();
  await expect(client.prepareRename(hook.result.current.uri, { line: 0, character: 5 })).rejects.toThrow("disposed");
});
</storycode:copy>

### Assemble `apps/web/src/lsp/useLsp.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/useLsp.test.tsx"><storycode:paste select="#f-2902d775e6f7-expression_statement"/><storycode:paste select="#f-2902d775e6f7-expression_statement-2"/></storycode:file>

</storycode:doc>
