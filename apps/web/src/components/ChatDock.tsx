import { createPortal } from "react-dom";
import { base64Of } from "../editor/mdPaste";
import { useConversationActions } from "./uiConversationActions";
import { useAgentStatus } from "../shell/AgentStatus";
import { agentIdentity } from "../lib/agentStatus";
import { MoreActions } from "./MoreActions";
import { ConversationModelChoice } from "./ConversationModelChoice";
import { UiCheckbox } from "./UiCheckbox";
import { ContextUpdates } from "./ContextUpdates";
import { registerInvestigationConversation } from "../lib/reviewInvestigations";
import { AgentProject } from "./AgentProject";
import { FeatureExperiments } from "./FeatureExperiments";
import { AgentChanges } from "./AgentChanges";
import { acpApi } from "../api/acp";
import type { AgentChange } from "../api/acp";
import type { AgentEditorContext } from "../api/agentTypes";
// The conversation is a TREE, not a transcript. Every message records the turn
// it continued from, so selecting an earlier turn and sending again forks a
// branch rather than destroying what came after it — you can try a different
// instruction from the same starting point and keep both. What the dock shows
// is the path from the root to the currently selected tip; where a turn has
// more than one child, a small switcher moves between the alternatives.
//
// Live output streams on the run channel keyed by the turn id, which is also
// the run id, so the answer appears as it is generated and is then replaced by
// the persisted turn when the session finishes.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FeatureSettings } from "./FeatureSettings";
import { loadCollapsedLineage, saveCollapsedLineage } from "../lib/conversationLineage";
import { api } from "../api/client";
import { AcpControls, useAcp } from "./AcpControls";
import { AgentPicker, preferredAgent, rememberAgent } from "./AgentPicker";
import type {
  AgentTotals,
  AgentTurn,
  AgentWsEvent,
  RunWsMessage,
  TranscriptEvent,
} from "../api/types";
import type { Realtime } from "../api/realtime";
import { ConversationEditor } from "../views/ConversationEditor";
import { ChatTree } from "./ChatTree";
import { StopMark } from "./icons";
import { useResponseRecovery } from "../lib/responseRecovery";

/** The providers the backend accepts (ProviderSelection::ALL), with each
 * one's default model as the free-text input's placeholder. */
export const PROVIDERS: { id: string; label: string; defaultModel: string }[] =
  [
    { id: "anthropic", label: "Anthropic", defaultModel: "claude-sonnet-5" },
    { id: "openai", label: "OpenAI", defaultModel: "gpt-5" },
    { id: "codex-subscription", label: "Codex — ChatGPT subscription", defaultModel: "gpt-6.1-sol" },
    { id: "grok-subscription", label: "Grok — subscription", defaultModel: "grok-build" },
    { id: "deepseek", label: "DeepSeek", defaultModel: "deepseek-chat" },
    { id: "grok", label: "xAI (Grok)", defaultModel: "grok-4" },
    { id: "openrouter", label: "OpenRouter", defaultModel: "openrouter/auto" },
    { id: "gab", label: "Gab AI", defaultModel: "arya" },
  ];

/** The model a provider runs when none is typed, for the placeholder. */
export function defaultModelFor(provider: string): string {
  return PROVIDERS.find((p) => p.id === provider)?.defaultModel ?? "";
}

/** Compact token count: 950 → "950", 12,400 → "12.4k", 3,100,000 → "3.1M". */
export function formatTokens(n: number): string {
  const compact = (value: number, suffix: string) => {
    const rounded =
      value >= 100
        ? Math.round(value).toString()
        : value.toFixed(1).replace(/\.0$/, "");
    return `${rounded}${suffix}`;
  };
  if (n < 1000) return String(n);
  if (n < 1_000_000) return compact(n / 1000, "k");
  return compact(n / 1_000_000, "M");
}

/** Session cost to four decimal places; "$—" when a model's price is
 * unknown (the server reports unknown, never a guess). */
export function formatUsd(usd: number | null): string {
  return usd === null ? "$—" : `$${usd.toFixed(4)}`;
}

/** Cache hit rate = reads / (uncached input + reads); null before any
 * input-bearing turn, so the dock can show "—" instead of a fake 0%. */
export function cacheHitRate(input: number, cacheRead: number): number | null {
  const denominator = input + cacheRead;
  return denominator === 0 ? null : cacheRead / denominator;
}

/** The dock header's one-line spend summary,
 * e.g. "$0.0342 · in 12.4k · out 3.1k · cache 78%". */
export function statsLine(t: AgentTotals): string {
  if (t.subscription) return t.input + t.output + t.cache_read > 0
    ? `${t.mixed ? "Mixed subscription/API" : "Subscription"} · reported in ${formatTokens(t.input)} · out ${formatTokens(t.output)} · cache read ${formatTokens(t.cache_read)} · credit cost/balance unknown`
    : "Subscription · token usage unknown · credit cost/balance unknown";
  const rate = cacheHitRate(t.input, t.cache_read);
  const cache = rate === null ? "cache —" : `cache ${Math.round(rate * 100)}%`;
  return [
    formatUsd(t.usd),
    `in ${formatTokens(t.input)}`,
    `out ${formatTokens(t.output)}`,
    cache,
  ].join(" · ");
}

