<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useDocumentRender.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `030c8098fd5b977f13d9eaa1f9f6aebc76a4ed3c38316de1b8fa1ef6f66da6f9`.

<book:entry path="apps/web/src/views/useDocumentRender.test.tsx">
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { installMockHandler } from "../api/client";
import { useDocumentRender } from "./useDocumentRender";
async function receipt(source: string, id: string) {
  const hash = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  return { document: "d", source_hash: [...new Uint8Array(hash)].map(byte => byte.toString(16).padStart(2, "0")).join(""), blocks: [{ kind: "prose", id }] };
}
afterEach(() => { cleanup(); installMockHandler(null as never); vi.unstubAllGlobals(); });
describe("source-backed render refresh", () => {
  it("accepts the newest acknowledged projection and refuses late older source responses", async () => {
    vi.stubGlobal("crypto", webcrypto);
    const replies: ((value: unknown) => void)[] = [];
    installMockHandler(() => new Promise(resolve => { replies.push(resolve); }));
    let source = "before";
    const accept = vi.fn(), failed = vi.fn();
    const hook = renderHook(() => useDocumentRender("d", () => source, accept, failed));
    const older = hook.result.current.refreshRender(source);
    source = "after adding a cell";
    const newer = hook.result.current.refreshRender(source);
    const current = await receipt(source, "new-cell");
    await act(async () => { replies[1](current); await newer; });
    await act(async () => { replies[0](await receipt("before", "old-cell")); await older; });
    expect(accept).toHaveBeenCalledTimes(1);
    expect(accept).toHaveBeenCalledWith(current.blocks);
  });
  it("does not publish a projection for another document or superseded editor text", async () => {
    vi.stubGlobal("crypto", webcrypto);
    let source = "saved";
    const accept = vi.fn();
    let answer = { ...await receipt(source, "cell"), document: "other" };
    installMockHandler(async () => answer);
    const hook = renderHook(() => useDocumentRender("d", () => source, accept, vi.fn()));
    await hook.result.current.refreshRender(source);
    expect(accept).not.toHaveBeenCalled();
    answer = await receipt("saved", "old-cell");
    installMockHandler(async () => { source = "new unsaved source"; return answer; });
    await hook.result.current.refreshRender("saved");
    expect(accept).not.toHaveBeenCalled();
  });
  it("does not publish an in-flight refresh after its document session unmounts", async () => {
    vi.stubGlobal("crypto", webcrypto);
    let reply!: (value: unknown) => void;
    installMockHandler(() => new Promise(resolve => { reply = resolve; }));
    const accept = vi.fn();
    const hook = renderHook(() => useDocumentRender("d", () => "saved", accept, vi.fn()));
    const waiting = hook.result.current.refreshRender("saved");
    hook.unmount();
    reply(await receipt("saved", "cell"));
    await waiting;
    expect(accept).not.toHaveBeenCalled();
  });
});
</book:entry>

</book:doc>
