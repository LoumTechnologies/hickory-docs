import { describe, expect, it } from "vitest";

import { computeEdits, toByteEdits } from "../../lib/diff";
import { applySourceEdits, mapEditsToSource, weaveOutputs } from "../../lib/weave";
import { deriveRibbons } from "../../lib/ribbons";
import { PROGRAM_DOC_PATH, PROGRAM_SOURCE, PROGRAM_TRANSCRIPT } from "./scripts";

describe("the home page's demo document", () => {
  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("weaves one shared fragment into two files", () => {
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

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("carries an edit made in a generated file back into the document", () => {
    const before = weaveOutputs(PROGRAM_SOURCE, PROGRAM_DOC_PATH)[0];
    const edited = before.content.replace("mid = (lo + hi) // 2", "mid = lo + (hi - lo) // 2");
    const source = applySourceEdits(
      PROGRAM_SOURCE,
      mapEditsToSource(before, toByteEdits(before.content, computeEdits(before.content, edited))),
    );

    // The fragment itself changed — not a copy of it stored beside the file.
    expect(source).toContain("mid = lo + (hi - lo) // 2");
    // So both files that paste that fragment move together, which is the
    // entire reason the middle column exists.
    for (const file of weaveOutputs(source, PROGRAM_DOC_PATH)) {
      expect(file.content).toContain("mid = lo + (hi - lo) // 2");
    }
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("comments the woven banner in the generated file's own language", () => {
    // The banner is the first line of the file, so a comment marker from the
    // wrong language is a syntax error in the very file it annotates.
    const file = weaveOutputs(PROGRAM_SOURCE, PROGRAM_DOC_PATH)[0];
    expect(file.content.startsWith("# woven by hick")).toBe(true);
  });

  // Protects docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md
  it("shows executable cells whose recorded transcript matches what they pin", () => {
    // The document claims an insertion point of 3 and pins it with an
    // expectation. A transcript that disagreed with the document beside it
    // would teach a visitor the opposite of the point being made.
    expect(PROGRAM_SOURCE).toContain("<hick:exec container=\"py\"");
    expect(PROGRAM_SOURCE).toContain('<hick:expect match="exact">3');
    expect(PROGRAM_TRANSCRIPT[0].out).toEqual(["3"]);

    // Every command in the transcript is one the document actually runs.
    for (const entry of PROGRAM_TRANSCRIPT) {
      expect(PROGRAM_SOURCE).toContain(entry.cmd);
    }
  });

  // Protects docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md
  it("declares a container for every cell that names one", () => {
    // A visitor pastes this document. An exec naming a container the document
    // never declares does not run.
    const declared = [...PROGRAM_SOURCE.matchAll(/<hick:container name="([^"]+)"/g)].map(
      (m) => m[1],
    );
    const used = [...PROGRAM_SOURCE.matchAll(/<hick:exec container="([^"]+)"/g)].map((m) => m[1]);
    expect(used.length).toBeGreaterThan(0);
    for (const name of used) expect(declared).toContain(name);
  });
});
