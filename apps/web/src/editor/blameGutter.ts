// Who last touched this line, in a column left of the line numbers.
//
// OFF BY DEFAULT, and that is a design decision rather than a caution. A blame
// column is a permanent 18-character indent on every line of every file, and
// it answers a question nobody asks most of the time. It earns its place when
// you are asking it — reviewing, bisecting, or working out whether a passage
// was written by a person or by the agent — and the rest of the time it is
// just less room for the code.
//
// It sits LEFT of the numbers on purpose. The numbers are the coordinate
// everything else in this app refers to — a stack trace, a ribbon, a
// collaborator saying "line 40" — so they stay against the text, where
// counting from them is easy. The annotation goes outside them.
//
// A run of lines from one commit is labelled ONCE, at its first line, and left
// blank below. Repeating the same name and date down forty rows is how a blame
// column becomes wallpaper: the eye stops reading it, and the boundaries —
// which are the actual information — disappear into the repetition.

import { RangeSet, StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension } from "@codemirror/state";
import { GutterMarker, gutter } from "@codemirror/view";
import { EditorView } from "@codemirror/view";

/** One line's authorship, as the server reports it. */
export interface LineBlame {
  line: number;
  commit: string;
  author: string;
  email: string;
  /** Unix seconds. Formatted here, in the reader's own locale. */
  time: number;
  summary: string;
  uncommitted: boolean;
}

/** Replace what the column knows. Empty turns it blank without unloading it. */
export const setBlame = StateEffect.define<readonly LineBlame[]>();

/** Show or hide the column. */
export const setBlameShown = StateEffect.define<boolean>();

interface BlameState {
  shown: boolean;
  /** By line number, 1-based. */
  byLine: Map<number, LineBlame>;
}

export const blameField = StateField.define<BlameState>({
  create: () => ({ shown: false, byLine: new Map() }),
  update(value, tr) {
    let next = value;
    for (const effect of tr.effects) {
      if (effect.is(setBlame)) {
        next = { ...next, byLine: new Map(effect.value.map((l) => [l.line, l])) };
      } else if (effect.is(setBlameShown)) {
        next = { ...next, shown: effect.value };
      }
    }
    return next;
  },
});

/** Whether the column is currently drawn. */
export function blameShown(state: EditorState): boolean {
  return state.field(blameField, false)?.shown ?? false;
}

/**
 * How a date reads in a blame column.
 *
 * Relative for anything recent, because "3 days ago" is what the question
 * usually means, and absolute past a year, because "412 days ago" is not
 * something anyone converts in their head.
 */
export function blameDate(seconds: number, now: number = Date.now()): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return "";
  const days = Math.floor((now - seconds * 1000) / 86_400_000);
  if (days < 0) return "just now";
  if (days === 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 30) return `${days}d ago`;
  if (days < 365) return `${Math.floor(days / 30)}mo ago`;
  return new Date(seconds * 1000).getFullYear().toString();
}

/** The author's first name (or the whole thing when there is only one word) —
 * a column this narrow cannot hold "Ada Lovelace <ada@example.com>". */
export function shortAuthor(author: string): string {
  const name = author.split("<")[0].trim();
  if (!name) return "unknown";
  const first = name.split(/\s+/)[0];
  return first.length > 12 ? `${first.slice(0, 11)}…` : first;
}

/** What one row shows: blank when this line continues the row above.
 *
 * `now` is threaded through rather than read inside, for the same reason
 * `blameDate` takes it: "3d ago" is a fact about a moment, and a test that
 * lets the real clock supply it passes all day and fails after seven in the
 * evening, which is how it was found. */
export function rowLabel(
  entry: LineBlame | undefined,
  previous: LineBlame | undefined,
  now: number = Date.now(),
): string {
  if (!entry) return "";
  // A run from one commit is labelled once, at its first line. Repeating it
  // down forty rows is how a blame column becomes wallpaper.
  const sameRun =
    previous !== undefined &&
    previous.commit === entry.commit &&
    previous.uncommitted === entry.uncommitted;
  if (sameRun) return "";
  if (entry.uncommitted) return "you — uncommitted";
  return `${shortAuthor(entry.author)} ${blameDate(entry.time, now)}`;
}

class BlameMarker extends GutterMarker {
  constructor(
    private readonly label: string,
    private readonly tip: string,
    private readonly uncommitted: boolean,
    /** The width-reserving spacer CodeMirror keeps at the top of a gutter.
     * It carries its own class so nothing counting real entries — a test, a
     * stylesheet — mistakes it for one. */
    private readonly spacer = false,
  ) {
    super();
  }

  eq(other: BlameMarker) {
    return other.label === this.label && other.tip === this.tip;
  }

  toDOM() {
    const el = document.createElement("span");
    el.className = this.spacer
      ? "cm-blame-spacer"
      : `cm-blame-entry${this.uncommitted ? " cm-blame-entry--mine" : ""}`;
    el.textContent = this.label;
    // `data-tip`, never `title`: every hover hint in this app is drawn by the
    // themed layer.
    if (this.tip) el.dataset.tip = this.tip;
    return el;
  }
}

/** The full hover: everything the column had no room for. */
export function blameTip(entry: LineBlame): string {
  if (entry.uncommitted) return "Not committed yet — your own work";
  const who = entry.email ? `${entry.author} <${entry.email}>` : entry.author;
  const when = entry.time > 0 ? new Date(entry.time * 1000).toLocaleString() : "";
  return [entry.summary, `${who}${when ? ` — ${when}` : ""}`, entry.commit]
    .filter(Boolean)
    .join("\n");
}

/**
 * The blame column.
 *
 * Placed before `lineNumbers()` in an editor's extension list, which is what
 * puts it to their left — CodeMirror lays gutters out in the order they are
 * declared.
 */
export function blameGutter(): Extension {
  return [
    blameField,
    gutter({
      class: "cm-blame-gutter",
      markers: (view) => {
        const state = view.state.field(blameField, false);
        if (!state?.shown || state.byLine.size === 0) return RangeSet.empty;
        const doc = view.state.doc;
        const marks: { from: number; value: BlameMarker }[] = [];
        for (let n = 1; n <= doc.lines; n++) {
          const entry = state.byLine.get(n);
          if (!entry) continue;
          const label = rowLabel(entry, state.byLine.get(n - 1));
          marks.push({
            from: doc.line(n).from,
            value: new BlameMarker(label, blameTip(entry), entry.uncommitted),
          });
        }
        return RangeSet.of(
          marks.map((m) => m.value.range(m.from)),
          true,
        );
      },
      // Nothing is drawn at all while the column is off, so an editor that
      // never turns it on pays for an empty facet and no DOM.
      initialSpacer: () => new BlameMarker("", "", false, true),
    }),
    EditorView.baseTheme({
      ".cm-blame-gutter": { minWidth: "0" },
    }),
  ];
}
