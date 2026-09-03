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