/** The tooltip behind the stats line: all four counters, uncompacted. */
export function statsTooltip(t: AgentTotals): string {
  if (t.subscription) return `Reported cache writes: ${t.cache_write_available ? formatTokens(t.cache_write) + " (partial)" : "unknown"}. Reported tokens only; totals may be incomplete. Subscription usage is separate from API charges. Cache writes are shown only when reported; Codex cache writes, reasoning tokens, provider call counts, credit cost and balance are unknown. Hickory never switches billing or purchases credits.`;
  const n = (v: number) => v.toLocaleString("en-US");
  return [
    `cost ${formatUsd(t.usd)}`,
    `input ${n(t.input)} tokens`,
    `output ${n(t.output)} tokens`,
    `cache read ${n(t.cache_read)} tokens`,
    `cache write ${n(t.cache_write)} tokens`,
  ].join("\n");
}

/** Whether any turn has reported usage yet — before that the stats line
 * would be all zeros, which reads as free rather than unstarted. */
export function hasUsage(t: AgentTotals): boolean {
  return t.input + t.output + t.cache_read + t.cache_write > 0;
}

export interface ChatDockProps {
  initialPrompt?: string;
  docId: string;
  /** Folder identity invalidates queued recovery when this window attaches a folder. */
  recoveryWorkspace?: string;
  getContext?: () => Promise<AgentEditorContext>;
  applyAgentChange?: (change: AgentChange) => void | Promise<void>;
  contextLabel?: string;
  contextControls?: import("react").ReactNode;
  realtime: Realtime;
  /** Collapse to the composer only. */
  /** Collapse, for the one place this is still a dock. As a PANE there is
   * nothing to collapse — the pane itself is the affordance, and it can be
   * resized, moved, or closed like anything else. */
  collapsed?: boolean;
  onToggleCollapsed?: () => void;
  /** The agent may have edited the document; refresh the views. */
  onAgentFinished: () => void;
  /** Open a session file (the conversation's record) in the app. */
  onOpenSession?: (path: string) => void;
}

/** Fold one live agent event into the streaming preview.
 *
 * Tokens append as they arrive; a script or tool start becomes a one-line
 * marker so the pause while it runs reads as work, not a stall. Everything
 * else (usage, thinking, completions) is bookkeeping the finished turn will
 * render properly, so it adds nothing here. */
export function appendStream(
  prev: string,
  e: TranscriptEvent | AgentWsEvent,
): string {
  if (e.kind === "token" || e.kind === "out" || e.kind === "err")
    return prev + e.data;
  if (e.kind === "script_started")
    return `${prev}\n\n[running ${e.lang} script…]\n`;
  if (e.kind === "tool_started") return `${prev}\n\n[${e.name}…]\n`;
  return prev;
}

/** The model's reasoning, streamed apart from its answer and shown folded. */
export function appendReasoning(
  prev: string,
  e: TranscriptEvent | AgentWsEvent,
): string {
  return e.kind === "reasoning" ? prev + e.data : prev;
}

/**
 * The composer's slash commands — the keyboard way to move around the tree.
 *
 *   /rewind      make the tip's parent the tip (one step back)
 *   /rewind N    go back N turns
 *   /rerun       start a SECOND conversation, from what you learned in this one
 *   /tree        zoom out: the conversation as a tree (toggle)
 *   /new         start a thread that continues from nothing
 *   /help        list these
 *
 * Rewind and re-run are two different acts and the difference is only about
 * what is KEPT (docs/specs/freeform/sessions-you-run-again.md):
 *
 *   rewind — one session file, one conversation, an abandoned branch in the
 *            tree. The change of mind is kept, visible in the file forever.
 *   re-run — a second session file; the first is a draft you may discard, and
 *            nothing in session 2 refers to session 1. The change of mind is
 *            NOT kept.
 *
 * A re-run is not a reenactment: every byte of the second session is written
 * by the harness, the tools really run, and it stands without the first ever
 * having existed.
 *
 * Anything else starting with "/" is a message, not a command — a path or a
 * date at the start of a sentence must not be eaten.
 */
export type SlashCommand =
  | { kind: "rewind"; steps: number }
  | { kind: "rerun" }
  | { kind: "tree" }
  | { kind: "new" }
  | { kind: "help" };

export function parseSlash(text: string): SlashCommand | null {
  const m = text.trim().match(/^\/(rewind|rerun|tree|new|help)(?:\s+(\d+))?$/i);
  if (!m) return null;
  switch (m[1].toLowerCase()) {
    case "rewind":
      return { kind: "rewind", steps: m[2] ? Math.max(1, Number(m[2])) : 1 };
    case "rerun":
      return { kind: "rerun" };
    case "tree":
      return { kind: "tree" };
    case "new":
      return { kind: "new" };
    default:
      return { kind: "help" };
  }
}

