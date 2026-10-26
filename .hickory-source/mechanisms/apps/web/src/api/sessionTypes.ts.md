<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/sessionTypes.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a3905c5c9955"></a>
## `apps/web/src/api/sessionTypes.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-a3905c5c9955-AuditRecord"></a>
### `AuditRecord`

<storycode:copy id="f-a3905c5c9955-AuditRecord">
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

</storycode:copy>
<a id="f-a3905c5c9955-SessionViewResponse"></a>
### `SessionViewResponse`

<storycode:copy id="f-a3905c5c9955-SessionViewResponse">
export interface SessionViewResponse {
  audit?: AuditRecord[];
  path: string;
  source: string;
  view: SessionView;
  blocks: Block[];
  /** What each element declares about where it came from. */
  links: SessionLink[];
}

</storycode:copy>
<a id="f-a3905c5c9955-SessionLink"></a>
### `SessionLink`

<storycode:copy id="f-a3905c5c9955-SessionLink">
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
</storycode:copy>

### Assemble `apps/web/src/api/sessionTypes.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/sessionTypes.ts"><storycode:paste select="#f-a3905c5c9955-AuditRecord"/><storycode:paste select="#f-a3905c5c9955-SessionViewResponse"/><storycode:paste select="#f-a3905c5c9955-SessionLink"/></storycode:file>

</storycode:doc>
