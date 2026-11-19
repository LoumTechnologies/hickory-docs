import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { WsRealtime } from "./realtime";

// A socket that records what was asked of it and never connects on its own,
// so the lifecycle can be driven a step at a time.
class FakeSocket {
  static made: FakeSocket[] = [];
  readyState = 0; // CONNECTING
  binaryType = "";
  onopen: (() => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onclose: (() => void) | null = null;
  closed = false;

  constructor(readonly url: string) {
    FakeSocket.made.push(this);
  }
  send() {}
  close() {
    this.closed = true;
    this.readyState = 3; // CLOSED
    this.onclose?.();
  }
}

beforeEach(() => {
  FakeSocket.made = [];
  vi.stubGlobal(
    "WebSocket",
    Object.assign(FakeSocket, { CONNECTING: 0, OPEN: 1, CLOSING: 2, CLOSED: 3 }),
  );
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("a room that is closed and opened again", () => {
  it("opens a new socket, because the old one is shut for good", () => {
    // The bug this exists to stop: React StrictMode mounts an effect, cleans
    // it up, and mounts it again — a DEV build only, which is why an app that
    // had only ever run as a production bundle never saw it. The cleanup
    // closes the room; without a way back the second mount holds a dead
    // socket and the document never syncs.
    const realtime = new WsRealtime("doc:abc");
    expect(FakeSocket.made).toHaveLength(1);

    realtime.close();
    expect(FakeSocket.made[0].closed).toBe(true);

    realtime.reopen();
    expect(FakeSocket.made).toHaveLength(2);
    expect(FakeSocket.made[1].url).toContain("doc%3Aabc");
  });

  it("ignores a retired socket's delayed close after reopening", () => {
    const realtime = new WsRealtime("doc:abc");
    const first = FakeSocket.made[0];
    const delayedClose = first.onclose;
    first.onclose = null;
    realtime.close();
    realtime.reopen();
    delayedClose?.();
    vi.advanceTimersByTime(1600);
    expect(FakeSocket.made).toHaveLength(2);
    realtime.close();
    expect(FakeSocket.made[1].closed).toBe(true);
  });

  it("cancels an older connection's queued retry across a deliberate reopen", () => {
    const realtime = new WsRealtime("doc:abc");
    FakeSocket.made[0].close();
    realtime.close();
    realtime.reopen();
    vi.advanceTimersByTime(1600);
    expect(FakeSocket.made).toHaveLength(2);
  });

  it("does nothing to a room that was never closed", () => {
    const realtime = new WsRealtime("doc:abc");
    realtime.reopen();
    expect(FakeSocket.made).toHaveLength(1);
  });

  it("stops retrying a room the server refuses, and tries again when reopened", () => {
    // Refusals are what stop a rejected room being retried forever. A
    // deliberate reopen is a fresh judgement about that room, so it must not
    // inherit a verdict reached before anybody asked for this connection.
    const realtime = new WsRealtime("doc:abc");
    for (let i = 0; i < 3; i++) {
      FakeSocket.made[FakeSocket.made.length - 1].onclose?.();
      vi.advanceTimersByTime(1600);
    }
    const givenUp = FakeSocket.made.length;
    // Given up: another close starts no further attempt.
    FakeSocket.made[givenUp - 1].onclose?.();
    vi.advanceTimersByTime(1600);
    expect(FakeSocket.made).toHaveLength(givenUp);

    realtime.reopen();
    expect(FakeSocket.made).toHaveLength(givenUp + 1);
  });
});
