<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Blame Column Is Optional And Honest

For: Authors editing generated text and reviewers tracing a result to its source.

Given any editor in the app, when the blame column is off — which it is by
default — then nothing is asked of git at all; and when it is turned on, then
every line shows who last committed it, uncommitted lines say so rather than
borrowing an author, and a folder with no history shows an empty column rather
than failing to open a file.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/the-blame-column-is-optional-and-honest.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-3d8799779e43"></a>
## `apps/web/src/editor/blameGutter.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Authorship display accepts non-Git providers](../authorship-display-accepts-non-git-providers/contract.md), [The Blame Column Is Optional And Honest](contract.md).

<a id="f-3d8799779e43-NOW"></a>
### `NOW`

<storycode:copy id="f-3d8799779e43-NOW">
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";

import {
  blameDate,
  blameGutter,
  blameShown,
  blameTip,
  rowLabel,
  setBlame,
  setBlameShown,
  shortAuthor,
  type LineBlame,
} from "./blameGutter";

const NOW = Date.UTC(2026, 7, 21) ;
</storycode:copy>
<a id="f-3d8799779e43-at"></a>
### `at`

<storycode:copy id="f-3d8799779e43-at">
const at = (daysAgo: number) => Math.floor((NOW - daysAgo * 86_400_000) / 1000);

</storycode:copy>
<a id="f-3d8799779e43-entry"></a>
### `entry`

<storycode:copy id="f-3d8799779e43-entry">
const entry = (over: Partial<LineBlame> = {}): LineBlame => ({
  line: 1,
  commit: "abc1234d",
  author: "Ada Lovelace",
  email: "ada@example.com",
  time: at(3),
  summary: "First three lines",
  uncommitted: false,
  ...over,
});

</storycode:copy>
<a id="f-3d8799779e43-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3d8799779e43-expression_statement">
describe("how a blame column reads a date", () => {
  it("is relative while relative is what the question means", () => {
    expect(blameDate(at(0), NOW)).toBe("today");
    expect(blameDate(at(1), NOW)).toBe("yesterday");
    expect(blameDate(at(9), NOW)).toBe("9d ago");
    expect(blameDate(at(70), NOW)).toBe("2mo ago");
  });

  it("becomes a year past that, because nobody converts '412 days ago'", () => {
    expect(blameDate(at(500), NOW)).toBe("2025");
  });

  it("says nothing rather than 1970 for a missing time", () => {
    expect(blameDate(0, NOW)).toBe("");
    expect(blameDate(Number.NaN, NOW)).toBe("");
  });
});

</storycode:copy>
<a id="f-3d8799779e43-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3d8799779e43-expression_statement-2">
describe("fitting an author into the column", () => {
  it("keeps the first name", () => {
    expect(shortAuthor("Ada Lovelace")).toBe("Ada");
    expect(shortAuthor("Ada Lovelace <ada@example.com>")).toBe("Ada");
    expect(shortAuthor("cher")).toBe("cher");
  });

  it("truncates a name no column could hold", () => {
    expect(shortAuthor("Bartholomewwwwwwww Smith")).toBe("Bartholomew…");
  });

  it("says unknown rather than showing an empty cell", () => {
    expect(shortAuthor("")).toBe("unknown");
    expect(shortAuthor("  <only@email>")).toBe("unknown");
  });
});

