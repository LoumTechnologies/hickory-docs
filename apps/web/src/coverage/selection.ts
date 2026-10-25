import type { CoverageRun } from "./types";
let selected: CoverageRun | null = null;
const event = "hickory:coverage-selection";
export function selectedCoverage() { return selected; }
export function selectCoverage(run: CoverageRun | null) { selected = run; window.dispatchEvent(new Event(event)); }
export function onCoverageSelection(callback: () => void) { window.addEventListener(event, callback); return () => window.removeEventListener(event, callback); }
export function coveredLocations(run: CoverageRun, path: string): { line: number; count: number; originalPath: string; originalLine: number; countsAvailable: boolean }[] {
  const out: ReturnType<typeof coveredLocations> = [];
  for (const [file, data] of Object.entries(run.evidence?.report.files ?? {})) {
    for (const [n, count] of Object.entries(data.lines)) {
      if (file === path) out.push({ line: +n, count, countsAvailable: data.line_counts, originalPath: file, originalLine: +n });
      else if (run.mappings[file]?.[n]?.path === path) out.push({ line: run.mappings[file][n].line, count, countsAvailable: data.line_counts, originalPath: file, originalLine: +n });
    }
  }
  return out;
}
