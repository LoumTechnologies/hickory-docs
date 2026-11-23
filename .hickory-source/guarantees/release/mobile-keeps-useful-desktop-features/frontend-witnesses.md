<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Check draft ordering and native transport ownership

For maintainers reviewing failure recovery before device checks. Deferred writes make save races observable; a native channel stand-in exercises closing during setup without requiring a running Tauri application.

## Draft save races

Two snapshots submitted while the first save is pending must reach storage in order with successive accepted revisions. A failed write does not advance the revision, so a later save preserves the latest text without claiming that the failed write succeeded.

## Late room setup

Closing before native room setup returns must retire that exact room rather than reopen it. The adapter must not send bytes after that close, even if setup eventually succeeds.

Assemble `apps/web/src/mobile/bridge.test.ts` in the order explained above.

## Shared protocol reconnect

The mobile transport seam still sends the existing Yjs handshake and reacts to server state. A queued retry must not reconnect after its owner closed the room; this protects a newer editor from an obsolete native room.

<mobile:copy id="mobile-frontend-witnesses-shared-protocol-reconnect">
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
</mobile:copy>

Assemble `apps/web/src/mobile/transport.test.ts` in the order explained above.

<mobile:file path="apps/web/src/mobile/transport.test.ts"><mobile:paste select="#mobile-frontend-witnesses-shared-protocol-reconnect"/></mobile:file>

## Offline browser flow

This browser witness checks the narrow-screen local authoring flow, restart recovery, disabled desktop-only actions, and absence of horizontal page overflow. Native storage and peer sockets are exercised by separate host tests, not inferred from this preview.

Assemble `apps/web/e2e/mobile.spec.ts` in the order explained above.

</mobile:doc>
