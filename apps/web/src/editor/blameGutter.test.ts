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
const at = (daysAgo: number) => Math.floor((NOW - daysAgo * 86_400_000) / 1000);

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

describe("labelling a run of lines", () => {
  it("labels the first line of a run", () => {
    expect(rowLabel(entry(), undefined)).toBe("Ada 3d ago");
  });

  it("leaves the rest of the run blank", () => {
    // Repeating the same name down forty rows is how a blame column becomes
    // wallpaper: the eye stops reading it, and the boundaries — which are the
    // actual information — disappear into the repetition.
    expect(rowLabel(entry({ line: 2 }), entry())).toBe("");
  });

  it("labels again where the commit changes", () => {
    const later = entry({ line: 2, commit: "ffff0000", author: "Grace Hopper" });
    expect(rowLabel(later, entry())).toBe("Grace 3d ago");
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
