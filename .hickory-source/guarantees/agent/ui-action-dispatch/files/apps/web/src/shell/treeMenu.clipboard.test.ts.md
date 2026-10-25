<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/treeMenu.clipboard.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `09cfcc5fc262ee0aa5ea79f81a2a2a3419f06ec00d1cfb9456c81e6529a2588c`.

<book:entry path="apps/web/src/shell/treeMenu.clipboard.test.ts">
import { expect, it, vi } from "vitest";
import { copyText } from "./treeMenu";

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reports actual clipboard fallback refusal and removes its temporary input", async () => {
  const previous = Object.getOwnPropertyDescriptor(document, "execCommand");
  const clipboard = Object.getOwnPropertyDescriptor(navigator, "clipboard");
  const command = vi.fn().mockReturnValue(false);
  Object.defineProperty(document, "execCommand", { configurable: true, value: command });
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: vi.fn().mockRejectedValue(new Error("denied")) } });
  try {
    await expect(copyText("notes/a.md")).rejects.toThrow("Clipboard permission is unavailable");
    expect(command).toHaveBeenCalledWith("copy");
    expect(document.querySelector("textarea")).toBeNull();
    command.mockReturnValue(true);
    await expect(copyText("notes/a.md")).resolves.toBeUndefined();
    expect(document.querySelector("textarea")).toBeNull();
  } finally {
    if (previous) Object.defineProperty(document, "execCommand", previous); else Reflect.deleteProperty(document, "execCommand");
    if (clipboard) Object.defineProperty(navigator, "clipboard", clipboard); else Reflect.deleteProperty(navigator, "clipboard");
  }
});
</book:entry>

</book:doc>
