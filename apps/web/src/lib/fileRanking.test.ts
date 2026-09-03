// Protects docs/guarantees/editor-intelligence/a-typed-path-opens-that-file.md

import { describe, expect, it } from "vitest";
import { rankFile, rankFiles } from "./fileRanking";

const file = (path: string) => ({ path, name: path.split("/").pop()! });

const FILES = [
  file("apps/web/src/config/default.json"),
  file("apps/web/src/lang.ts"),
  file("crates/hick-lsp/src/lang_detect.rs"),
  file("crates/hick-lsp/src/document.rs"),
  file("docs/lang_detect.md"),
  file("lang_detect.rs"),
];

describe("ranking files for the command bar", () => {
  it("puts a path typed out in full first", () => {
    // The defect this exists for: on 2026-09-02, typing this opened
    // `default.json`.
    const ranked = rankFiles(FILES, "crates/hick-lsp/src/lang_detect.rs");
    expect(ranked[0].path).toBe("crates/hick-lsp/src/lang_detect.rs");
  });

  it("puts an exact name above anything that merely contains it", () => {
    const ranked = rankFiles(FILES, "lang_detect.rs");
    expect(ranked.map((f) => f.path)).toEqual([
      // The bare name, exact.
      "lang_detect.rs",
      // Then the one whose path ends with it.
      "crates/hick-lsp/src/lang_detect.rs",
    ]);
  });

  it("prefers a name match to a path match", () => {
    const ranked = rankFiles(
      [file("lang/anything.ts"), file("src/lang.ts")],
      "lang",
    );
    expect(ranked[0].path).toBe("src/lang.ts");
  });

  it("breaks a tie by the shorter path", () => {
    const ranked = rankFiles(
      [file("vendor/deep/nested/app.ts"), file("src/app.ts")],
      "app.ts",
    );
    expect(ranked[0].path).toBe("src/app.ts");
  });

  it("does not read a name match as a path suffix", () => {
    // `detect.rs` is part of `lang_detect.rs`, not a path segment of it.
    expect(rankFile(file("crates/x/lang_detect.rs"), "detect.rs")).toBe(4);
    expect(rankFile(file("crates/x/detect.rs"), "detect.rs")).toBe(1);
  });

  it("ignores case and surrounding space", () => {
    expect(rankFile(file("src/Program.cs"), "  program.cs  ")).toBe(1);
  });

  it("says no when nothing matches", () => {
    expect(rankFile(file("src/app.ts"), "nonsense")).toBeNull();
    expect(rankFiles(FILES, "nonsense")).toEqual([]);
  });

  it("offers everything for an empty query, in tree order", () => {
    expect(rankFiles(FILES, "")).toHaveLength(FILES.length);
  });

  it("caps AFTER ranking, never during the walk", () => {
    // The second half of the same defect: the cap used to be applied while
    // the tree was being walked, so a better match sitting later in the tree
    // was dropped before anything compared it.
    const noise = Array.from({ length: 50 }, (_, i) => file(`a/noise${i}/thing.ts`));
    const ranked = rankFiles([...noise, file("thing.ts")], "thing.ts", 5);
    expect(ranked).toHaveLength(5);
    expect(ranked[0].path).toBe("thing.ts");
  });
});
