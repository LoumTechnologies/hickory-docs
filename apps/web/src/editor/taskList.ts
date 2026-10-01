// Markdown task lists, made clickable.
//
// The styling half lives in markdownStyling.ts, which is decoration-only by
// charter — it tints the line and dims the `[ ]`, and never touches text.
// This module is the other half, and it is a different kind of thing: it
// REPLACES the three-character box with a checkbox, and a click on that
// checkbox edits the document.
//
// Two rules keep the replacement honest:
//
//  - It is an INLINE replacement of exactly the characters it stands for, so
//    no screen row appears that has no document line behind it and the
//    gutters keep counting truthfully. See
//    docs/guarantees/authoring/the-gutters-never-skip-a-number.md.
//  - The box on the line holding the cursor is NOT replaced. A person who
//    put the caret there is editing the text, and text you cannot see is text
//    you cannot fix — the same reason a fold opens when you step into it.
//
// Where the tasks come from is the caller's business: a plain `.md` buffer
// scans itself, while a `.md` document must use its own structure parse so
// a `- [ ]` inside an exec cell's payload stays a command and not a checkbox.

import { StateField } from "@codemirror/state";
import type { EditorState, Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import type { TaskItem } from "./hickDoc";
import { parseMarkdownProse } from "./markdownStyling";

/** Where a task list's items come from, given the buffer being shown. */
export type TaskSource = (state: EditorState) => readonly TaskItem[];

/** The default source: scan the buffer as plain markdown. */
export const markdownTasks: TaskSource = (state) =>
  parseMarkdownProse(state.doc.toString()).tasks;

/**
 * The edit that flips the box of the task at `pos`, or null when `pos` is not
 * on a task line.
 *
 * Pure and text-in/change-out so the toggle can be tested without a view —
 * and so the click handler never has to reason about which of several boxes
 * on screen it hit. It rescans rather than trusting an offset captured when
 * the widget was built: a widget survives edits above it, and an offset from
 * two seconds ago can point into the middle of a word.
 */
export function toggleTaskAt(
  text: string,
  pos: number,
  tasks: readonly TaskItem[] = parseMarkdownProse(text).tasks,
): { from: number; to: number; insert: string } | null {
  const task = tasks.find((t) => pos >= t.from && pos <= t.to);
  if (!task) return null;
  // Only the middle character changes, so a `[X]` written in capitals comes
  // back as `[ ]` and an unchecked box becomes the lowercase `[x]` that every
  // markdown renderer agrees about.
  return {
    from: task.boxFrom + 1,
    to: task.boxFrom + 2,
    insert: task.checked ? " " : "x",
  };
}

/** The lines any cursor or selection currently touches. */
function activeLines(state: EditorState): Set<number> {
  const lines = new Set<number>();
  for (const range of state.selection.ranges) {
    const first = state.doc.lineAt(range.from).number;
    const last = state.doc.lineAt(range.to).number;
    for (let n = first; n <= last; n++) lines.add(n);
  }
  return lines;
}

class CheckboxWidget extends WidgetType {
  constructor(private readonly checked: boolean) {
    super();
  }

  // Position is deliberately not part of identity: typing prose above a task
  // moves every box below it, and rebuilding all of them would throw away the
  // DOM (and any focus ring) for a change that alters nothing visible.
  eq(other: CheckboxWidget) {
    return other.checked === this.checked;
  }

  toDOM(view: EditorView) {
    const box = document.createElement("span");
    box.className = `cm-md-checkbox${this.checked ? " cm-md-checkbox--on" : ""}`;
    box.setAttribute("role", "checkbox");
    box.setAttribute("aria-checked", String(this.checked));
    box.setAttribute("aria-label", this.checked ? "Done" : "Not done");
    // The glyph is text rather than a background image so it inherits the
    // editor's colour, scales with the zoom level, and survives a theme swap.
    box.textContent = this.checked ? "✓" : "";
    // mousedown, not click: by the time a click fires CodeMirror has already
    // moved the selection, which would reveal the source and take the box out
    // from under the pointer mid-gesture.
    box.addEventListener("mousedown", (event) => {
      event.preventDefault();
      const change = toggleTaskAt(view.state.doc.toString(), view.posAtDOM(box));
      if (change) view.dispatch({ changes: change, userEvent: "input.toggleTask" });
    });
    return box;
  }

  ignoreEvent() {
    // The box handles its own press; the editor must not also read it as a
    // click into text it is not showing.
    return true;
  }
}

function buildCheckboxes(state: EditorState, source: TaskSource): DecorationSet {
  const tasks = source(state);
  if (tasks.length === 0) return Decoration.none;
  const live = activeLines(state);
  const ranges: Range<Decoration>[] = [];
  for (const task of tasks) {
    if (live.has(state.doc.lineAt(task.from).number)) continue;
    if (task.boxTo > task.boxFrom && task.boxTo <= state.doc.length) {
      ranges.push(
        Decoration.replace({ widget: new CheckboxWidget(task.checked) }).range(
          task.boxFrom,
          task.boxTo,
        ),
      );
    }
  }
  return Decoration.set(ranges, true);
}

/**
 * The clickable-checkbox extension.
 *
 * A StateField rather than a ViewPlugin because the decoration is a
 * replacement, and only field-provided decorations are folded into the same
 * update CodeMirror measures — a plugin producing these would leave the
 * height map describing a document that no longer exists.
 */
export function taskCheckboxes(source: TaskSource = markdownTasks): Extension {
  return StateField.define<DecorationSet>({
    create: (state) => buildCheckboxes(state, source),
    update: (value, tr) =>
      // Selection matters as much as content here: moving the caret onto a
      // task line is what reveals its source.
      tr.docChanged || tr.selection ? buildCheckboxes(tr.state, source) : value,
    provide: (f) => EditorView.decorations.from(f),
  });
}