/** What each act keeps — shown where the choice is made, not in a doc. */
export const REWIND_TIP =
  "Rewind — continue from here, in THIS conversation. The turns after it stay on their own branch in the same session file, so the change of mind is kept and visible forever.";

export const RERUN_TIP =
  "Re-run — start a SECOND conversation from what you learned in this one. The first becomes a draft you may discard (sessions/ is gitignored), and nothing in the second refers to it. The change of mind is not kept.";

/** The turn `steps` back from `tip` along parent pointers (null = root). */
export function rewindFrom(
  turns: readonly AgentTurn[],
  tip: string | null,
  steps: number,
): string | null {
  const byId = new Map(turns.map((t) => [t.id, t]));
  let cursor = tip;
  for (let i = 0; i < steps && cursor; i++)
    cursor = byId.get(cursor)?.parent_id ?? null;
  return cursor;
}

export const SLASH_HELP =
  "/rewind [N] — back N turns, keeping the branch · /rerun — a second conversation, discarding this one · /tree — zoom out to the tree · /new — start a thread · /help";

/** Path from a root turn down to `tip`, oldest first. */
export function branchOf(turns: AgentTurn[], tip: string | null): AgentTurn[] {
  if (!tip) return [];
  const byId = new Map(turns.map((t) => [t.id, t]));
  const path: AgentTurn[] = [];
  let cursor: string | null = tip;
  const seen = new Set<string>();
  while (cursor && !seen.has(cursor)) {
    seen.add(cursor);
    const turn = byId.get(cursor);
    if (!turn) break;
    path.push(turn);
    cursor = turn.parent_id;
  }
  return path.reverse();
}

/** Children of `parent` (null = conversation roots), oldest first. */
export function childrenOf(
  turns: AgentTurn[],
  parent: string | null,
): AgentTurn[] {
  return turns.filter((t) => t.parent_id === parent);
}

/** Follow the newest child at each step — the tip a branch switch lands on. */
export function deepestFrom(turns: AgentTurn[], from: string): string {
  let cursor = from;
  for (;;) {
    const kids = childrenOf(turns, cursor);
    if (kids.length === 0) return cursor;
    cursor = kids[kids.length - 1].id;
  }
}

async function prepareChatFile(file: File): Promise<NonNullable<AgentEditorContext["attachments"]>[number]> {
  let content: Blob = file;
  let media_type = file.type || "application/octet-stream";
  if (file.size > 25 * 1024 * 1024) throw Error(`${file.name} exceeds 25 MiB. Choose a smaller file.`);
  if (file.type.startsWith("image/")) {
    const bitmap = await createImageBitmap(file);
    try {
      const canvas = document.createElement("canvas");
      let blob: Blob | null = null;
      for (const dimension of [1024, 768, 512, 256]) {
        const scale = Math.min(1, dimension / Math.max(bitmap.width, bitmap.height));
        canvas.width = Math.max(1, Math.round(bitmap.width * scale)); canvas.height = Math.max(1, Math.round(bitmap.height * scale));
        canvas.getContext("2d")!.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
        blob = await new Promise<Blob | null>(resolve => canvas.toBlob(resolve, "image/png"));
        if (blob && blob.size <= 131072) break;
      }
      if (!blob || blob.size > 131072) throw Error(`${file.name} is too detailed. Crop it and retry.`);
      content = blob; media_type = "image/png";
    } finally { bitmap.close(); }
  } else {
    if (file.size > 393216) throw Error(`${file.name} exceeds 384 KiB. Choose a smaller file.`);
  }
  const encoded = await base64Of(content);
  return { name: file.name || "pasted-image.png", media_type, data: encoded.slice(encoded.indexOf(",") + 1) };
}

