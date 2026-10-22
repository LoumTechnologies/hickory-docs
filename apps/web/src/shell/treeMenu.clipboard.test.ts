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
