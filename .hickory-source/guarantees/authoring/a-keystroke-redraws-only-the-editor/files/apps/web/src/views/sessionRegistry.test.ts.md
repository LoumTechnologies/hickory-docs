<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/sessionRegistry.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Keystroke Redraws Only The Editor](../../../../../contract.md)

Captured SHA-256: `6b2c3522860154d4d6e606ff74d5e8f47c0c3b89a61a194a10b8a4048a86a911`.

<book:entry path="apps/web/src/views/sessionRegistry.test.ts">
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
</book:entry>

</book:doc>
