// In-app prompts keep sync and in-flight jobs responsive while an answer is pending.
import { useCallback, useEffect, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";

type PromptPolicy = { signal?: AbortSignal; agent?: boolean; currentRevision?: () => unknown };
type PromptBase = { id: string; title: string; agent: boolean; currentRevision?: () => unknown; baseline: unknown; cleanup?: () => void };
interface TextPrompt extends PromptBase { kind: "text"; initial: string; resolve: (value: string | null) => void }
interface ChoicePrompt<T> extends PromptBase { kind: "choice"; options: { label: string; value: T }[]; resolve: (value: T | null) => void }
type Prompt = TextPrompt | ChoicePrompt<unknown>;
const base = (title: string, policy: PromptPolicy): PromptBase => ({ id: crypto.randomUUID(), title, agent: policy.agent === true, currentRevision: policy.currentRevision, baseline: policy.currentRevision?.() });

export function usePrompt() {
  const [prompt, setPrompt] = useState<Prompt | null>(null);
  const pending = useRef<Prompt | null>(null);
  const replace = useCallback((next: Prompt, signal?: AbortSignal) => {
    if (signal?.aborted) { next.resolve(null); return; }
    pending.current?.cleanup?.(); pending.current?.resolve(null);
    pending.current = next;
    const cancel = () => { if (pending.current !== next) return; next.cleanup?.(); pending.current = null; setPrompt(null); next.resolve(null); };
    if (signal) { signal.addEventListener("abort", cancel, { once: true }); next.cleanup = () => signal.removeEventListener("abort", cancel); }
    setPrompt(next);
  }, []);
  const askText = useCallback((title: string, initial: string, policy: PromptPolicy = {}): Promise<string | null> => new Promise(resolve => {
    replace({ ...base(title, policy), kind: "text", initial, resolve }, policy.signal);
  }), [replace]);
  const askChoice = useCallback(<T,>(title: string, options: { label: string; value: T }[], policy: PromptPolicy = {}): Promise<T | null> => new Promise(resolve => {
    replace({ ...base(title, policy), kind: "choice", options, resolve: resolve as (value: unknown) => void }, policy.signal);
  }), [replace]);
  const settle = useCallback((value: unknown, expectedId?: string) => {
    const current = pending.current;
    if (expectedId && current?.id !== expectedId) return;
    pending.current = null; setPrompt(null); current?.cleanup?.(); current?.resolve(value as never);
  }, []);
  useEffect(() => () => { const current = pending.current; pending.current = null; current?.cleanup?.(); current?.resolve(null); }, []);
  return { prompt, askText, askChoice, settle };
}

export function PromptPanel({ prompt, onSettle }: { prompt: ReturnType<typeof usePrompt>["prompt"]; onSettle: (value: unknown, expectedId?: string) => void }) {
  return prompt ? <ActivePrompt key={prompt.id} prompt={prompt} onSettle={onSettle} /> : null;
}
function ActivePrompt({ prompt, onSettle }: { prompt: Prompt; onSettle: (value: unknown, expectedId?: string) => void }) {
  const [draft, setDraft] = useState(prompt.kind === "text" ? prompt.initial : "");
  const [error, setError] = useState("");
  const parent = `prompt:${prompt.id}`;
  const current = () => !prompt.currentRevision || Object.is(prompt.currentRevision(), prompt.baseline);
  const revision = useUiRevision(() => [prompt.id, draft, error, current()]);
  const finish = (value: unknown): UiOutcome => {
    if (value !== null && !current()) { const reason = "Source changed while this prompt was open. Cancel and request a current action."; setError(reason); return { status: "unavailable", reason }; }
    onSettle(value, prompt.id);
    return { status: "completed", value: { prompt: prompt.id, settled: true, cancelled: value === null, effect: "Caller resumes; inspect actual operation result, not just prompt settlement" } };
  };
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"], enabled = true) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && (key === "read" || key === "cancel" || current()), reason: !current() ? "Source changed; obtain a new prompt" : undefined }), execute } });
  const human = (): UiOutcome => ({ status: "requires_input", request: parent, reason: "This prompt requires a human decision under its caller's policy; inspect its actual result afterward" });
  if (prompt.kind === "text") {
    add("draft", "Set prompt text draft", { text: { type: "string", maxLength: 1048576 } }, async (args, actor) => {
      if (actor.kind === "agent" && !prompt.agent) return human();
      setDraft(args.text as string); return { status: "completed", value: { draft_changed: true, submitted: false } };
    });
    add("submit", "Apply prompt text", {}, async (_args, actor) => actor.kind === "agent" && !prompt.agent ? human() : finish(draft.trim() ? draft : null));
  } else add("choice", "Choose prompt option", { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async (args, actor) => {
    if (actor.kind === "agent" && !prompt.agent) return human();
    const option = prompt.options[args.index as number];
    return option ? finish(option.value) : { status: "unavailable", reason: "Observe a current option index" };
  });
  add("cancel", "Cancel active prompt", {}, async () => finish(null));
  add("read", "Read active prompt bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ id: prompt.id, title: prompt.title, kind: prompt.kind, draft: prompt.kind === "text" ? draft : null, options: prompt.kind === "choice" ? prompt.options.map((o, index) => ({ index, label: o.label })) : null, source_current: current(), agent_input_allowed: prompt.agent, error }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside active prompt" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  const invoke = useUiCommands(commands);
  const click = (key: string, args: Parameters<typeof invoke>[1] = {}) => { void invoke(`${parent}:${key}`, args).catch(reason => setError(String(reason))); };
  return <div className="prompt-panel" role="dialog" aria-label={prompt.title}>
    <p className="prompt-panel__title">{prompt.title}</p>
    {error && <p role="alert">{error}</p>}
    {!current() && <p role="status">Source changed; cancel and request a current action.</p>}
    {prompt.kind === "text" ? <form onSubmit={event => { event.preventDefault(); click("submit"); }}>
      <input autoFocus value={draft} aria-label={prompt.title} onChange={event => setDraft(event.target.value)} onKeyDown={event => { if (event.key === "Escape") click("cancel"); }} />
      <button type="submit" disabled={!current()}>Apply</button><button type="button" onClick={() => click("cancel")}>Cancel</button>
    </form> : <ul className="prompt-panel__options">{prompt.options.map((option, index) => <li key={`${option.label}-${index}`}><button type="button" disabled={!current()} onClick={() => click("choice", { index })}>{option.label}</button></li>)}<li><button type="button" onClick={() => click("cancel")}>Cancel</button></li></ul>}
  </div>;
}
