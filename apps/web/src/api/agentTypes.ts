/** Four-way token usage, split the way providers bill it. */
export interface AgentUsage {
  input_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  output_tokens: number;
}

export interface AgentTurn {
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
  buffers: { id?: string; kind?: string; document?: string; name: string; path: string | null; content: string; focused: boolean }[];
}
