<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/agentTypes.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1689be10ab92"></a>
## `apps/web/src/api/agentTypes.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1689be10ab92-AgentUsage"></a>
### `AgentUsage`

Four-way token usage, split the way providers bill it.

<storycode:copy id="f-1689be10ab92-AgentUsage">
/** Four-way token usage, split the way providers bill it. */
export interface AgentUsage {
  input_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  output_tokens: number;
}

</storycode:copy>
<a id="f-1689be10ab92-AgentTurn"></a>
### `AgentTurn`

<storycode:copy id="f-1689be10ab92-AgentTurn">
export interface AgentTurn {
  backend?: string;
  id: string;
  parent_id: string | null;
  prompt: string;
  answer: string | null;
  status: "running" | "ok" | "error" | string;
  error: string | null;
  created_at: string;
  /** Provider selector this turn ran on ("anthropic", "openai", …). */
  provider: string;
  /** Model id this turn ran on — recorded per turn, so a mid-conversation
   * model change stays visible. */
  model: string;
  /** The turn's final token usage; null while running or after a failure. */
  usage: AgentUsage | null;
  /** The session file this turn is recorded in, relative to the folder. */
  session?: string;
}

</storycode:copy>
<a id="f-1689be10ab92-AgentTotals"></a>
### `AgentTotals`

Session-wide spend across a document's turns, each turn priced on the
model it ran on. `usd` is null when any turn's model has no known price.

<storycode:copy id="f-1689be10ab92-AgentTotals">
/** Session-wide spend across a document's turns, each turn priced on the
 * model it ran on. `usd` is null when any turn's model has no known price. */
export interface AgentTotals {
  usd: number | null;
  input: number;
  output: number;
  cache_read: number;
  cache_write: number;
}

</storycode:copy>
<a id="f-1689be10ab92-AgentTurnsResponse"></a>
### `AgentTurnsResponse`

<storycode:copy id="f-1689be10ab92-AgentTurnsResponse">
export interface AgentTurnsResponse {
  turns: AgentTurn[];
  /** The provider the next turn would run on, defaults resolved. */
  provider: string;
  /** The model the next turn would run on, defaults resolved. */
  model: string;
  totals: AgentTotals;
  backend?: string;
}


</storycode:copy>
<a id="f-1689be10ab92-AgentEditorContext"></a>
### `AgentEditorContext`

A snapshot of what the user has open, taken when Send is pressed.

<storycode:copy id="f-1689be10ab92-AgentEditorContext">
/** A snapshot of what the user has open, taken when Send is pressed. */
export interface AgentEditorContext {
  ui_window?: string;
  live?: boolean;
  projection?: import("../lib/contextProjection").ReadingProjection;
  buffers: { observations?: (ReturnType<typeof import("../debug/observations").observationContext>[number] | ReturnType<typeof import("../debug/providerObservations").providerObservationContext>[number])[]; provenance_uri?: string; id?: string; kind?: string; document?: string; name: string; path: string | null; content: string; focused: boolean }[];
}
</storycode:copy>

### Assemble `apps/web/src/api/agentTypes.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/agentTypes.ts"><storycode:paste select="#f-1689be10ab92-AgentUsage"/><storycode:paste select="#f-1689be10ab92-AgentTurn"/><storycode:paste select="#f-1689be10ab92-AgentTotals"/><storycode:paste select="#f-1689be10ab92-AgentTurnsResponse"/><storycode:paste select="#f-1689be10ab92-AgentEditorContext"/></storycode:file>

</storycode:doc>
