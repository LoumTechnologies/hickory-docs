<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/treeMenu.clipboard.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-249d5b90c8f3"></a>
## `apps/web/src/shell/treeMenu.clipboard.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-249d5b90c8f3-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-249d5b90c8f3-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/shell/treeMenu.clipboard.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/treeMenu.clipboard.test.ts"><storycode:paste select="#f-249d5b90c8f3-expression_statement"/></storycode:file>

</storycode:doc>
