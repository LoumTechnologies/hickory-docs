// Guarantee: docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md
// — a session hook re-runs on every keystroke; the registry must not tell the
// workspace about a publish whose every field is the one it already holds.
import { describe, expect, it, vi } from "vitest";
import { SessionRegistry, type DocSession } from "./documentSession";

const session = (over: Partial<DocSession> = {}): DocSession =>
  ({ docId: "d1", doc: null, syncState: "idle", ...over }) as unknown as DocSession;

describe("the session registry", () => {
  it("emits on the first publish and on a real change", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    registry.publish(session());
    expect(heard).toHaveBeenCalledTimes(1);
    registry.publish(session({ syncState: "editing" }));
    expect(heard).toHaveBeenCalledTimes(2);
    expect(registry.get("d1")?.syncState).toBe("editing");
  });

  it("stays silent when a fresh object carries the same fields", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    const first = session();
    registry.publish(first);
    const version = registry.version;
    registry.publish(session()); // a new object, the same session
    expect(heard).toHaveBeenCalledTimes(1);
    expect(registry.version).toBe(version);
    // And the held object is the first one: a subscriber reading the snapshot
    // sees a stable identity, which is what lets it skip its own render.
    expect(registry.get("d1")).toBe(first);
  });

  it("notices a field added or removed, not only one replaced", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    registry.publish(session());
    registry.publish(session({ banner: { kind: "pass", text: "ok" } }));
    expect(heard).toHaveBeenCalledTimes(2);
  });
});
