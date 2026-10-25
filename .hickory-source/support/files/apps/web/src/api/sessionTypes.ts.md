<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/sessionTypes.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d38ea374e638115f6827b24c6063d22e2fbf5c94845a094725319e6eacfee963`.

<book:entry path="apps/web/src/api/sessionTypes.ts">
import type { Block, SessionView } from "./types";

export interface AuditRecord {
  id?: string; record_kind: string; kind?: string; actor?: string; recorded_at?: string;
  turn?: string | null; parent?: string | null; session_line: number;
  tool?: string; file?: string; state?: string; revision?: string; scope?: string;
  current_state?: string; dependency_snapshots?: { path: string; sha256: string | null; current_state?: string }[];
  path?: string; first_line?: number; last_line?: number; content?: string; sha256?: string;
  entry?: { statement: string; reason: string; evidence: string[]; affects: string[];
    alternatives: string[]; falsifier: string; supersedes?: string; requirement?: string; exclusions: string[]; dependencies?: string[]; witnesses?: string[]; challenges?: string[] };
}

export interface SessionViewResponse {
  audit?: AuditRecord[];
  path: string;
  source: string;
  view: SessionView;
  blocks: Block[];
  /** What each element declares about where it came from. */
  links: SessionLink[];
}

export interface SessionLink {
  family: "lineage" | "context" | "declared";
  /** Byte span in the session source. */
  span: [number, number];
  to: { path: string; lines?: [number, number] };
  title: string;
  historical?: AuditRecord & { claim?: string; quoted_literals?: { literal: string; present_in_recorded_passage: boolean }[] };
  /** Independently checked current document → output lineage. */
  evidence?: { path: string; lines: [number, number] };
  /** The 1-based lines of the session the span starts and ends on. */
  lines: [number, number];
}
</book:entry>

</book:doc>
