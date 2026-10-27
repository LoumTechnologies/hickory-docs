import { afterEach, describe, expect, it, vi } from "vitest";
import { DraftWriter, NativeSocket, type Draft } from "./bridge";

const draft: Draft = { id: "one", title: "Notes", source: "first", revision: 0, remote: null, y_state: null };
afterEach(() => { delete (window as unknown as { __TAURI__?: unknown }).__TAURI__; });
describe("mobile recovery saves", () => {
  it("orders pending saves using only accepted revisions", async () => {
    let release!: () => void;
    const wait = new Promise<void>(resolve => { release = resolve; });
    const snapshots: Draft[] = [];
    const writer = new DraftWriter(draft, async value => { snapshots.push(value); if (value.revision === 0) await wait; return { ...value, revision: value.revision + 1 }; });
    const first = writer.save(draft);
    const second = writer.save({ ...draft, source: "latest 🦀" });
    await Promise.resolve(); await Promise.resolve();
    expect(snapshots).toHaveLength(1);
    release(); await first; await second;
    expect(snapshots.map(d => [d.source,d.revision])).toEqual([["first",0],["latest 🦀",1]]);
  });
  it("keeps a failed save visible and retries the latest text at the accepted revision", async () => {
    const write = vi.fn().mockRejectedValueOnce(Error("disk full")).mockImplementation(async (value: Draft) => ({ ...value, revision: value.revision + 1 }));
    const writer = new DraftWriter(draft, write);
    await expect(writer.save(draft)).rejects.toThrow("disk full");
    await expect(writer.flush()).rejects.toThrow("disk full");
    await expect(writer.save({ ...draft, source: "recover me" })).resolves.toMatchObject({ source: "recover me", revision: 1 });
    expect(write.mock.calls[1][0].revision).toBe(0);
  });
});
it("retires a room closed while native setup was pending", async () => {
  let release!: () => void;
  const setup = new Promise<void>(resolve => { release = resolve; });
  const invoke = vi.fn((command: string) => command === "mobile_room_open" ? setup : Promise.resolve());
  (window as unknown as { __TAURI__: unknown }).__TAURI__ = { core: { invoke, Channel: class { onmessage = () => {}; } } };
  const socket = new NativeSocket("ws://localhost/api/ws?doc=doc%3Aa", () => {});
  const opened = vi.fn(); socket.onopen = opened;
  socket.close(); socket.send(new Uint8Array([0,1]));
  release(); await Promise.resolve(); await Promise.resolve();
  expect(opened).not.toHaveBeenCalled();
  expect(invoke.mock.calls.filter(([command]) => command === "mobile_room_send")).toHaveLength(0);
  expect(invoke.mock.calls.filter(([command]) => command === "mobile_room_close").length).toBeGreaterThan(0);
});
