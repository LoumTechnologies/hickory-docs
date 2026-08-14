// Ranked search over the lines of a project.
//
// Substring matching answers the wrong question. Typing `median` into a
// substring box gets you every line containing those letters, in file order,
// with no sense of which one you meant — and typing `how are runs stored`
// gets you nothing at all. BM25 ranks: a line matching a rare term scores
// above a line matching a common one, and a query of several words prefers
// the line that has most of them.
//
// This is the lexical half of the approach semble uses (BM25 fused with
// static embeddings via reciprocal rank fusion). The lexical half needs no
// model, no download, and no network, which is why it comes first; the
// embedding half is a lookup table of token vectors and can be added later
// without changing this interface.
//
// Three deliberate choices:
//
//  * **Identifiers are split.** `load_runs` and `loadRuns` both produce
//    `load` and `runs`, so searching either spelling finds both — the thing
//    substring search cannot do across naming conventions.
//  * **The whole project is one corpus.** Term rarity is only meaningful
//    against everything: `def` is common, `bisect` is not, and per-file IDF
//    would call both equally interesting in the file that holds them.
//  * **An exact substring always matches.** Ranking must never lose a line
//    that literally contains what you typed, whatever the statistics say.

import type { FileModel, LineageModel } from "./model";

const K1 = 1.2;
const B = 0.75;

/** Split a line into search terms: identifiers, split on case and separators. */
export function tokenize(text: string): string[] {
  const out: string[] = [];
  for (const raw of text.split(/[^A-Za-z0-9_]+/)) {
    if (!raw) continue;
    const lower = raw.toLowerCase();
    if (lower.length > 1) out.push(lower);
    // camelCase and snake_case both become their parts, so a query in one
    // convention finds an identifier written in the other.
    for (const part of raw.split(/_+|(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])/)) {
      const p = part.toLowerCase();
      if (p.length > 1 && p !== lower) out.push(p);
    }
  }
  return out;
}

interface Line {
  file: string;
  line: number;
  terms: string[];
  text: string;
}

export interface RankedLine {
  file: string;
  line: number;
  score: number;
  /** True when the line literally contains the query. */
  exact: boolean;
}

export class SearchIndex {
  private lines: Line[] = [];
  private df = new Map<string, number>();
  private avgLength = 0;

  constructor(files: Iterable<FileModel>) {
    for (const file of files) {
      file.lines.forEach((text, i) => {
        const terms = tokenize(text);
        this.lines.push({ file: file.path, line: i, terms, text });
        for (const term of new Set(terms)) this.df.set(term, (this.df.get(term) ?? 0) + 1);
      });
    }
    const total = this.lines.reduce((n, l) => n + l.terms.length, 0);
    this.avgLength = this.lines.length ? total / this.lines.length : 0;
  }

  /** Inverse document frequency, floored at zero so a term in every line
   *  cannot drag a score negative. */
  private idf(term: string): number {
    const n = this.lines.length;
    const df = this.df.get(term) ?? 0;
    return Math.max(0, Math.log(1 + (n - df + 0.5) / (df + 0.5)));
  }

  /** Rank every line that matches at all, best first. */
  search(query: string, limit = 200): RankedLine[] {
    const raw = query.trim();
    if (!raw) return [];
    const needle = raw.toLowerCase();
    const terms = tokenize(raw);
    const out: RankedLine[] = [];

    for (const line of this.lines) {
      const exact = line.text.toLowerCase().includes(needle);
      let score = 0;
      if (terms.length) {
        const counts = new Map<string, number>();
        for (const term of line.terms) counts.set(term, (counts.get(term) ?? 0) + 1);
        const length = line.terms.length || 1;
        for (const term of terms) {
          const tf = counts.get(term) ?? 0;
          if (!tf) continue;
          const norm = tf * (K1 + 1);
          const denom = tf + K1 * (1 - B + (B * length) / (this.avgLength || 1));
          score += this.idf(term) * (norm / denom);
        }
      }
      // A literal hit outranks a merely statistical one: somebody who typed
      // an exact string is asking for that string.
      if (exact) score += 10;
      if (score > 0) out.push({ file: line.file, line: line.line, score, exact });
    }

    out.sort((a, b) => b.score - a.score || a.file.localeCompare(b.file) || a.line - b.line);
    return out.slice(0, limit);
  }

  /**
   * The best matching lines per file, for folding a column down to its hits.
   *
   * Capped per file rather than only overall: ranked search matches far more
   * lines than a substring scan — `load_runs` reaches every line mentioning
   * `load` — so folding to *every* hit can leave a file no shorter than it
   * started, which is not a fold at all.
   */
  searchByFile(query: string, perFile = 15, limit = 1000): Map<string, Set<number>> {
    const byFile = new Map<string, Set<number>>();
    const counts = new Map<string, number>();
    for (const hit of this.search(query, limit)) {
      const seen = counts.get(hit.file) ?? 0;
      if (seen >= perFile) continue;
      counts.set(hit.file, seen + 1);
      if (!byFile.has(hit.file)) byFile.set(hit.file, new Set());
      byFile.get(hit.file)!.add(hit.line);
    }
    return byFile;
  }

  /** How many lines match anywhere, for reporting rather than folding. */
  countByFile(query: string): { lines: number; files: number } {
    const files = new Set<string>();
    const hits = this.search(query, 100000);
    for (const hit of hits) files.add(hit.file);
    return { lines: hits.length, files: files.size };
  }
}

/** One index per model, rebuilt only when the model changes. */
export function indexOf(model: LineageModel): SearchIndex {
  return new SearchIndex(model.files.values());
}
