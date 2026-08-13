import { describe, expect, it } from "vitest";

import { computeEdits, toByteEdits } from "../../lib/diff";
import { applySourceEdits, mapEditsToSource, weaveOutputs } from "../../lib/weave";
import { deriveRibbons } from "../../lib/ribbons";
import {
  COLLAB_DOC_PATH,
  HAND_WRITTEN,
  COLLAB_SOURCE,
  KNOWLEDGE_DOC_PATH,
  KNOWLEDGE_STEPS,
  PROGRAM_DOC_PATH,
  PROGRAM_SOURCE,
  REMOTE_PULL_EDIT,
  advance,
  changedRange,
  findRange,
} from "./scripts";

const last = KNOWLEDGE_STEPS[KNOWLEDGE_STEPS.length - 1];

describe("the knowledge-work walkthrough", () => {
  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("never rewrites a byte the human typed", () => {
    // The agent appends — its session, its tool calls, the files it weaves.
    // The notes stay exactly as written, which is the difference between an
    // editor and a slideshow.
    for (const step of KNOWLEDGE_STEPS.slice(1)) {
      expect(step.source.startsWith(HAND_WRITTEN)).toBe(true);
    }
    expect(KNOWLEDGE_STEPS[0].source.length).toBeLessThan(HAND_WRITTEN.length);
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("generates nothing until the agent has run, then one ticket per note", () => {
    expect(weaveOutputs(KNOWLEDGE_STEPS[0].source, KNOWLEDGE_DOC_PATH)).toHaveLength(0);
    expect(weaveOutputs(KNOWLEDGE_STEPS[1].source, KNOWLEDGE_DOC_PATH)).toHaveLength(0);
    expect(weaveOutputs(last.source, KNOWLEDGE_DOC_PATH).map((f) => f.path)).toEqual([
      "jira/HD-412.md",
      "jira/HD-413.md",
      "jira/HD-414.md",
    ]);
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("traces every ticket description back to the note that produced it", () => {
    const files = weaveOutputs(last.source, KNOWLEDGE_DOC_PATH);
    for (const file of files) {
      const pasted = deriveRibbons(file, KNOWLEDGE_DOC_PATH, last.source).filter(
        (r) => r.kind === "paste",
      );
      // Exactly one note feeds each ticket, and the ribbon is not empty —
      // the picture the demo draws has to have something to draw.
      expect(pasted).toHaveLength(1);
      expect(pasted[0].bytes).toBeGreaterThan(0);
      expect(last.source.slice(pasted[0].sourceSpan[0], pasted[0].sourceSpan[1])).toBe(
        file.content.slice(pasted[0].outputRange[0], pasted[0].outputRange[1]),
      );
    }
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("carries an edit made in the ticket back into the meeting note", () => {
    const file = weaveOutputs(last.source, KNOWLEDGE_DOC_PATH)[0];
    const edited = file.content.replace(
      "Login fails on Safari 17",
      "Login fails on Safari 17 and 18",
    );
    const edits = toByteEdits(file.content, computeEdits(file.content, edited));
    const source = applySourceEdits(last.source, mapEditsToSource(file, edits));

    // The note itself changed — not a copy of it stored beside the ticket.
    expect(source).toContain("Login fails on Safari 17 and 18 whenever");
    // And re-weaving from the changed document reproduces the edited ticket.
    expect(weaveOutputs(source, KNOWLEDGE_DOC_PATH)[0].content).toBe(edited);
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("does not comment a Markdown ticket with a Rust comment", () => {
    // The woven banner is the first line of the file. `//` there is visible
    // prose in Markdown, so it would corrupt the very ticket it annotates.
    const file = weaveOutputs(last.source, KNOWLEDGE_DOC_PATH)[0];
    expect(file.content.startsWith("<!-- woven by hick")).toBe(true);
  });

  it("keeps the visitor's own edits when moving between steps that share a source", () => {
    const typed = { step: 4, source: `${last.source}\ntyped by hand`, edited: true };
    const next = advance(typed, 1);
    expect(next.step).toBe(5);
    expect(next.source).toBe(typed.source);
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("points at the text a step actually added, not at the end of the document", () => {
    // Step 4 inserts its tool calls INSIDE the session element step 3 opened.
    // "everything past the old length" would point at the closing tag — past
    // the only thing the step exists to show.
    const before = KNOWLEDGE_STEPS[2].source;
    const after = KNOWLEDGE_STEPS[3].source;
    const range = changedRange(before, after)!;
    expect(range).not.toBeNull();
    const revealed = after.slice(range[0], range[1]);
    // Whole lines, starting exactly at the first tool call — not one
    // character into it, which is where a character-exact diff would land.
    expect(revealed.startsWith("<hick:tool")).toBe(true);
    expect(revealed).toContain("jira.create_issue");
    // Everything after the tool calls is new too (the woven ticket files),
    // so the range runs to the end rather than stopping at the session.
    expect(revealed).toContain('<hick:file path="jira/HD-412.md"');
  });

  it("has nothing to reveal when a step changes nothing, and says where to look instead", () => {
    const lineage = KNOWLEDGE_STEPS[4];
    expect(changedRange(KNOWLEDGE_STEPS[3].source, lineage.source)).toBeNull();
    // The lineage step is exactly where every note must be on screen.
    const focus = findRange(lineage.source, lineage.focus!)!;
    expect(focus).not.toBeNull();
    expect(lineage.source.slice(focus[0], focus[1])).toBe(lineage.focus);
  });

  it("stops at both ends instead of running off the list", () => {
    const first = { step: 0, source: KNOWLEDGE_STEPS[0].source, edited: false };
    expect(advance(first, -1)).toBe(first);
    const end = { step: KNOWLEDGE_STEPS.length - 1, source: last.source, edited: false };
    expect(advance(end, 1)).toBe(end);
  });
});

describe("the literate-programming demo", () => {
  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("tangles one shared fragment into two files", () => {
    const files = weaveOutputs(PROGRAM_SOURCE, PROGRAM_DOC_PATH);
    expect(files.map((f) => f.path)).toEqual(["search/bisect.py", "search/test_bisect.py"]);
    for (const file of files) {
      expect(file.content).toContain("def bisect_left(xs, target):");
    }
    // The claim the demo makes out loud: there is only ONE copy of the code,
    // so both files quote the same source span.
    const spans = files.map(
      (f) =>
        deriveRibbons(f, PROGRAM_DOC_PATH, PROGRAM_SOURCE).find(
          (r) => r.kind === "paste" && PROGRAM_SOURCE.slice(...r.sourceSpan).includes("bisect_left"),
        )!.sourceByteSpan,
    );
    expect(spans[0]).toEqual(spans[1]);
  });

  it("keeps the two tangled files in step when the fragment is edited", () => {
    const before = weaveOutputs(PROGRAM_SOURCE, PROGRAM_DOC_PATH)[0];
    const edited = before.content.replace("mid = (lo + hi) // 2", "mid = lo + (hi - lo) // 2");
    const source = applySourceEdits(
      PROGRAM_SOURCE,
      mapEditsToSource(before, toByteEdits(before.content, computeEdits(before.content, edited))),
    );
    for (const file of weaveOutputs(source, PROGRAM_DOC_PATH)) {
      expect(file.content).toContain("mid = lo + (hi - lo) // 2");
    }
  });
});

describe("the collaboration demo's document", () => {
  it("weaves the limits table the pulled commit edits", () => {
    const files = weaveOutputs(COLLAB_SOURCE, COLLAB_DOC_PATH);
    expect(files.map((f) => f.path)).toEqual(["docs/limits.md"]);
    expect(files[0].content).toContain("scale 6000 requests / minute");
    // Pull replaces a line that must actually be in the document.
    expect(COLLAB_SOURCE).toContain(REMOTE_PULL_EDIT.find);
  });
});
