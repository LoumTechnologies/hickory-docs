// The frontend half of the element registry.
//
// The server declares each element once (`crates/hick-blocks`): its tag,
// its attributes, how it renders to a block and which actions it answers.
// This is the mirror: one view per block `kind`, keyed by the same string
// that comes over the wire, so adding an element is one Rust file and one
// folder here — never a new branch in the editor.
//
// A view draws a RENDERED block: the editor has folded the block's source
// lines away (see editor/rendered.ts) and mounted an element in their
// place; the view fills it. It gets the slot (where and what) and a context
// (what the document around it knows), and returns what to draw.

import type { ReactNode } from "react";
import type { EditorView } from "@codemirror/view";

import type { Block, DiagramBlock, ExecBlock, ExecutorInfo, SessionLink } from "../api/types";
import type { HickBlock } from "../editor/hickDoc";
import type { RenderedSlot } from "../editor/rendered";
import type { TableLayout } from "../components/TablePanel";

/** The kinds of block the editor renders in place of their source. */
export type SlotKind =
  | "exec"
  | "output"
  | "diagram"
  | "math"
  | "table"
  | "picture"
  | "session-user"
  | "session-assistant"
  | "session-tool"
  | "session-tool-result"
  | "session-read"
  | "session-wrote"
  | "session-context"
  | "session-observation"
  | "session-action"
  | "session-reasoning"
  | "session-input"
  | "session-meta";

/** What the document around a rendered block knows, handed to every view. */
export interface SlotContext {
  /** The live editor, when mounted. */
  view: EditorView | null;
  /** This document's path (for assets and per-table state), or null. */
  path: string | null;
  /** The server's rendered exec blocks — statuses and transcripts. */
  execBlocks: ExecBlock[];
  /** The server's rendered diagram blocks, pastes resolved. */
  diagramBlocks: DiagramBlock[];
  /** A session's blocks from the server, for the conversation's cards. */
  sessionBlocks?: Block[];
  /** The provenance each answer rests on, keyed by the answer's block
   * start: what the turn read, wrote and pointed at, as chips under it. */
  sessionLinksAt?: ReadonlyMap<number, readonly SessionLink[]>;
  runningCells: Set<string>;
  /** Block starts whose transcript is revealed (the rail's Replay). */
  replaying: readonly number[];
  executor?: ExecutorInfo;
  tableLayouts?: Record<string, TableLayout>;
  onTableLayout?: (key: string, layout: TableLayout) => void;
  /** Replace the slot's block content in the document; `origin` names the
   * user event for the undo history. */
  replaceBlockContent: (slot: RenderedSlot, text: string, origin?: string) => void;
}

export interface ElementView {
  kind: SlotKind;
  /** Whether this view draws the given structure block. */
  draws: (block: HickBlock) => boolean;
  /** Draw the rendered block. */
  render: (slot: RenderedSlot, cx: SlotContext) => ReactNode;
}
