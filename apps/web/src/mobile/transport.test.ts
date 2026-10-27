import { afterEach, expect, it, vi } from "vitest";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { WsRealtime, type RealtimeSocket } from "../api/realtime";

class Socket implements RealtimeSocket {
  readyState = 0; binaryType = "";
  onopen: RealtimeSocket["onopen"] = null;
  onmessage: RealtimeSocket["onmessage"] = null;
  onclose: RealtimeSocket["onclose"] = null;
  send = vi.fn();
  close() { this.readyState = 3; this.onclose?.(new CloseEvent("close")); }
}
afterEach(() => vi.useRealTimers());
it("uses the supplied native transport and cancels retries after close", () => {
  vi.useFakeTimers();
  const sockets: Socket[] = [];
  const factory = vi.fn(() => { const socket = new Socket(); sockets.push(socket); return socket; });
  const realtime = new WsRealtime("doc:a", factory);
  const doc = new Y.Doc(); const awareness = new Awareness(doc);
  realtime.bindDoc(doc, awareness);
  const first = sockets[0]; first.readyState = 1; first.onopen?.(new Event("open"));
  expect(first.send).toHaveBeenCalled();
  expect((first.send.mock.calls[0] as unknown as [Uint8Array])[0][0]).toBe(0);
  first.close(); realtime.close(); vi.advanceTimersByTime(2000);
  expect(factory).toHaveBeenCalledTimes(1);
  awareness.destroy(); doc.destroy();
});
