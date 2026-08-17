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

const ORDER: readonly TerminalKind[] = ["text", "tab", "tree", "port"];

/** The terminal a connection should use, or null when nothing can host it. */
export function pickTerminal(available: AvailableTerminals): TerminalKind | null {
  for (const kind of ORDER) if (available[kind]) return kind;
  return null;
}
