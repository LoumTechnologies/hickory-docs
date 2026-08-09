import { describe, expect, it } from "vitest";

import { createRoom } from "./collabRoom";

const A = { name: "You", color: "#8f6f3f" };
const B = { name: "Priya", color: "#3f6f8f" };

describe("the collaboration demo's room", () => {
  // Protects docs/guarantees/collaboration/document-text-never-duplicates.md
  it("seeds exactly once, so both clients hold one copy of the document", () => {
    const room = createRoom("# Plan limits\n", A, B);
    try {
      expect(room.peers[0].ytext.toString()).toBe("# Plan limits\n");
      // The failure this guards against is the document doubling: two clients
      // that both seed produce "# Plan limits\n# Plan limits\n" after merge.
      expect(room.peers[1].ytext.toString()).toBe("# Plan limits\n");
    } finally {
      room.destroy();
    }
  });

  it("converges both ways", () => {
    const room = createRoom("one\n", A, B);
    try {
      const [a, b] = room.peers;
      room.edit(a, 0, 0, "zero\n");
      expect(b.ytext.toString()).toBe("zero\none\n");
      room.edit(b, b.ytext.length, b.ytext.length, "two\n");
      expect(a.ytext.toString()).toBe("zero\none\ntwo\n");
    } finally {
      room.destroy();
    }
  });

  it("carries each person's presence to the other client", () => {
    const room = createRoom("x", A, B);
    try {
      const [a, b] = room.peers;
      const seenByB = [...b.awareness.getStates().values()].map(
        (s) => (s as { user?: { name: string } }).user?.name,
      );
      expect(seenByB).toContain(A.name);
      const seenByA = [...a.awareness.getStates().values()].map(
        (s) => (s as { user?: { name: string } }).user?.name,
      );
      expect(seenByA).toContain(B.name);
    } finally {
      room.destroy();
    }
  });

  it("stops relaying once destroyed", () => {
    const room = createRoom("x", A, B);
    const [a, b] = room.peers;
    room.destroy();
    // Destroyed docs are inert; the point is that nothing throws and the
    // relay does not resurrect a torn-down page's editors.
    expect(() => room.edit(a, 0, 0, "y")).not.toThrow();
    expect(b.ytext.toString()).toBe("x");
  });
});
