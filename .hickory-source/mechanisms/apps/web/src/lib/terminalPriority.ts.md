<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/terminalPriority.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-018b4c1ce60e"></a>
## `apps/web/src/lib/terminalPriority.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-018b4c1ce60e-TerminalKind"></a>
### `TerminalKind`

Where a lineage connection to a file should terminate, given what is on
screen. One pure decision, in priority order:
1. open pane text — the file's actual text is visible; land on it.
2. its tab — the file is open but not the active tab; land ON the tab.
3. a visible tree row — a folder-tree pane shows the file's path; land
on that row's near edge, the same way a side tab takes a connection.
4. a divider port — nothing on screen names the file; the "open here"
button is the terminal of last resort.
A collapsed directory means the row is not visible, so a tree-pane file
whose row is folded away falls through to the port — the caller reports
only terminals that are actually on screen.

<storycode:copy id="f-018b4c1ce60e-TerminalKind">
// Where a lineage connection to a file should terminate, given what is on
// screen. One pure decision, in priority order:
//
//   1. open pane text — the file's actual text is visible; land on it.
//   2. its tab — the file is open but not the active tab; land ON the tab.
//   3. a visible tree row — a folder-tree pane shows the file's path; land
//      on that row's near edge, the same way a side tab takes a connection.
//   4. a divider port — nothing on screen names the file; the "open here"
//      button is the terminal of last resort.
//
// A collapsed directory means the row is not visible, so a tree-pane file
// whose row is folded away falls through to the port — the caller reports
// only terminals that are actually on screen.

export type TerminalKind = "text" | "tab" | "tree" | "port";

</storycode:copy>
<a id="f-018b4c1ce60e-AvailableTerminals"></a>
### `AvailableTerminals`

<storycode:copy id="f-018b4c1ce60e-AvailableTerminals">
export interface AvailableTerminals {
  /** An editor pane is showing the file's text. */
  text?: boolean;
  /** The file is open as a (possibly inactive) tab somewhere. */
  tab?: boolean;
  /** A folder-tree pane has a visible row for the file's path. */
  tree?: boolean;
  /** A divider "open here" port exists for the file. */
  port?: boolean;
}

</storycode:copy>
<a id="f-018b4c1ce60e-ORDER"></a>
### `ORDER`

<storycode:copy id="f-018b4c1ce60e-ORDER">
const ORDER: readonly TerminalKind[] = ["text", "tab", "tree", "port"];

</storycode:copy>
<a id="f-018b4c1ce60e-pickTerminal"></a>
### `pickTerminal`

The terminal a connection should use, or null when nothing can host it.

<storycode:copy id="f-018b4c1ce60e-pickTerminal">
/** The terminal a connection should use, or null when nothing can host it. */
export function pickTerminal(available: AvailableTerminals): TerminalKind | null {
  for (const kind of ORDER) if (available[kind]) return kind;
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/lib/terminalPriority.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/terminalPriority.ts"><storycode:paste select="#f-018b4c1ce60e-TerminalKind"/><storycode:paste select="#f-018b4c1ce60e-AvailableTerminals"/><storycode:paste select="#f-018b4c1ce60e-ORDER"/><storycode:paste select="#f-018b4c1ce60e-pickTerminal"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c87f1724da0b"></a>
## `apps/web/src/lib/terminalPriority.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c87f1724da0b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c87f1724da0b-expression_statement">
import { describe, expect, it } from "vitest";

import { pickTerminal } from "./terminalPriority";

describe("pickTerminal", () => {
  it("prefers visible pane text over everything", () => {
    expect(pickTerminal({ text: true, tab: true, tree: true, port: true })).toBe("text");
  });

  it("prefers the file's tab when its text is not on screen", () => {
    expect(pickTerminal({ tab: true, tree: true, port: true })).toBe("tab");
  });

  it("prefers a visible tree row over the divider port", () => {
    expect(pickTerminal({ tree: true, port: true })).toBe("tree");
  });

  it("falls through to the port when the row is not visible", () => {
    // A collapsed directory means the row is simply not rendered: the caller
    // reports tree: false, and the connection lands on the port.
    expect(pickTerminal({ tree: false, port: true })).toBe("port");
  });

  it("answers null when nothing on screen can host the connection", () => {
    expect(pickTerminal({})).toBeNull();
    expect(pickTerminal({ text: false, tab: false, tree: false, port: false })).toBeNull();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/terminalPriority.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/terminalPriority.test.ts"><storycode:paste select="#f-c87f1724da0b-expression_statement"/></storycode:file>


</storycode:doc>
