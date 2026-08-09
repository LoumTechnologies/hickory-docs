// Two clients in one room, with no server between them.
//
// The landing page cannot open a WebSocket to a document a visitor does not
// own, so the two panes of the collaboration demo are two REAL Yjs clients —
// separate Y.Docs, separate awareness states — wired to each other in this
// tab. Convergence, presence and cursor mapping are therefore the CRDT's
// actual behaviour, not an animation of it; the only thing being faked is the
// network hop, which is replaced by a direct hand-off.

import * as Y from "yjs";
import {
  Awareness,
  applyAwarenessUpdate,
  encodeAwarenessUpdate,
} from "y-protocols/awareness";

/** Marks an update that arrived from the other client, so relays stop. */
const RELAY = "relay";

export interface Peer {
  id: string;
  name: string;
  color: string;
  ydoc: Y.Doc;
  ytext: Y.Text;
  awareness: Awareness;
}

export interface Room {
  peers: [Peer, Peer];
  /** Replace a range of the shared text, as a collaborator would. */
  edit(peer: Peer, from: number, to: number, insert: string): void;
  destroy(): void;
}

function makePeer(id: string, name: string, color: string): Peer {
  const ydoc = new Y.Doc();
  const ytext = ydoc.getText("source");
  const awareness = new Awareness(ydoc);
  return { id, name, color, ydoc, ytext, awareness };
}

export function createRoom(
  seed: string,
  a: { name: string; color: string },
  b: { name: string; color: string },
): Room {
  const first = makePeer("a", a.name, a.color);
  const second = makePeer("b", b.name, b.color);

  const link = (from: Peer, to: Peer) => {
    const onUpdate = (update: Uint8Array, origin: unknown) => {
      if (origin === RELAY) return;
      Y.applyUpdate(to.ydoc, update, RELAY);
    };
    from.ydoc.on("update", onUpdate);

    const onAwareness = (
      { added, updated, removed }: { added: number[]; updated: number[]; removed: number[] },
      origin: unknown,
    ) => {
      if (origin === RELAY) return;
      const changed = [...added, ...updated, ...removed];
      if (changed.length === 0) return;
      applyAwarenessUpdate(to.awareness, encodeAwarenessUpdate(from.awareness, changed), RELAY);
    };
    from.awareness.on("update", onAwareness);

    return () => {
      from.ydoc.off("update", onUpdate);
      from.awareness.off("update", onAwareness);
    };
  };

  const unlink = [link(first, second), link(second, first)];

  // Identity is published AFTER linking, for the same reason the seed is:
  // awareness has no back-fill here, so a state set before the relay exists
  // is a person the other client never learns about — two editors that
  // silently show no one else present.
  for (const peer of [first, second]) {
    peer.awareness.setLocalStateField("user", { name: peer.name, color: peer.color });
  }

  // Seed AFTER linking, and from ONE client only. Seeding before the link
  // would leave the second client empty forever; seeding from both would mint
  // two independent copies of the same text, which the CRDT merges by
  // concatenation — the document-doubling failure the workspace guards
  // against in docs/guarantees/collaboration/document-text-never-duplicates.md.
  first.ytext.insert(0, seed);

  return {
    peers: [first, second],
    edit(peer, fromIndex, toIndex, insert) {
      peer.ydoc.transact(() => {
        if (toIndex > fromIndex) peer.ytext.delete(fromIndex, toIndex - fromIndex);
        if (insert) peer.ytext.insert(fromIndex, insert);
      });
    },
    destroy() {
      for (const off of unlink) off();
      for (const peer of [first, second]) {
        peer.awareness.destroy();
        peer.ydoc.destroy();
      }
    },
  };
}