function CameraCapture({ disabled, onFile, onError }: { disabled: boolean; onFile: (file: File) => void; onError: (error: string) => void }) {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const video = useRef<HTMLVideoElement | null>(null);
  const dialog = useRef<HTMLDialogElement | null>(null);
  const cameraEpoch = useRef(0);
  const close = () => { cameraEpoch.current++; setStream(current => { current?.getTracks().forEach(track => track.stop()); return null; }); };
  useEffect(() => () => { cameraEpoch.current++; }, []);
  useEffect(() => { if (stream) dialog.current?.showModal(); if (video.current && stream) { video.current.srcObject = stream; void video.current.play().catch(() => onError("Camera preview could not play. Close it and retry.")); } return () => stream?.getTracks().forEach(track => track.stop()); }, [stream]);
  const open = async () => {
    const epoch = ++cameraEpoch.current;
    try {
      if (!navigator.mediaDevices?.getUserMedia) throw Error("Camera unavailable. Use HTTPS or attach an existing photo.");
      const camera = await navigator.mediaDevices.getUserMedia({ video: { facingMode: "environment" }, audio: false });
      if (epoch !== cameraEpoch.current) { camera.getTracks().forEach(track => track.stop()); return; }
      setStream(camera);
    } catch (error) { if (epoch !== cameraEpoch.current) return; onError(`Could not open the camera: ${error}. Allow camera access or attach an existing photo.`); }
  };
  const capture = () => {
    const element = video.current;
    if (!element?.videoWidth) { onError("Camera is not ready. Wait for the preview and retry."); return; }
    const canvas = document.createElement("canvas"); canvas.width = element.videoWidth; canvas.height = element.videoHeight;
    canvas.getContext("2d")!.drawImage(element, 0, 0);
    const epoch = cameraEpoch.current;
    canvas.toBlob(blob => { if (epoch !== cameraEpoch.current) return; if (blob) { onFile(new File([blob], "camera-photo.png", { type: "image/png" })); close(); } else onError("Could not capture the photo. Retry or attach an existing image."); }, "image/png");
  };
  return <><button type="button" aria-label="Take picture" title="Take picture" disabled={disabled} onClick={() => void open()}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true"><path d="M3 6h4l2-3h6l2 3h4v15H3Z"/><circle cx="12" cy="13" r="4"/></svg></button>{stream && createPortal(<dialog ref={dialog} aria-label="Camera" onCancel={close} style={{ width: "min(70vw, 720px)", height: "70vh", background: "var(--bg, #222)", padding: 12 }}><video ref={video} autoPlay playsInline muted style={{ width: "100%", height: "80%", objectFit: "contain" }} /><button type="button" onClick={capture}>Use photo</button><button type="button" onClick={close}>Cancel</button></dialog>, document.body)}</>;
}

export function ChatDock({
  initialPrompt = "",
  docId,
  recoveryWorkspace = "window",
  getContext,
  applyAgentChange,
  contextLabel,
  contextControls,
  realtime,
  collapsed = false,
  onToggleCollapsed,
  onAgentFinished,
  onOpenSession,
}: ChatDockProps) {
  const [hydrated, setHydrated] = useState(false);
  const [turns, setTurns] = useState<AgentTurn[]>([]);
  const recovery = useResponseRecovery(docId, recoveryWorkspace, initialPrompt, turns, hydrated);
  const tip = recovery.state.tip;
  const setTip = recovery.select;
  const prompt = recovery.text;
  const setPrompt = recovery.edit;
  const [attachments, setAttachments] = useState<NonNullable<AgentEditorContext["attachments"]>>([]);
  const [dictating, setDictating] = useState(false);
  const dictatingRef = useRef(false);
  const cancelDictation = useRef<(() => void) | null>(null);
  const [dictationGeneration, setDictationGeneration] = useState(0);
  const dictatingChanged = useCallback((active: boolean) => { dictatingRef.current = active; setDictating(active); }, []);
  const [preparingAttachments, setPreparingAttachments] = useState(false);
  const attachmentPicker = useRef<HTMLInputElement | null>(null);
  const attachmentEpoch = useRef(0);
  const attachmentBusy = useRef(false);
  useEffect(() => { attachmentEpoch.current++; attachmentBusy.current = false; setPreparingAttachments(false); setAttachments([]); return () => { attachmentEpoch.current++; }; }, [docId]);
  const [running, setRunning] = useState<string | null>(null);
  // A stop was asked for and its terminal frame has not arrived yet. The
  // button stays pressed-looking rather than clickable twice.
  const [stopping, setStopping] = useState(false);
  const [stream, setStream] = useState("");
  const [reasoning, setReasoning] = useState("");
  const [showTree, setShowTree] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [unavailable, setUnavailable] = useState(false);
  const [totals, setTotals] = useState<AgentTotals | null>(null);
  // The model control. Provider is hydrated once from the server's resolved
  // default; the model input stays empty (placeholder = the provider's
  // default) unless the session already chose one explicitly.
  const [showCollapsedLineage, setShowCollapsedLineage] = useState(loadCollapsedLineage);
  const [backend, setBackend] = useState(preferredAgent);
  const [sending, setSending] = useState(false);
  const [experimentFeatures, setExperimentFeatures] = useState<string[]>([]);
  const [activeExperimentFeatures, setActiveExperimentFeatures] = useState<string[]>([]);
  useEffect(() => { if (!running) setActiveExperimentFeatures([]); }, [running]);
  const sendingRef = useRef(false);
  const earlyEvents = useRef<RunWsMessage[]>([]);
  const [provider, setProvider] = useState("");
  const [model, setModel] = useState("");

  const selectedTurn = turns.find(t => t.id === tip);
  const selectedBackend = selectedTurn?.backend ?? (selectedTurn?.provider.startsWith("acp:") ? selectedTurn.provider.slice(4) : "python");
  useEffect(() => { if (selectedTurn && !running) setBackend(selectedBackend); }, [tip, selectedBackend, running]);
  const selectedSession = selectedTurn?.session;
  const acp = useAcp(docId, selectedTurn ? selectedBackend : backend, selectedSession, running, hydrated);
  const logRef = useRef<HTMLDivElement>(null);
  const runningRef = useRef<string | null>(null);
  runningRef.current = running;
  const onFinishedRef = useRef(onAgentFinished);
  onFinishedRef.current = onAgentFinished;
  const hydratedRef = useRef(false);

  const refresh = useCallback(
    () =>
      api.agentTurns(docId).then(
        (r) => {
          setTurns(r.turns);
          setHydrated(true);
          setTotals(r.totals);
          // Hydrate the controls once — later polls must not clobber a
          // choice being made in the select/input mid-conversation.
          if (!hydratedRef.current) {
            hydratedRef.current = true;
            setBackend(r.turns.length ? r.backend ?? "python" : preferredAgent());
            setRunning(r.turns.find(t => t.status === "running")?.id ?? null);
            setProvider(r.provider);
            if (r.model !== defaultModelFor(r.provider)) setModel(r.model);
          }
        },
        () => undefined,
      ),
    [docId],
  );

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Live agent output for the turn in flight.
  useEffect(
    () =>
      realtime.onRunEvent((msg) => {
        // The up-loop's files_changed notice rides the same channel but
        // belongs to the document session, not the agent stream.
        if (!("run_id" in msg)) return;
        if (sendingRef.current && !runningRef.current) { earlyEvents.current.push(msg); return; }
        if (msg.run_id !== runningRef.current) return;
        if ("event" in msg) {
          const e = msg.event;
          setStream((prev) => appendStream(prev, e));
          setReasoning((prev) => appendReasoning(prev, e));
          return;
        }
        setRunning(null);
        setStopping(false);
        setStream("");
        setReasoning("");
        void refresh();
        onFinishedRef.current();
      }),
    [realtime, refresh],
  );

  // The hand on the cord. The run halts at its next seam — mid-stream
  // included, which is what stops the token spend on a model looping — and
  // the terminal "stopped" frame above clears the running state.
  const stop = useCallback(async () => {
    if (!runningRef.current || stopping) return false;
    setStopping(true);
    try { await api.agentStop(docId); return true; }
    catch (reason) { setStopping(false); setError(String(reason)); return false; }
  }, [docId, stopping]);

  const branch = useMemo(() => branchOf(turns, tip), [turns, tip]);
  // The conversation's record, when a turn has been written to one. The
  // lens draws every finished turn from the file; a turn still running, or
  // one that failed before it was recorded, is drawn as a card until then.
  const sessionPath = useMemo(
    () => [...branch].reverse().find((turn) => turn.session)?.session ?? null,
    [branch],
  );
  const lensStamp = turns.map((turn) => `${turn.id}:${turn.status}`).join(",");

  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => {
      void api.agentTurns(docId).then(r => {
        const turn = r.turns.find(t => t.id === runningRef.current);
        if (turn && turn.status !== "running") {
          setRunning(null); setStopping(false); setStream(""); setReasoning("");
          void refresh(); onFinishedRef.current();
        }
      }, () => undefined);
    }, 1000);
    return () => clearInterval(timer);
  }, [running, docId, refresh]);

  const captureRequest = acp.state?.edits?.captureRequest;
  useEffect(() => {
    if (!running || !captureRequest || !getContext) return;
    let live = true;
    void getContext().then(async context => { if (live) await acpApi.edits(docId, { capture_request: captureRequest, context }, backend); })
      .catch(e => setError(String(e)));
    return () => { live = false; };
  }, [running, captureRequest, docId, backend]);

  const addFiles = async (files: File[]) => {
    if (!files.length || sendingRef.current || running || attachmentBusy.current) return;
    if (files.length + attachments.length > 8) { setError("Attach at most eight files per message. Remove some files and retry."); return; }
    const epoch = attachmentEpoch.current;
    attachmentBusy.current = true; setPreparingAttachments(true);
    try {
      const additions = await Promise.all(files.map(prepareChatFile));
      if (epoch !== attachmentEpoch.current) return;
      const next = [...attachments, ...additions];
      if (next.length > 8 || next.reduce((sum, a) => sum + a.data.length, 0) > 524288) throw Error("Attachments exceed 384 KiB per message or eight files. Remove files or choose smaller ones.");
      setAttachments(next); setError(null);
    } catch (error) { if (epoch === attachmentEpoch.current) setError(`Could not prepare attachments: ${error}. Try a smaller file or another image format.`); }
    finally { if (epoch === attachmentEpoch.current) { attachmentBusy.current = false; setPreparingAttachments(false); } }
  };
  const send = async (investigation?: string) => {
    const submitted = recovery.snapshot();
    const text = (investigation ?? submitted.text).trim() || (attachments.length ? "Please inspect the attached files." : "");
    if (preparingAttachments || dictatingRef.current) return;
    if (attachments.length && backend !== "python") { setError("Choose Hickory Python to send attachments."); return; }
    if (!text || running || sendingRef.current || (!acp.state?.ready || acp.busy)) {
      if (investigation) throw new Error("The selected conversation is busy or unavailable.");
      return;
    }
    setError(null);
    setNote(null);
    const slash = backend !== "python" && !acp.state?.canRewind && text.startsWith("/rewind") ? null : parseSlash(text);
    if (backend !== "python" && !acp.state?.canRewind && text.startsWith("/rewind")) { setNote("This agent continues from its latest turn. Use New thread to start another conversation."); return; }
    if (slash) {
      setPrompt("");
      switch (slash.kind) {
        case "rewind": {
          const target = rewindFrom(turns, tip, slash.steps);
          setTip(target);
          setNote(
            target
              ? `Tip is now "${turns.find((t) => t.id === target)?.prompt.slice(0, 60) ?? target}" — the next message continues from there.`
              : "Tip is the root — the next message starts a new thread.",
          );
          return;
        }
        case "rerun": {
          // A re-run is not a branch of this conversation: it is a SECOND
          // conversation, in its own session file, that stands without this
          // one. So the tip goes to nothing — the next message opens the new
          // session — and what carries across is whatever you distil by hand.
          setTip(null);
          setNote(
            "Re-run: the next message starts a SECOND conversation, in its own " +
              "session file. This one becomes a draft you may discard — sessions/ " +
              "is gitignored, so that is already the default — and nothing in the " +
              "new one will refer to it. Distil what you learned first: " +
              "`hick ingest --from carry <session>` writes the opening prompt and leaves the " +
              "tests you kept and the approaches you ruled out for you to fill in. " +
              "To keep the change of mind instead, rewind.",
          );
          return;
        }
        case "tree":
          setShowTree((v) => !v);
          return;
        case "new":
          setTip(null);
          setNote("New thread: the next message continues from nothing.");
          return;
        case "help":
          setNote(SLASH_HELP);
          return;
      }
    }
    setStream("");
    setReasoning("");
    setSending(true); sendingRef.current = true; earlyEvents.current = [];
    try {
      // The visible tip is the parent: rewinding is just selecting an earlier
      // turn before sending. The model control rides along exactly as shown:
      // an emptied model input clears back to the provider's default.
      const requestArgs: Parameters<typeof api.agent> = [
        docId, text, tip, provider || undefined,
        provider ? model.trim() : undefined, backend,
      ];
      if (getContext) requestArgs[6] = await getContext();
      if (attachments.length) requestArgs[6] = { ...(requestArgs[6] ?? { buffers: [] }), attachments };
      if (backend === "codex" && experimentFeatures.length) requestArgs[7] = { features: experimentFeatures };
      const { session_id } = await api.agent(...requestArgs);
      setActiveExperimentFeatures(backend === "codex" ? experimentFeatures : []);
      setExperimentFeatures([]);
      setAttachments([]);
      setRunning(session_id);
      runningRef.current = session_id;
      recovery.accepted(investigation ? { ...submitted, revision: -1 } : submitted, session_id);
      // Show the new turn immediately rather than waiting for the round trip.
      setTurns((prev) => [
        ...prev,
        {
          id: session_id,
          parent_id: tip,
          prompt: text,
          answer: null,
          status: "running",
          error: null,
          created_at: new Date().toISOString(),
          backend,
          provider: (backend === "python") ? provider || "anthropic" : `acp:${backend}`,
          model: (backend === "python") ? model.trim() || defaultModelFor(provider) || "claude-sonnet-5" : "",
          usage: null,
          session: tip ? turns.find((t) => t.id === tip)?.session : undefined,
        },
      ]);
      const buffered = earlyEvents.current.filter(m => "run_id" in m && m.run_id === session_id);
      for (const msg of buffered) {
        if ("event" in msg) { setStream(prev => appendStream(prev, msg.event)); setReasoning(prev => appendReasoning(prev, msg.event)); }
        else if ("status" in msg) { setRunning(null); void refresh(); onFinishedRef.current(); }
      }
      return session_id;
    } catch (e) {
      if (investigation) throw e;
      const message = e instanceof Error ? e.message : String(e);
      // The local server answers the agent route with a 503 whose message
      // starts "agent not available" when no provider key is in the
      // environment (serve/agent.rs::start_turn); that is a configuration
      // fact, not a failure to report as a red error.
      if (!provider.endsWith("-subscription") && /agent not (configured|available)/i.test(message))
        setUnavailable(true);
      else setError(message);
    } finally { sendingRef.current = false; setSending(false); earlyEvents.current = []; }
  };

  const conversationActions = useConversationActions({ doc: docId, draft: prompt, hasAttachments: attachments.length > 0, tip, turns, running, sending, stopping,
    ready: !!acp.state?.ready && !acp.busy && !preparingAttachments && !dictating, tree: showTree, collapsed,
    evidence: { stream, reasoning, totals, note, error, unavailable, recovery_error: recovery.error, backend, provider, model, context: contextLabel ?? null, session_path: sessionPath },
    setDraft: setPrompt, discard: () => { cancelDictation.current?.(); setDictationGeneration(value => value + 1); attachmentEpoch.current++; attachmentBusy.current = false; setPreparingAttachments(false); setAttachments([]); recovery.discard(); }, setTip, setTree: setShowTree,
    send: () => send(), stop, toggleCollapsed: onToggleCollapsed,
  });
  const investigationSend = useRef(send);
  investigationSend.current = send;
  useEffect(() => registerInvestigationConversation({
    id: docId,
    label: `${contextLabel ?? docId} · ${selectedSession ?? "new conversation"} · branch ${tip ?? "root"}`,
    send: async (text) => {
      const turn = await investigationSend.current(text);
      if (!turn) throw new Error("Investigation was not started.");
      return turn;
    },
  }), [docId, contextLabel, selectedSession, tip]);

  const statusNode = useRef<HTMLElement>(null);
  useAgentStatus({ identity: agentIdentity(backend, PROVIDERS.find(item => item.id === provider)?.label ?? provider, model.trim() || defaultModelFor(provider), acp.state, acp.agents.find(agent => agent.id === backend)?.name),
    owner: `${docId}:${selectedSession ?? "new"}:${tip ?? "root"}`, backend, provider: backend === "python" ? provider : "", node: statusNode, agents: acp.agents,
    details: totals && (hasUsage(totals) || totals.subscription) ? `${statsLine(totals)}\n${statsTooltip(totals)}` : undefined }, !collapsed);
  return (
    <section ref={statusNode}
      className={`chat-dock${collapsed ? " collapsed" : ""}${onToggleCollapsed ? "" : " chat-dock--pane"}`}
      aria-label="Agent chat"
      data-coach-target="agent"
    >
      {!collapsed && backend === "python" && <ContextUpdates key={running ?? tip ?? "root"} doc={docId} branch={running ?? tip} getContext={getContext} />}
      <header className="chat-head">
        {onToggleCollapsed ? (
          <button
            className="chat-toggle"
            aria-expanded={!collapsed}
            onClick={() => void conversationActions.invoke("collapse")}
            data-tip={collapsed ? "Show conversation" : "Hide conversation"}
          >
            <span aria-hidden="true">{collapsed ? "▴" : "▾"}</span> Agent
          </button>
        ) : (
          <span className="chat-toggle chat-toggle--static">Agent</span>
        )}
        {tip && (
          <button
            className="btn-link chat-new"
            disabled={running !== null || sending}
            onClick={() => void conversationActions.invoke("new")}
            data-tip="Start a conversation that does not continue from any existing turn"
          >
            New thread
          </button>
        )}
        <MoreActions label="Conversation actions">
          {turns.length > 0 && <span className="muted">{branch.length} of {turns.length} turns on this branch</span>}
        {turns.length > 0 && (
          <button
            className={`btn-link chat-tree-toggle${showTree ? " chat-tree-toggle--on" : ""}`}
            onClick={() => void conversationActions.invoke("tree", { open: !showTree })}
            aria-pressed={showTree}
            data-tip="Zoom out: every turn as a node, branches where you rewound (/tree)"
          >
            Tree
          </button>
        )}
        </MoreActions>
        {!collapsed && <FeatureSettings label="Agent settings" owner={docId}>
        {backend === "python" && <ConversationModelChoice scope={`${docId}:${tip ?? "root"}:${selectedSession ?? "new"}`} provider={provider} model={model} disabled={running !== null || sending} providers={PROVIDERS} defaultModel={defaultModelFor(provider)} onProvider={next => { setProvider(next); setModel(""); setUnavailable(false); setError(null); }} onModel={setModel} />}
      <AgentPicker scope={`${docId}:${tip ?? "root"}:${selectedSession ?? "new"}`} backend={backend} agents={acp.agents} disabled={running !== null || sending} detecting={acp.detecting} error={acp.catalogueError}
        onRefresh={() => acp.refreshAgents()} onChange={next => {
          rememberAgent(next); setBackend(next); setTip(null); setUnavailable(false); setError(null); setNote(null);
        }} />
        <AcpControls doc={docId} backend={backend} running={running !== null} control={acp} settingsOnly />
        {contextControls}
      {!collapsed && <FeatureExperiments doc={docId} backend={backend} running={running !== null || sending || acp.busy} features={experimentFeatures} activeFeatures={activeExperimentFeatures} onFeatures={setExperimentFeatures} onUpdate={async () => { const result = await acp.install(); if (result.error) throw new Error(result.error); if (result.superseded) throw new Error("Agent selection changed; inspect the installed adapter before retrying"); }} />}
        <label><UiCheckbox label="conversation collapsed lineage" checked={showCollapsedLineage} onCheckedChange={checked => {
          setShowCollapsedLineage(checked); saveCollapsedLineage(checked);
        }} />Show lineage for collapsed conversation items</label>
        </FeatureSettings>}
      </header>
      {!collapsed && <AcpControls doc={docId} backend={backend} running={running !== null} control={acp} settingsOpen={false} showTools={false} />}


      {!collapsed && (
        <div className="chat-log" ref={logRef}>
          <AgentChanges doc={docId} state={acp.state} running={running !== null} apply={applyAgentChange} update={acp.setState} />
          {showTree ? (
            <ChatTree
              turns={turns}
              tip={tip}
              onSelect={id => { if (!running && !sending) void conversationActions.invoke("tip", { id }); }}
            />
          ) : (
            <>
              <ConversationEditor cancelDictation={cancelDictation} dictationScope={`${docId}:${tip ?? "root"}:${dictationGeneration}`} onDictatingChange={dictatingChanged} hasAttachments={attachments.length > 0} onFiles={files => void addFiles(files)} composerTools={<>
                <button type="button" aria-label="Attach files" title="Attach files" disabled={preparingAttachments || sending || !!running} onClick={() => attachmentPicker.current?.click()}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true"><path d="m8 12 7-7a4 4 0 0 1 6 6L10 22a6 6 0 0 1-8-8L13 3M5 17l11-11"/></svg></button><input ref={attachmentPicker} aria-label="Attach files" type="file" multiple hidden disabled={preparingAttachments || sending || !!running} onChange={event => { void addFiles(Array.from(event.target.files ?? [])); event.target.value = ""; }} />
                <CameraCapture key={docId} disabled={preparingAttachments || sending || !!running} onFile={file => void addFiles([file])} onError={setError} />
                {preparingAttachments && <span role="status">Preparing attachments…</span>}
                {attachments.map((attachment, index) => <span key={index}>{attachment.media_type === "image/png" && <img alt={attachment.name} src={`data:image/png;base64,${attachment.data}`} width={48} height={48} style={{ objectFit: "contain" }} />} {attachment.name} <button type="button" aria-label={`Remove ${attachment.name}`} disabled={sending || preparingAttachments} onClick={() => setAttachments(items => items.filter((_, i) => i !== index))}>×</button> </span>)}
              </>} renderFinal={turn => <AgentProject doc={docId} turn={turn.id} getContext={getContext} applyChange={applyAgentChange} />} branch={branch} path={sessionPath} stamp={lensStamp} running={running}
                showCollapsedLineage={showCollapsedLineage} stream={stream} reasoning={reasoning} draft={prompt} onDraft={setPrompt}
                onSend={() => void conversationActions.invoke("send")} sending={sending} canSend={!running && !sending && !preparingAttachments && !dictating && (!!acp.state?.ready && !acp.busy)} />
              {branch.length > 0 && <details className="chat-history"><summary>Conversation history</summary><div className="chat-turn-strip" aria-label="Turns">
                {branch.map(turn => <span key={turn.id} className="chat-turn-strip__turn">
                  <span className="muted">{turn.prompt.slice(0, 40)}</span>
                  {(backend === "python" || acp.state?.canRewind) && turn.id !== tip &&
                    <button className="btn-link chat-rewind" disabled={!!running || sending} onClick={() => void conversationActions.invoke("tip", { id: turn.id })} data-tip={REWIND_TIP}>rewind here</button>}
                </span>)}
                {sessionPath && onOpenSession && <button className="btn-link" onClick={() => onOpenSession(sessionPath)}>session</button>}
              </div></details>}
            </>
          )}
        </div>
      )}

      <div className="chat-composer">
        {running ? (
          // The way out of a runaway turn — a model looping mid-stream bills
          // tokens until somebody pulls this. Never disabled while a run is
          // live; "Stopping…" only means the request is in flight.
          <button
            className="btn chat-stop"
            disabled={stopping}
            onClick={() => void conversationActions.invoke("stop")}
            aria-label="Stop the agent"
            data-tip="Stop this turn now — the stream is cut and no more tokens are spent"
          >
            <StopMark /> {stopping ? "Stopping…" : "Stop"}
          </button>
        ) : (collapsed || showTree) ? (
          <button
            className="btn btn-primary"
            disabled={!prompt.trim() || sending || (!acp.state?.ready || acp.busy)}
            onClick={() => void conversationActions.invoke("send")}
          >
            {sending ? "Starting…" : "Send"}
          </button>
        ) : null}
      </div>
      {prompt && <button className="btn-link" onClick={() => void conversationActions.invoke("discard")}>Discard response</button>}
      {recovery.error && <p className="chat-note error" role="status">{recovery.error}</p>}
      {unavailable && (
        <p className="chat-note muted">
          {provider.endsWith("-subscription") ? "Open Agent settings, sign in with your subscription account, then retry. No API billing fallback was attempted." : <>The agent needs a provider API key. Add one in <a href="#/settings">Settings</a>, then try again.</>}
        </p>
      )}
      {note && <p className="chat-note muted">{note}</p>}
      {error && <p className="chat-note error">{error}</p>}
    </section>
  );
}
