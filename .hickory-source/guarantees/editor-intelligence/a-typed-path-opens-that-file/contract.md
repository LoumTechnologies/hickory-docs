<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Typed Path Opens That File

For: Authors using completion, diagnostics and refactoring inside literate source.

Given the command bar in files mode, when a query is typed, then the answers
are **ranked**, best first: a path typed out in full, then an exact file name,
then a path suffix, then a name that starts with the query, then a name that
contains it, then a path that contains it. Ties go to the shorter path.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-typed-path-opens-that-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-e6261dad35a2"></a>
## `apps/web/src/lib/fileRanking.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Typed Path Opens That File](contract.md).

<a id="f-e6261dad35a2-file"></a>
### `file`

Protects docs/guarantees/editor-intelligence/a-typed-path-opens-that-file.md

<storycode:copy id="f-e6261dad35a2-file">
// Protects docs/guarantees/editor-intelligence/a-typed-path-opens-that-file.md

import { describe, expect, it } from "vitest";
import { rankFile, rankFiles } from "./fileRanking";

const file = (path: string) => ({ path, name: path.split("/").pop()! });

</storycode:copy>
<a id="f-e6261dad35a2-FILES"></a>
### `FILES`

<storycode:copy id="f-e6261dad35a2-FILES">
const FILES = [
  file("apps/web/src/config/default.json"),
  file("apps/web/src/lang.ts"),
  file("crates/hick-lsp/src/lang_detect.rs"),
  file("crates/hick-lsp/src/document.rs"),
  file("docs/lang_detect.md"),
  file("lang_detect.rs"),
];

</storycode:copy>
<a id="f-e6261dad35a2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e6261dad35a2-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/lib/fileRanking.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/fileRanking.test.ts"><storycode:paste select="#f-e6261dad35a2-file"/><storycode:paste select="#f-e6261dad35a2-FILES"/><storycode:paste select="#f-e6261dad35a2-expression_statement"/></storycode:file>

<a id="output-f38ff51e726c"></a>
## `apps/web/src/lib/fileRanking.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Typed Path Opens That File](contract.md).

<a id="f-f38ff51e726c-RankableFile"></a>
### `RankableFile`

How the command bar orders files for a typed query.
It used to do neither half of this. Matching was a bare
`path.includes(term)` with the results left in whatever order the folder
tree happened to walk, and the cap of 40 was applied DURING that walk — so
a file matching well but sitting late in the tree was dropped before
anything compared it to the ones that got in. Typing a path and being
handed something else was the symptom recorded on 2026-09-02, when
`crates/hick-lsp/src/lang_detect.rs` opened `default.json`.
The rule this encodes is the one every editor's Go-To-File follows: the
more of the query the file's own NAME accounts for, the better the match,
and a path you typed out in full is not a guess to be second-guessed.
Something the bar can offer for a file query.

<storycode:copy id="f-f38ff51e726c-RankableFile">
// How the command bar orders files for a typed query.
//
// It used to do neither half of this. Matching was a bare
// `path.includes(term)` with the results left in whatever order the folder
// tree happened to walk, and the cap of 40 was applied DURING that walk — so
// a file matching well but sitting late in the tree was dropped before
// anything compared it to the ones that got in. Typing a path and being
// handed something else was the symptom recorded on 2026-09-02, when
// `crates/hick-lsp/src/lang_detect.rs` opened `default.json`.
//
// The rule this encodes is the one every editor's Go-To-File follows: the
// more of the query the file's own NAME accounts for, the better the match,
// and a path you typed out in full is not a guess to be second-guessed.

/** Something the bar can offer for a file query. */
export interface RankableFile {
  /** Root-relative path, the way the tree spells it. */
  path: string;
  /** The last segment. */
  name: string;
}

</storycode:copy>
<a id="f-f38ff51e726c-rankFile"></a>
### `rankFile`

How well a file answers a query. Lower is better; `null` is no match.
The bands are deliberately coarse and far apart: within a band, ties are
broken by path length, which is what makes `src/app.ts` beat
`vendor/deep/nested/app.ts` for the query `app.ts` without either of them
needing a rule of its own.

<storycode:copy id="f-f38ff51e726c-rankFile">
/**
 * How well a file answers a query. Lower is better; `null` is no match.
 *
 * The bands are deliberately coarse and far apart: within a band, ties are
 * broken by path length, which is what makes `src/app.ts` beat
 * `vendor/deep/nested/app.ts` for the query `app.ts` without either of them
 * needing a rule of its own.
 */
export function rankFile(file: RankableFile, term: string): number | null {
  const query = term.trim().toLowerCase();
  if (!query) return 0;
  const path = file.path.toLowerCase();
  const name = file.name.toLowerCase();

  // Typing a path out in full is an instruction, not a hint.
  if (path === query) return 0;
  if (name === query) return 1;
  // A path suffix: `src/lang_detect.rs` for `crates/…/src/lang_detect.rs`.
  // Anchored on a separator so `detect.rs` does not read as a suffix of
  // `lang_detect.rs` here — it is a name match, which the band below covers.
  if (path.endsWith(`/${query}`)) return 2;
  if (name.startsWith(query)) return 3;
  if (name.includes(query)) return 4;
  if (path.includes(query)) return 5;
  return null;
}

</storycode:copy>
<a id="f-f38ff51e726c-rankFiles"></a>
### `rankFiles`

The files that answer a query, best first, capped.
The cap is applied AFTER ranking. Applying it during the walk — which is
what this replaced — means the answer depends on the order the tree was
built in, and the best match can be thrown away without ever being looked
at.

<storycode:copy id="f-f38ff51e726c-rankFiles">
/**
 * The files that answer a query, best first, capped.
 *
 * The cap is applied AFTER ranking. Applying it during the walk — which is
 * what this replaced — means the answer depends on the order the tree was
 * built in, and the best match can be thrown away without ever being looked
 * at.
 */
export function rankFiles<T extends RankableFile>(
  files: readonly T[],
  term: string,
  limit = 40,
): T[] {
  const scored: { file: T; score: number }[] = [];
  for (const file of files) {
    const score = rankFile(file, term);
    if (score !== null) scored.push({ file, score });
  }
  scored.sort(
    (a, b) =>
      a.score - b.score ||
      // A shorter path is nearer the surface of the project, and is the one
      // meant far more often than not.
      a.file.path.length - b.file.path.length ||
      a.file.path.localeCompare(b.file.path),
  );
  return scored.slice(0, limit).map((entry) => entry.file);
}
</storycode:copy>

### Assemble `apps/web/src/lib/fileRanking.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/fileRanking.ts"><storycode:paste select="#f-f38ff51e726c-RankableFile"/><storycode:paste select="#f-f38ff51e726c-rankFile"/><storycode:paste select="#f-f38ff51e726c-rankFiles"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/fileRanking.ts](contract.md#output-f38ff51e726c)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hick-lsp/src/lang_detect.rs](../../../mechanisms/crates/hick-lsp/src/lang_detect.rs.md#output-0053bd574cca)
- [crates/hick-plugins/src/routing.rs](../../../mechanisms/crates/hick-plugins/src/routing.rs.md#output-007f87027669)

### Tests and fixtures

- [apps/web/src/lib/fileRanking.test.ts](contract.md#output-e6261dad35a2)
- [crates/hick-lang/tests/fixtures/literate-project/app.ts](../../../support/files/crates/hick-lang/tests/fixtures/literate-project/app.ts.md)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