</storycode:copy>
<a id="f-3d8799779e43-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-3d8799779e43-expression_statement-3">
describe("labelling a run of lines", () => {
  it("reads its date from the moment it is given, not from the wall clock", () => {
    // The defect this replaced: `rowLabel` called `blameDate` without a
    // `now`, so these tests measured against the real clock. They passed all
    // day and failed after seven in the evening, when the fixture's three
    // days became four in UTC.
    expect(rowLabel(entry(), undefined, NOW)).toBe("Ada 3d ago");
    expect(rowLabel(entry(), undefined, NOW + 86_400_000)).toBe("Ada 4d ago");
  });

  it("labels the first line of a run", () => {
    expect(rowLabel(entry(), undefined, NOW)).toBe("Ada 3d ago");
  });

  it("leaves the rest of the run blank", () => {
    // Repeating the same name down forty rows is how a blame column becomes
    // wallpaper: the eye stops reading it, and the boundaries — which are the
    // actual information — disappear into the repetition.
    expect(rowLabel(entry({ line: 2 }), entry())).toBe("");
  });

  it("labels again where the commit changes", () => {
    const later = entry({ line: 2, commit: "ffff0000", author: "Grace Hopper" });
    expect(rowLabel(later, entry(), NOW)).toBe("Grace 3d ago");
  });

  it("marks uncommitted work as yours", () => {
    // The working tree is the common case. Attributing a line somebody just
    // typed to whoever last touched the file is a lie in the direction that
    // matters.
    expect(rowLabel(entry({ uncommitted: true, commit: "" }), undefined)).toBe(
      "you — uncommitted",
    );
  });

  it("does not run an uncommitted line into a committed one above it", () => {
    const mine = entry({ line: 2, uncommitted: true, commit: "" });
    expect(rowLabel(mine, entry())).toBe("you — uncommitted");
  });

  it("shows nothing for a line git said nothing about", () => {
    expect(rowLabel(undefined, entry())).toBe("");
  });
});

</storycode:copy>
<a id="f-3d8799779e43-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-3d8799779e43-expression_statement-4">
describe("the hover, which holds what the column could not", () => {
  it("gives the summary, the full identity, and the sha", () => {
    const tip = blameTip(entry());
    expect(tip).toContain("First three lines");
    expect(tip).toContain("Ada Lovelace <ada@example.com>");
    expect(tip).toContain("abc1234d");
  });

  it("says plainly when the line is not committed", () => {
    expect(blameTip(entry({ uncommitted: true }))).toMatch(/not committed yet/i);
  });
});

