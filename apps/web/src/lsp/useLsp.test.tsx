import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createLspChannel, encodeLspFrame, type JsonRpcMessage } from "./channel";
import type { Realtime } from "../api/realtime";
import * as revisionProof from "./diagnosticRevision";
import { useLsp } from "./useLsp";

afterEach(() => { cleanup(); vi.useRealTimers(); });

it("keeps messages immediate while binding engine hashes independently of client counters and retiring obsolete proof", async () => {
  // Contract: .hickory-source/frontends/contract.md
  let finish!: (digest: string) => void;
  const digest = "sha256:" + "a".repeat(64);
  const proof = vi.spyOn(revisionProof,"sourceDigest").mockImplementation(() => new Promise(resolve => { finish=resolve; }));
  const channel = createLspChannel({send:()=>{}}), realtime = {lsp:()=>channel} as unknown as Realtime;
  const hook=renderHook(({text})=>useLsp(realtime,"proof.py",text),{initialProps:{text:"value"}});
  const send=(message:string,hash:string|null,version=999)=>act(()=>channel.handleFrame(encodeLspFrame({jsonrpc:"2.0",method:"textDocument/publishDiagnostics",params:{uri:"hick:///proof.py",version,diagnostics:[{range:{start:{line:0,character:0},end:{line:0,character:1}},message,data:{hickory:{source_hash:hash},provider:{hickory:{source_hash:digest}}}}]}})));
  try {
    send("captured report",digest);expect(hook.result.current.diagnostics[0].message).toBe("captured report");expect(hook.result.current.diagnosticSource).toBeUndefined();
    hook.rerender({text:"changed"});await act(async()=>finish(digest));expect(hook.result.current.diagnosticSource).toBeUndefined();
    proof.mockResolvedValue(digest);send("current engine report",digest);
    await act(async()=>{});expect(hook.result.current.diagnosticSource).toBe("changed");
    const mixed = [
      {range:{start:{line:0,character:0},end:{line:0,character:1}},message:"current contribution",data:{hickory:{source_hash:digest}}},
      {range:{start:{line:0,character:1},end:{line:0,character:2}},message:"unknown contribution",data:{hickory:{source_hash:null}}}
    ];
    act(()=>channel.handleFrame(encodeLspFrame({jsonrpc:"2.0",method:"textDocument/publishDiagnostics",params:{uri:"hick:///proof.py",diagnostics:mixed}})));
    await act(async()=>{});
    expect(revisionProof.diagnosticBoundSource(hook.result.current.diagnostics[0])).toBe("changed");
    expect(revisionProof.diagnosticBoundSource(hook.result.current.diagnostics[1])).toBeUndefined();
    expect(hook.result.current.diagnosticSource).toBeUndefined();
    // An unversioned/unknown engine envelope never borrows a matching window
    // counter or a lookalike hash inside opaque provider data.
    send("unknown report",null,1);await act(async()=>{});
    expect(hook.result.current.diagnostics[0].message).toBe("unknown report");expect(hook.result.current.diagnosticSource).toBeUndefined();
    hook.rerender({text:"later"});proof.mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
    send("slow valid report",digest);send("new unknown report",null);
    await act(async()=>finish(digest));expect(hook.result.current.diagnostics[0].message).toBe("new unknown report");expect(hook.result.current.diagnosticSource).toBeUndefined();
  } finally {hook.unmount();proof.mockRestore();}
});
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

it("binds diagnostic positions to published versions and preserves unversioned messages without positional proof", () => {
  // Contract: .hickory-source/frontends/contract.md
  const channel = createLspChannel({ send() {} });
  const realtime = { lsp: () => channel } as unknown as Realtime;
  const hook = renderHook(({ text }) => useLsp(realtime, "problem.py", text), { initialProps: { text: "value = 1" } });
  const publish = (version: number | undefined, message: string) => act(() => channel.handleFrame(encodeLspFrame({ jsonrpc: "2.0", method: "textDocument/publishDiagnostics", params: {
    uri: "hick:///problem.py", ...(version === undefined ? {} : { version }), diagnostics: [{ message, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } } }],
  } })));
  publish(1, "current"); expect(hook.result.current.diagnosticSource).toBe("value = 1");
  expect(revisionProof.diagnosticBoundSource(hook.result.current.diagnostics[0])).toBe("value = 1");
  hook.rerender({ text: "value = 2" }); expect(hook.result.current.diagnosticSource).not.toBe("value = 2");
  publish(99, "wrong version"); expect(hook.result.current.diagnostics[0].message).toBe("current");
  publish(undefined, "unknown source"); expect(hook.result.current.diagnostics[0].message).toBe("unknown source"); expect(hook.result.current.diagnosticSource).toBeUndefined();
  expect(revisionProof.diagnosticBoundSource(hook.result.current.diagnostics[0])).toBeUndefined();
  hook.unmount();
});
