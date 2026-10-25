<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/agentTypes.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2c9bef08046c330feee9df0b8c67400c9afdae29cfb6a49c4ebcd5da485bb8b4`.

<book:entry path="apps/web/src/api/agentTypes.ts">
/** Four-way token usage, split the way providers bill it. */
export interface AgentUsage {
  input_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  output_tokens: number;
}

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

/** Session-wide spend across a document's turns, each turn priced on the
 * model it ran on. `usd` is null when any turn's model has no known price. */
export interface AgentTotals {
  usd: number | null;
  input: number;
  output: number;
  cache_read: number;
  cache_write: number;
}

export interface AgentTurnsResponse {
  turns: AgentTurn[];
  /** The provider the next turn would run on, defaults resolved. */
  provider: string;
  /** The model the next turn would run on, defaults resolved. */
  model: string;
  totals: AgentTotals;
  backend?: string;
}


/** A snapshot of what the user has open, taken when Send is pressed. */
export interface AgentEditorContext {
  ui_window?: string;
  live?: boolean;
  projection?: import("../lib/contextProjection").ReadingProjection;
  buffers: { observations?: (ReturnType<typeof import("../debug/observations").observationContext>[number] | ReturnType<typeof import("../debug/providerObservations").providerObservationContext>[number])[]; provenance_uri?: string; id?: string; kind?: string; document?: string; name: string; path: string | null; content: string; focused: boolean }[];
}
</book:entry>

</book:doc>