</storycode:copy>
<a id="f-3d8799779e43-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-3d8799779e43-expression_statement-5">
describe("the column in an editor", () => {
  const mount = () =>
    new EditorView({
      state: EditorState.create({
        doc: "one\ntwo\nthree\n",
        extensions: [blameGutter(), lineNumbers()],
      }),
      parent: document.body,
    });

  it("draws nothing until it is turned on", () => {
    // Off by default: a blame column is a permanent indent on every line of
    // every file, answering a question nobody asks most of the time.
    const view = mount();
    view.dispatch({ effects: setBlame.of([entry()]) });
    expect(blameShown(view.state)).toBe(false);
    expect(view.dom.querySelectorAll(".cm-blame-entry").length).toBe(0);
    view.destroy();
  });

  it("draws a row per blamed line once it is on", () => {
    const view = mount();
    view.dispatch({
      effects: [
        setBlameShown.of(true),
        setBlame.of([entry(), entry({ line: 2 }), entry({ line: 3, commit: "zzz" })]),
      ],
    });
    expect(blameShown(view.state)).toBe(true);
    const cells = [...view.dom.querySelectorAll(".cm-blame-entry")];
    expect(cells.length).toBeGreaterThanOrEqual(3);
    view.destroy();
  });

  it("goes away again when it is turned off", () => {
    const view = mount();
    view.dispatch({ effects: [setBlameShown.of(true), setBlame.of([entry()])] });
    view.dispatch({ effects: setBlameShown.of(false) });
    expect(view.dom.querySelectorAll(".cm-blame-entry").length).toBe(0);
    view.destroy();
  });

  it("uses data-tip, never a native title", () => {
    const view = mount();
    view.dispatch({ effects: [setBlameShown.of(true), setBlame.of([entry()])] });
    const cell = view.dom.querySelector(".cm-blame-entry") as HTMLElement;
    expect(cell.dataset.tip).toContain("First three lines");
    expect(cell.getAttribute("title")).toBeNull();
    view.destroy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/blameGutter.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/blameGutter.test.ts"><storycode:paste select="#f-3d8799779e43-NOW"/><storycode:paste select="#f-3d8799779e43-at"/><storycode:paste select="#f-3d8799779e43-entry"/><storycode:paste select="#f-3d8799779e43-expression_statement"/><storycode:paste select="#f-3d8799779e43-expression_statement-2"/><storycode:paste select="#f-3d8799779e43-expression_statement-3"/><storycode:paste select="#f-3d8799779e43-expression_statement-4"/><storycode:paste select="#f-3d8799779e43-expression_statement-5"/></storycode:file>

<a id="output-cd7f4a46ce4a"></a>
## `apps/web/src/editor/blameGutter.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Authorship display accepts non-Git providers](../authorship-display-accepts-non-git-providers/contract.md), [The Blame Column Is Optional And Honest](contract.md).

<a id="f-cd7f4a46ce4a-LineBlame"></a>
### `LineBlame`

Who last touched this line, in a column left of the line numbers.
OFF BY DEFAULT, and that is a design decision rather than a caution. A blame
column is a permanent 18-character indent on every line of every file, and
it answers a question nobody asks most of the time. It earns its place when
you are asking it — reviewing, bisecting, or working out whether a passage
was written by a person or by the agent — and the rest of the time it is
just less room for the code.
It sits LEFT of the numbers on purpose. The numbers are the coordinate
everything else in this app refers to — a stack trace, a ribbon, a
collaborator saying "line 40" — so they stay against the text, where
counting from them is easy. The annotation goes outside them.
A run of lines from one commit is labelled ONCE, at its first line, and left
blank below. Repeating the same name and date down forty rows is how a blame
column becomes wallpaper: the eye stops reading it, and the boundaries —
which are the actual information — disappear into the repetition.

<storycode:copy id="f-cd7f4a46ce4a-LineBlame">
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
import { EditorView, Decoration, type DecorationSet } from "@codemirror/view";
import { editorLanguageActions } from "./languageActions";
import { authorColor, type TextOrigin } from "../lib/attribution";

/** One line's authorship, as the server reports it. */
export interface LineBlame {
  origin?: TextOrigin;
  provider?: string;
  line: number;
  commit: string;
  author: string;
  email: string;
  /** Unix seconds. Formatted here, in the reader's own locale. */
  time: number;
  summary: string;
  uncommitted: boolean;
}

</storycode:copy>
<a id="f-cd7f4a46ce4a-setBlame"></a>
### `setBlame`

Replace what the column knows. Empty turns it blank without unloading it.

<storycode:copy id="f-cd7f4a46ce4a-setBlame">
/** Replace what the column knows. Empty turns it blank without unloading it. */
export const setBlame = StateEffect.define<readonly LineBlame[]>();

</storycode:copy>
<a id="f-cd7f4a46ce4a-setAuthorshipShown"></a>
### `setAuthorshipShown`

<storycode:copy id="f-cd7f4a46ce4a-setAuthorshipShown">
export const setAuthorshipShown = StateEffect.define<boolean>();

</storycode:copy>
<a id="f-cd7f4a46ce4a-setBlameShown"></a>
### `setBlameShown`

<storycode:copy id="f-cd7f4a46ce4a-setBlameShown">
/** Show or hide the column. */
export const setBlameShown = StateEffect.define<boolean>();

</storycode:copy>
<a id="f-cd7f4a46ce4a-BlameState"></a>
### `BlameState`

<storycode:copy id="f-cd7f4a46ce4a-BlameState">
interface BlameState {
  shown: boolean;
  /** By line number, 1-based. */
  byLine: Map<number, LineBlame>;
}

</storycode:copy>
<a id="f-cd7f4a46ce4a-blameField"></a>
### `blameField`

<storycode:copy id="f-cd7f4a46ce4a-blameField">
export const blameField = StateField.define<BlameState>({
  create: () => ({ shown: false, byLine: new Map() }),
  update(value, tr) {
    let next = tr.docChanged ? { ...value, byLine: new Map<number, LineBlame>() } : value;
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

</storycode:copy>
<a id="f-cd7f4a46ce4a-blameShown"></a>
### `blameShown`

<storycode:copy id="f-cd7f4a46ce4a-blameShown">
/** Whether the column is currently drawn. */
export function blameShown(state: EditorState): boolean {
  return state.field(blameField, false)?.shown ?? false;
}

</storycode:copy>
<a id="f-cd7f4a46ce4a-blameDate"></a>
### `blameDate`

How a date reads in a blame column.
Relative for anything recent, because "3 days ago" is what the question
usually means, and absolute past a year, because "412 days ago" is not
something anyone converts in their head.

<storycode:copy id="f-cd7f4a46ce4a-blameDate">
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

</storycode:copy>
<a id="f-cd7f4a46ce4a-shortAuthor"></a>
### `shortAuthor`

The author's first name (or the whole thing when there is only one word) —
a column this narrow cannot hold "Ada Lovelace <ada@example.com>".

<storycode:copy id="f-cd7f4a46ce4a-shortAuthor">
/** The author's first name (or the whole thing when there is only one word) —
 * a column this narrow cannot hold "Ada Lovelace <ada@example.com>". */
export function shortAuthor(author: string): string {
  const name = author.split("<")[0].trim();
  if (!name) return "unknown";
  const first = name.split(/\s+/)[0];
  return first.length > 12 ? `${first.slice(0, 11)}…` : first;
}

</storycode:copy>
<a id="f-cd7f4a46ce4a-rowLabel"></a>
### `rowLabel`

What one row shows: blank when this line continues the row above.
`now` is threaded through rather than read inside, for the same reason
`blameDate` takes it: "3d ago" is a fact about a moment, and a test that
lets the real clock supply it passes all day and fails after seven in the
evening, which is how it was found.

<storycode:copy id="f-cd7f4a46ce4a-rowLabel">
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

</storycode:copy>
<a id="f-cd7f4a46ce4a-BlameMarker"></a>
### `BlameMarker`

<storycode:copy id="f-cd7f4a46ce4a-BlameMarker">
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

</storycode:copy>
<a id="f-cd7f4a46ce4a-blameTip"></a>
### `blameTip`

The full hover: everything the column had no room for.

<storycode:copy id="f-cd7f4a46ce4a-blameTip">
/** The full hover: everything the column had no room for. */
export function blameTip(entry: LineBlame): string {
  if (entry.uncommitted) return "Not committed yet — your own work";
  const who = entry.email ? `${entry.author} <${entry.email}>` : entry.author;
  const when = entry.time > 0 ? new Date(entry.time * 1000).toLocaleString() : "";
  return [entry.summary, `${who}${when ? ` — ${when}` : ""}`, entry.commit]
    .filter(Boolean)
    .join("\n");
}

</storycode:copy>
<a id="f-cd7f4a46ce4a-blameGutter"></a>
### `blameGutter`

The blame column.
Placed before `lineNumbers()` in an editor's extension list, which is what
puts it to their left — CodeMirror lays gutters out in the order they are
declared.

<storycode:copy id="f-cd7f4a46ce4a-blameGutter">
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
    authorshipField,
    attributionActions(),
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
      ".cm-authorship": { borderLeft: "3px solid var(--author-color)", backgroundColor: "color-mix(in srgb, var(--author-color) 9%, transparent)" },
      ".cm-author-0": { "--author-color": "var(--ribbon-0)" },
      ".cm-author-1": { "--author-color": "var(--ribbon-1)" },
      ".cm-author-2": { "--author-color": "var(--ribbon-2)" },
      ".cm-author-3": { "--author-color": "var(--ribbon-3)" },
      ".cm-author-4": { "--author-color": "var(--ribbon-4)" },
      ".cm-author-5": { "--author-color": "var(--ribbon-5)" },
      ".cm-origin-agent": { "--author-color": "var(--tok-function)", borderLeftStyle: "dashed" },
      ".cm-origin-human": { "--author-color": "var(--accent-2)" },
      ".cm-origin-imported": { borderLeftStyle: "dotted" },
    }),
  ];
}


</storycode:copy>
<a id="f-cd7f4a46ce4a-AuthorshipState"></a>
### `AuthorshipState`

<storycode:copy id="f-cd7f4a46ce4a-AuthorshipState">
interface AuthorshipState { shown: boolean; lines: readonly LineBlame[]; marks: DecorationSet }
</storycode:copy>
<a id="f-cd7f4a46ce4a-authorshipField"></a>
### `authorshipField`

Unsaved edits invalidate attribution rather than inheriting the old line identity.

<storycode:copy id="f-cd7f4a46ce4a-authorshipField">
/** Unsaved edits invalidate attribution rather than inheriting the old line identity. */
export const authorshipField = StateField.define<AuthorshipState>({
  create: () => ({ shown: false, lines: [], marks: Decoration.none }),
  update(value, tr) {
    let { shown, lines } = value;
    if (tr.docChanged) lines = [];
    for (const effect of tr.effects) {
      if (effect.is(setBlame)) lines = effect.value;
      if (effect.is(setAuthorshipShown)) shown = effect.value;
    }
    const marks = shown ? lines.filter(line => line.line > 0 && line.line <= tr.newDoc.lines).map(line => {
      const origin = line.origin ?? "unknown";
      return Decoration.line({ attributes: {
        class: `cm-authorship cm-author-${authorColor(line.email || line.author)} cm-origin-${origin}`,
        "data-tip": `${line.provider ?? "Git blame"}: ${line.author || "unknown author"}${line.uncommitted ? " · uncommitted" : ""} · text origin: ${origin}`,
      } }).range(tr.newDoc.line(line.line).from);
    }) : [];
    return { shown, lines, marks: Decoration.set(marks, true) };
  },
  provide: field => EditorView.decorations.from(field, value => value.marks),
});


</storycode:copy>
<a id="f-cd7f4a46ce4a-attributionActions"></a>
### `attributionActions`

Read only the attribution already installed in the displayed editor layers.

<storycode:copy id="f-cd7f4a46ce4a-attributionActions">
/** Read only the attribution already installed in the displayed editor layers. */
function attributionActions(): Extension {
  const active = (view: EditorView) => view.state.field(blameField).shown || view.state.field(authorshipField).shown;
  const at = (view: EditorView, line: number) => {
    const blame = view.state.field(blameField), colors = view.state.field(authorshipField);
    return { gutter: blame.shown ? blame.byLine.get(line) : undefined, color: colors.shown ? colors.lines.find(entry => entry.line === line) : undefined };
  };
  const integer = { type: "number" as const, integer: true, minimum: 1, maximum: 100_000_000 };
  return [editorLanguageActions.of({ id: "attribution:page", label: "Read displayed attribution line page", visible: active, enabled: () => true,
    parameters: { first_line: integer, count: { ...integer, maximum: 64 } },
    execute: async (view, args) => {
      const first = Number(args.first_line), count = Number(args.count);
      if (first > view.state.doc.lines) return { status: "unavailable", reason: "Line is outside the current editor" };
      const last = Math.min(view.state.doc.lines, first + count - 1);
      const lines = Array.from({ length: last - first + 1 }, (_, index) => {
        const line = first + index, { gutter, color } = at(view, line);
        return { line, gutter_present: !!gutter, gutter_label: gutter ? rowLabel(gutter, view.state.field(blameField).byLine.get(line - 1)) : null, authorship_present: !!color, author_color: color ? authorColor(color.email || color.author) : null, origin: color?.origin ?? null };
      });
      const value = { lines, total_lines: view.state.doc.lines, next: last < view.state.doc.lines ? last + 1 : null, source_read: false };
      if (new TextEncoder().encode(JSON.stringify(value)).length > 8192) return { status: "unavailable", reason: "Attribution page exceeds 8192 UTF-8 bytes; request fewer lines" };
      return { status: "completed", value };
    } }), editorLanguageActions.of({ id: "attribution:detail", label: "Read displayed attribution detail bytes", visible: active, enabled: () => true,
    parameters: { line: integer, offset: { ...integer, minimum: 0 }, limit: { ...integer, maximum: 8192 } },
    execute: async (view, args) => {
      const line = Number(args.line);
      if (line > view.state.doc.lines) return { status: "unavailable", reason: "Line is outside the current editor" };
      const { gutter, color } = at(view, line);
      if (!gutter && !color) return { status: "unavailable", reason: "No attribution is displayed on this line" };
      // Copy known metadata only: provider records may also contain full source text.
      const record = (entry: LineBlame) => ({ author: entry.author, email: entry.email, commit: entry.commit, time: entry.time, summary: entry.summary, uncommitted: entry.uncommitted, origin: entry.origin ?? "unknown", provider: entry.provider ?? "Git blame" });
      const bytes = new TextEncoder().encode(JSON.stringify({ line, gutter: gutter ? { ...record(gutter), tooltip: blameTip(gutter) } : null, authorship: color ? record(color) : null, origin_is_not_authenticated_identity: true }));
      const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current attribution details" };
      return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "JSON UTF-8", span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length } };
    } })];
}
</storycode:copy>

### Assemble `apps/web/src/editor/blameGutter.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/blameGutter.ts"><storycode:paste select="#f-cd7f4a46ce4a-LineBlame"/><storycode:paste select="#f-cd7f4a46ce4a-setBlame"/><storycode:paste select="#f-cd7f4a46ce4a-setAuthorshipShown"/><storycode:paste select="#f-cd7f4a46ce4a-setBlameShown"/><storycode:paste select="#f-cd7f4a46ce4a-BlameState"/><storycode:paste select="#f-cd7f4a46ce4a-blameField"/><storycode:paste select="#f-cd7f4a46ce4a-blameShown"/><storycode:paste select="#f-cd7f4a46ce4a-blameDate"/><storycode:paste select="#f-cd7f4a46ce4a-shortAuthor"/><storycode:paste select="#f-cd7f4a46ce4a-rowLabel"/><storycode:paste select="#f-cd7f4a46ce4a-BlameMarker"/><storycode:paste select="#f-cd7f4a46ce4a-blameTip"/><storycode:paste select="#f-cd7f4a46ce4a-blameGutter"/><storycode:paste select="#f-cd7f4a46ce4a-AuthorshipState"/><storycode:paste select="#f-cd7f4a46ce4a-authorshipField"/><storycode:paste select="#f-cd7f4a46ce4a-attributionActions"/></storycode:file>

<a id="output-f14fc693b064"></a>
## `apps/web/src/lib/blamePref.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Blame Column Is Optional And Honest](contract.md).

<a id="f-f14fc693b064-BLAME_KEY"></a>
### `BLAME_KEY`

Whether the blame column is showing.
A display preference of this machine, like the theme and the zoom: it
follows the reader's current job — reviewing, bisecting, working out
whether a passage came from the agent — not the project. So localStorage,
beside the other appearance preferences.
Off by default, deliberately. A blame column is a permanent indent on every
line of every file, answering a question nobody asks most of the time.

<storycode:copy id="f-f14fc693b064-BLAME_KEY">
// Whether the blame column is showing.
//
// A display preference of this machine, like the theme and the zoom: it
// follows the reader's current job — reviewing, bisecting, working out
// whether a passage came from the agent — not the project. So localStorage,
// beside the other appearance preferences.
//
// Off by default, deliberately. A blame column is a permanent indent on every
// line of every file, answering a question nobody asks most of the time.

export const BLAME_KEY = "hickory.blameColumn";

</storycode:copy>
<a id="f-f14fc693b064-BLAME_CHANGED_EVENT"></a>
### `BLAME_CHANGED_EVENT`

The event a toggle fires so every open editor turns together. Two panes
disagreeing about whether the column is up would look like a bug.

<storycode:copy id="f-f14fc693b064-BLAME_CHANGED_EVENT">
/** The event a toggle fires so every open editor turns together. Two panes
 * disagreeing about whether the column is up would look like a bug. */
export const BLAME_CHANGED_EVENT = "hickory:blame-changed";

</storycode:copy>
<a id="f-f14fc693b064-loadBlameShown"></a>
### `loadBlameShown`

<storycode:copy id="f-f14fc693b064-loadBlameShown">
export function loadBlameShown(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): boolean {
  return storage?.getItem(BLAME_KEY) === "1";
}

</storycode:copy>
<a id="f-f14fc693b064-saveBlameShown"></a>
### `saveBlameShown`

<storycode:copy id="f-f14fc693b064-saveBlameShown">
export function saveBlameShown(
  shown: boolean,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): void {
  storage?.setItem(BLAME_KEY, shown ? "1" : "0");
}

</storycode:copy>
<a id="f-f14fc693b064-toggleBlameShown"></a>
### `toggleBlameShown`

Flip it, persist it, and tell every editor. Returns the new value.

<storycode:copy id="f-f14fc693b064-toggleBlameShown">
/** Flip it, persist it, and tell every editor. Returns the new value. */
export function toggleBlameShown(): boolean {
  const next = !loadBlameShown();
  saveBlameShown(next);
  window.dispatchEvent(new CustomEvent(BLAME_CHANGED_EVENT, { detail: next }));
  return next;
}

</storycode:copy>
<a id="f-f14fc693b064-onBlameChanged"></a>
### `onBlameChanged`

Hear about a change. Returns the unsubscribe.

<storycode:copy id="f-f14fc693b064-onBlameChanged">
/** Hear about a change. Returns the unsubscribe. */
export function onBlameChanged(handler: (shown: boolean) => void): () => void {
  const listener = (event: Event) => {
    const shown = (event as CustomEvent).detail;
    if (typeof shown === "boolean") handler(shown);
  };
  window.addEventListener(BLAME_CHANGED_EVENT, listener);
  return () => window.removeEventListener(BLAME_CHANGED_EVENT, listener);
}
</storycode:copy>

### Assemble `apps/web/src/lib/blamePref.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/blamePref.ts"><storycode:paste select="#f-f14fc693b064-BLAME_KEY"/><storycode:paste select="#f-f14fc693b064-BLAME_CHANGED_EVENT"/><storycode:paste select="#f-f14fc693b064-loadBlameShown"/><storycode:paste select="#f-f14fc693b064-saveBlameShown"/><storycode:paste select="#f-f14fc693b064-toggleBlameShown"/><storycode:paste select="#f-f14fc693b064-onBlameChanged"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/web/src/editor/blameGutter.ts](contract.md#output-cd7f4a46ce4a)
- [apps/web/src/editor/useBlame.ts](../../../mechanisms/apps/web/src/editor/useBlame.ts.md#output-9b02a0106d13)
- [apps/web/src/lib/blamePref.ts](contract.md#output-f14fc693b064)
- [apps/web/src/lib/menuBridge.ts](../../authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md#output-2f6629ca93b0)
- [apps/web/src/views/useZoom.ts](../../../mechanisms/apps/web/src/views/useZoom.ts.md#output-49e957f1c44e)
- [crates/hickory-cli/src/agent_lineage.rs](../../../mechanisms/crates/hickory-cli/src/agent_lineage.rs.md#output-7b65c6db8d7c)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)

### Tests and fixtures

- [apps/web/src/editor/blameGutter.controls.test.tsx](../../../mechanisms/apps/web/src/editor/blameGutter.controls.test.tsx.md#output-de1d44118481)
- [apps/web/src/editor/blameGutter.test.ts](contract.md#output-3d8799779e43)
- [crates/hickory-cli/tests/serve_blame.rs](../../../mechanisms/crates/hickory-cli/tests/serve_blame.rs.md#output-39e5027e9d27)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
