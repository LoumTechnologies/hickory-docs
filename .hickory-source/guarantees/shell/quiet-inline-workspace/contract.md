<quiet:doc xmlns:quiet="http://www.hickorydocs.com/1.0">
# Keep the workspace quiet until a task needs more

For: people opening, editing, running and debugging files without learning a new interface first.

The editor keeps familiar branch, problem and caret status. Connection legends describe only painted connections; display preferences remain reachable from the file and command search. Agent status belongs to the visible conversation. Optional tools load when requested, and panels close without replacing the editor. Environment repairs disclose their commands before execution. Missing account telemetry remains unknown rather than becoming an invented allowance.

This requirement supersedes earlier always-visible provenance toggle and workspace-tool placement requirements. Existing execution, source-revision, permission and write-review boundaries still apply. Visual feel and language-specific installation success require live acceptance; focused witnesses below establish only the behavior they exercise.

## Show file text before optional capabilities

File display is eventually consistent: mount available source or disk text without
waiting for rendered blocks, owner sessions, provenance or language services.
Attach late capabilities to the same editor. Loading or failure of an optional
capability must not erase readable content, selection, scrolling or unsaved work.
Generated disk readings remain read-only until provenance can route edits back to
the source; existing permission, revision and save acknowledgement rules apply.

The November 5 source audit covered file routing, sessions, plain/generated
editors, language attachment, folder refresh, restoration and lazy workspace tools.
The maintained implementations below and their linked mechanisms remove source
render gates and generated-session gates, display generated disk contents during
weave loading/failure, reconfigure late output extensions, and publish background
provenance incrementally with one request at a time. Remaining restoration,
folder refresh, repeated evaluation and document LSP findings are tracked in
[issue 117](https://github.com/LoumTechnologies/hickory-docs/issues/117).
This is not a claim that every panel or installed release has measured fast loads.

Focused witnesses live in `workspaceTabs.test.tsx`, `shell/views.test.tsx`,
`documentSaveRender.test.tsx` and `OutputEditorPane.test.tsx`: pending blocks,
missing owner sessions, stalled/failed provenance, write refusal before provenance,
late capability attachment to the same view, and sequential background publishing.
The save/render witness waits for runnable controls after save acknowledgement;
acknowledgement alone does not imply asynchronous rail layout has finished.
Existing source-receipt rejection checks remain in `useDocumentRender.test.tsx`.

## Provider and runtime evidence limits

Codex quota reads use the native App Server's documented account/rateLimits/read method. Claude Code documents rate_limits in status-line input, but the installed ACP connection does not currently forward that input; Grok's checked public CLI reference does not establish a structured subscription quota read. Their detected accounts remain visible in account details with a link to native usage, without fabricated percentages or reset counts. This is an integration limit, not evidence of unlimited usage. Model effort and speed appear only when reported by the selected adapter.

DeepSeek timing follows the [published Beijing schedule](https://api-docs.deepseek.com/zh-cn/quick_start/pricing/) and the [2026 State Council holiday notice](https://www.beijing.gov.cn/cs/gncs/zcwj/202603/t20260327_4568275.html), checked November 4, 2026. Weekends stay off-peak even on makeup working days. Outside the checked calendar the UI reports the schedule unavailable and links to current pricing. It never queues requests or changes billing automatically.

## Focused verification — November 4, 2026

Executed on this checkout:

- 78 focused frontend checks passed across quiet-workspace controls, status controls, status counts, dependency disclosure, agent behavior and merge behavior. Node 26 requires `NODE_OPTIONS=--no-experimental-webstorage` for the existing jsdom setup.
- The focused blank-window application witness passed, including New File navigation; unrelated application tests were not executed.
- The native private-window safety witness passed and refuses both new picker endpoints for a private attachment.
- `python3 scripts/test-account-usage.py target/debug/examples/python_agent_ui_fixture` passed through real HTTP using the explicit disposable CLI fixture. This proves protocol handling and side-effect limits, not live account availability.
- `just test-quiet-ui` passed through a real Chromium browser and native disposable workspace: initial footer, typing, connection-settings discovery/dismissal, contextual agent identity, keyboard account disclosure and hidden feature measurement. It also caught and now guards the merge-editor import-order startup failure. The single development sample observed the first editable view in 3,406 ms and insertion/observation in 38 ms; no release performance or comparative speed claim follows.
- TypeScript checking, the production web build and the Linux desktop `cargo check` passed. The desktop check required building the embedded web assets first.

The source/output check and ownership audit passed before this evidence-only addition and are repeated after regeneration. No broad regression suite or post-commit status monitoring was run. Native OS picker dialogs, signed installers, live paid account quotas and zero-configuration language setup were not exercised here. Claude/Grok quota integration is tracked in [issue 116](https://github.com/LoumTechnologies/hickory-docs/issues/116).

## apps/web/src/shell/WorkspaceTools.tsx

Heavy review, coverage and import interfaces belong to a requested task. This module is imported only after a person opens a file tool or a command. Closing the panel returns focus without navigating away or replacing an editor. Each underlying tool retains its existing preview, permission and source-revision checks.

<a id="output-quiet-deceb2806831"></a>
<quiet:copy id="quiet-deceb2806831">
import { lazy, useEffect, useRef } from "react";
import type { EditorView } from "@codemirror/view";
import { LineageSettings } from "../components/LineageSettings";
import { ProvenanceToggles } from "./ProvenanceToggles";
import type { ComponentProps } from "react";
import { useUiCommands } from "../lib/uiActionBridge";
import "./quietWorkspace.css";

const CoverageTools = lazy(() => import("../coverage/CoverageTools").then(module => ({ default: module.CoverageTools })));
const NativeRuntimeHistory = lazy(() => import("../debug/RuntimeRuns").then(module => ({ default: module.NativeRuntimeHistory })));
const ReviewPanel = lazy(() => import("../components/ReviewPanel").then(module => ({ default: module.ReviewPanel })));
const MeetingImport = lazy(() => import("../components/MeetingImport").then(module => ({ default: module.MeetingImport })));
const SnapshotExport = lazy(() => import("../components/SnapshotExport").then(module => ({ default: module.SnapshotExport })));
const WorkflowTour = lazy(() => import("../components/WorkflowTour").then(module => ({ default: module.WorkflowTour })));

export type WorkspaceTool = "connections" | "execution" | "review" | "import" | "export" | "help";
export const TOOL_LABELS: Record<WorkspaceTool, string> = {
  connections: "Connection display", execution: "Tests and execution", review: "Review this file",
  import: "Import meeting", export: "Export document snapshot", help: "Workflow help",
};

export function WorkspaceTools({ tool, path, document, source, view, provenance, projectId, onGraph, onOpen, onOpenNote, close }: {
  tool: WorkspaceTool; path: string | null; document?: string; source?: string; view?: EditorView | null;
  provenance: ComponentProps<typeof ProvenanceToggles>; projectId: string | null;
  onGraph: () => void; onOpen: (path: string, line?: number) => void; onOpenNote: (id: string) => void; close: () => void;
}) {
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    const previous = window.document.activeElement as HTMLElement | null;
    panel.current?.querySelector<HTMLButtonElement>("button")?.focus();
    return () => { if (previous?.isConnected) previous.focus(); };
  }, []);
  const invoke = useUiCommands([{ key: "workspace-tool:graph", command: { label: "Show document connections graph", parent: "workspace-tools", subject: "workspace-tool", parameters: {}, prerequisites: [], read: () => ({ revision: projectId ?? "", enabled: !!projectId && tool === "connections" }), execute: async () => { onGraph(); return { status: "completed", value: { graph_requested: true } }; } } }, { key: "workspace-tool:close", command: { label: "Close workspace tool", subject: "workspace-tool", parent: "workspace-tools", parameters: {}, prerequisites: [], read: () => ({ revision: tool, enabled: true }), execute: async () => { close(); return { status: "completed", value: null }; } } }]);
  return <aside ref={panel} className="workspace-tool-panel" role="region" aria-label={TOOL_LABELS[tool]} onKeyDown={event => {
    if (event.key === "Escape" && !(event.target as HTMLElement).closest("input,textarea,[contenteditable=true]")) { event.stopPropagation(); close(); }
  }}>
    <header><strong>{TOOL_LABELS[tool]}</strong>{path && <small title={path}>{path}</small>}<button type="button" aria-label="Close workspace tool" onClick={close}>Close</button></header>
    {tool === "connections" && <><ProvenanceToggles {...provenance} /><LineageSettings />{projectId && <button type="button" onClick={() => { void invoke("workspace-tool:graph"); }}>Show document connections graph</button>}</>}
    {tool === "execution" && <><CoverageTools path={path} /><NativeRuntimeHistory path={path ?? undefined} source={source} view={view} /></>}
    {tool === "review" && <ReviewPanel document={path ?? undefined} source={source} view={view} initialOpen />}
    {tool === "import" && <MeetingImport onOpen={onOpen} onOpenNote={onOpenNote} />}
    {tool === "export" && <SnapshotExport docId={document} />}
    {tool === "help" && <WorkflowTour surface="app" onOpen={onOpen} focused={{ path: path ?? undefined, source, view }} />}
  </aside>;
}
</quiet:copy>

<quiet:file path="apps/web/src/shell/WorkspaceTools.tsx"><quiet:paste select="#quiet-deceb2806831" /></quiet:file>

## apps/web/src/shell/quietWorkspace.css

Panels overlay the editor only while requested. Agent summaries truncate at narrow widths while their keyboard-accessible disclosure retains the full identity and account details. No animation or layout-wide transition is needed.

<a id="output-quiet-a891e30d12d9"></a>
<quiet:copy id="quiet-a891e30d12d9">
.workspace-tool-panel { position: fixed; z-index: 60; right: 12px; bottom: 30px; width: min(560px, calc(100vw - 24px)); max-height: 65vh; overflow: auto; padding: 12px; background: var(--bg); color: var(--fg); border: 1px solid var(--border); box-shadow: 0 4px 20px #0003; }
.workspace-tool-panel > header { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.workspace-tool-panel > header small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.workspace-tool-panel > header button { margin-left: auto; }
.status-bar__right .status-bar__path { min-width: 0; max-width: 20vw; flex-shrink: 2; }
.agent-status { display: inline-flex; gap: 6px; align-items: center; min-width: 0; }
.agent-status > summary { cursor: pointer; list-style: none; max-width: 42vw; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.agent-status__details { position: fixed; z-index: 65; right: 12px; bottom: 30px; max-width: min(480px, 90vw); max-height: 60vh; overflow: auto; padding: 12px; background: var(--bg); color: var(--fg); border: 1px solid var(--border); box-shadow: 0 4px 20px #0003; }
.agent-status__details p { margin: 4px 0; }
.environment-notice > details:not([open]) { font-size: 12px; }
@media (max-width: 800px) { .agent-status > summary { max-width: 30vw; } }
</quiet:copy>

<quiet:file path="apps/web/src/shell/quietWorkspace.css"><quiet:paste select="#quiet-a891e30d12d9" /></quiet:file>

## apps/web/src/shell/ReviewEvents.tsx

Gutter review requests must keep working when their panel is no longer permanently mounted. Retain only the small event listener at startup. After the lazy review component has installed its listeners, replay the captured request once; subsequent requests go directly to the mounted reviewer.

<a id="output-quiet-16d67cad5b64"></a>
<quiet:copy id="quiet-16d67cad5b64">
import { lazy, Suspense, useEffect, useState, useRef } from "react";
import { REVIEW_TARGET, REVIEW_ERROR } from "../lib/reviews";
const Review = lazy(() => import("../components/ReviewPanel").then(module => ({ default: module.ReviewPanel })));
export function ReviewEvents() {
  const [pending, setPending] = useState<{ type: string; detail: unknown } | null>(null);
  const [loaded, setLoaded] = useState(false);
  const replayed = useRef(false);
  useEffect(() => {
    if (loaded) return;
    const open = (event: Event) => { if (!replayed.current) setPending({ type: event.type, detail: (event as CustomEvent).detail }); };
    window.addEventListener(REVIEW_TARGET, open); window.addEventListener(REVIEW_ERROR, open);
    return () => { window.removeEventListener(REVIEW_TARGET, open); window.removeEventListener(REVIEW_ERROR, open); };
  }, [loaded]);
  return pending ? <Suspense fallback={<span role="status">Opening review…</span>}><Review initialOpen /><Replay pending={pending} onReady={() => { replayed.current = true; setLoaded(true); }} /></Suspense> : null;
}
function Replay({ pending, onReady }: { pending: { type: string; detail: unknown }; onReady: () => void }) {
  useEffect(() => { onReady(); window.dispatchEvent(new CustomEvent(pending.type, { detail: pending.detail })); }, [pending]);
  return null;
}
</quiet:copy>

<quiet:file path="apps/web/src/shell/ReviewEvents.tsx"><quiet:paste select="#quiet-16d67cad5b64" /></quiet:file>

## apps/web/src/lib/agentStatus.ts

Normalize provider-reported quota windows before rendering them. Expired, malformed and absent windows stay unknown; the provider's duration supplies the label, so a future policy change cannot silently become a five-hour allowance. Model identity comes from reported configuration, while API providers retain their explicitly selected identity.

DeepSeek timing is a published schedule estimate, separate from account telemetry and the bill. Beijing dates determine holidays and weekends. The checked calendar covers 2026 only; outside that calendar the UI links to pricing instead of pretending to know the next holiday. Countdown tests pin boundary instants, including China dates that differ from UTC.

<a id="output-quiet-2c85bd61cdf0"></a>
<quiet:copy id="quiet-2c85bd61cdf0">
import type { AcpState } from "../api/acp";

export interface QuotaWindow { label: string; used: number; resetsAt: number }
export interface AccountUsage { windows: QuotaWindow[]; resets: number | null; checkedAt: number; error?: string }

/** Reject expired or malformed windows; unknown is never zero used. */
export function codexUsage(raw: unknown, now = Date.now()): AccountUsage {
  const data = raw as { rateLimits?: { primary?: unknown; secondary?: unknown }; rateLimitsByLimitId?: Record<string, { primary?: unknown; secondary?: unknown }>; rateLimitResetCredits?: { availableCount?: unknown } } | null;
  const buckets = data?.rateLimitsByLimitId;
  const limits = buckets?.codex ?? data?.rateLimits;
  const windows = [limits?.primary, limits?.secondary].flatMap(value => {
    const v = value as { usedPercent?: unknown; windowDurationMins?: unknown; resetsAt?: unknown } | null;
    if (!v || typeof v.usedPercent !== "number" || !Number.isFinite(v.usedPercent) || v.usedPercent < 0 || v.usedPercent > 100 || typeof v.resetsAt !== "number" || !Number.isFinite(v.resetsAt) || v.resetsAt * 1000 <= now || typeof v.windowDurationMins !== "number" || v.windowDurationMins <= 0) return [];
    const minutes = v.windowDurationMins;
    return [{ label: minutes === 10080 ? "Week" : minutes === 300 ? "5h" : `${minutes}m`, used: v.usedPercent, resetsAt: v.resetsAt }];
  });
  const count = data?.rateLimitResetCredits?.availableCount;
  return { windows, resets: typeof count === "number" && Number.isSafeInteger(count) && count >= 0 ? count : null, checkedAt: now };
}

/** Read reported choices by category/id, never guess effort from a model name. */
export function agentIdentity(backend: string, provider: string, model: string, state: AcpState | null | undefined, name?: string): string {
  if (backend === "python") return ["Hickory agent", provider || "Provider not reported", model || "Model not reported"].join(" · ");
  const options = state?.configOptions ?? [];
  const choice = (kinds: string[]) => options.find(option => kinds.includes(option.category ?? "") || kinds.includes(option.id));
  const modelChoice = choice(["model"]);
  const effort = choice(["thought_level", "reasoning_effort", "effort"]);
  const speed = choice(["speed", "service_tier", "fast_mode"]);
  const label = (option: typeof modelChoice) => {
    const values = option?.options?.flatMap(item => "value" in item ? [item] : item.options);
    return values?.find(item => item.value === option?.currentValue)?.name ?? String(option?.currentValue);
  };
  return [`${name ?? backend} via ACP`, modelChoice ? label(modelChoice) : state?.models?.currentModelId || "Model not reported",
    effort && label(effort), speed && (speed.type === "boolean" ? speed.currentValue ? "Fast" : "Default speed" : label(speed))].filter(Boolean).join(" · ");
}

// DeepSeek's published schedule, checked 2026-11-04. Beijing has no DST.
// https://api-docs.deepseek.com/zh-cn/quick_start/pricing/
// Holiday ranges: State Council notice 国办发明电〔2025〕7号.
const HOLIDAYS_2026 = [["01-01", "01-03"], ["02-15", "02-23"], ["04-04", "04-06"], ["05-01", "05-05"], ["06-19", "06-21"], ["09-25", "09-27"], ["10-01", "10-07"]];
function peakAt(time: number): boolean | null {
  const beijing = new Date(time + 8 * 3600000);
  if (beijing.getUTCFullYear() !== 2026) return null;
  const day = beijing.getUTCDay(), date = beijing.toISOString().slice(5, 10);
  if (day === 0 || day === 6 || HOLIDAYS_2026.some(([start, end]) => date >= start && date <= end)) return false;
  const hour = beijing.getUTCHours();
  return hour >= 9 && hour < 12 || hour >= 14 && hour < 18;
}
export function deepseekWindow(now = Date.now()): { peak: boolean; changesAt: number } | null {
  if (!Number.isFinite(now)) return null;
  const peak = peakAt(now);
  if (peak === null) return null;
  // Every change is on an hour boundary. Search through a full holiday week.
  for (let next = Math.floor(now / 3600000) * 3600000 + 3600000; next < now + 16 * 86400000; next += 3600000) {
    const state = peakAt(next);
    if (state === null) return null;
    if (state !== peak) return { peak, changesAt: next };
  }
  return null;
}
export function timeLeft(until: number, now: number): string {
  const minutes = Math.max(0, Math.ceil((until - now) / 60000));
  return minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60}m` : `${minutes}m`;
}
</quiet:copy>

<quiet:file path="apps/web/src/lib/agentStatus.ts"><quiet:paste select="#quiet-2c85bd61cdf0" /></quiet:file>

## apps/web/src/shell/AgentStatus.tsx

Only mounted, visible conversations contribute identity to the status bar. Clicking or focusing a conversation selects its status; closing it removes its entry. The store holds metadata separately from streamed answer text. Codex usage refreshes at most once a minute while a relevant conversation is open, skips background windows and discards responses after unmount.

The disclosure is keyboard-accessible and explicitly distinguishes account allowance from per-conversation token cost. It lists detected agents even when they do not report quotas. No login, credit purchase, reset redemption or inference is performed by reading status.

<a id="output-quiet-20ef37b0ae72"></a>
<quiet:copy id="quiet-20ef37b0ae72">
import { useEffect, useId, useState, useSyncExternalStore } from "react";
import type { RefObject } from "react";
import { request, MOCK } from "../api/client";
import { codexUsage, deepseekWindow, timeLeft, type AccountUsage } from "../lib/agentStatus";
import { useUiReading } from "../lib/uiReading";
import { useUiCommands } from "../lib/uiActionBridge";
import { useRef } from "react";
import "./quietWorkspace.css";

interface Entry { id: string; owner?: string; identity: string; backend: string; provider: string; node: RefObject<HTMLElement | null>; details?: string; agents: { id: string; name: string; available?: boolean }[] }
let entries: Entry[] = [];
const listeners = new Set<() => void>();
const subscribe = (callback: () => void) => { listeners.add(callback); return () => { listeners.delete(callback); }; };
const snapshot = () => entries;
function changed() { listeners.forEach(callback => callback()); }

/** Mounted conversations publish metadata only; streaming tokens never enter this store. */
export function useAgentStatus(value: Omit<Entry, "id">, enabled: boolean) {
  const id = useId();
  const signature = JSON.stringify([value.owner, value.identity, value.backend, value.provider, value.details, value.agents]);
  useEffect(() => {
    if (!enabled) return;
    entries = [...entries.filter(entry => entry.id !== id), { ...value, id }]; changed();
    return () => { entries = entries.filter(entry => entry.id !== id); changed(); };
  }, [id, enabled, signature]);
}
function visible(entry: Entry) {
  const node = entry.node.current;
  return !!node?.isConnected && !node.closest("[hidden],[aria-hidden=true]");
}

/** Fetch only while the status for an open conversation is visible; never sign in. */
function useCodexUsage(enabled: boolean, revision: number) {
  const [usage, setUsage] = useState<AccountUsage | null>(null);
  useEffect(() => {
    setUsage(null);
    if (!enabled || MOCK) return;
    let live = true, busy = false;
    const refresh = async () => {
      if (document.hidden || busy) return;
      busy = true;
      try {
        const result = await request<unknown>("GET", "/api/agents/codex-subscription/usage");
        if (live) setUsage(codexUsage(result));
      } catch { if (live) setUsage({ windows: [], resets: null, checkedAt: Date.now(), error: "Usage is not reported. Check native Codex sign-in or update Codex, then reopen account details." }); }
      finally { busy = false; }
    };
    void refresh();
    const timer = window.setInterval(() => { void refresh(); }, 60000);
    const focus = () => { void refresh(); };
    window.addEventListener("focus", focus);
    return () => { live = false; clearInterval(timer); window.removeEventListener("focus", focus); };
  }, [enabled, revision]);
  return usage;
}

export function AgentStatusBar() {
  const all = useSyncExternalStore(subscribe, snapshot);
  const disclosure = useRef<HTMLDetailsElement>(null);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [usageRevision, setUsageRevision] = useState(0);
  const [active, setActive] = useState<string | null>(null);
  const [clock, setClock] = useState(Date.now);

  useEffect(() => {
    if (!all.length) return;
    const choose = (event: Event) => {
      const entry = all.find(entry => entry.node.current?.contains(event.target as Node));
      if (entry) setActive(entry.id);
    };
    const timer = window.setInterval(() => { if (!document.hidden) setClock(Date.now()); }, 30000);
    document.addEventListener("focusin", choose); document.addEventListener("pointerdown", choose);
    // Re-evaluate mounted panes when the shell changes tab visibility.
    const observer = new MutationObserver(() => setClock(Date.now()));
    observer.observe(document.body, { subtree: true, attributes: true, attributeFilter: ["hidden", "aria-hidden"] });
    return () => { clearInterval(timer); observer.disconnect(); document.removeEventListener("focusin", choose); document.removeEventListener("pointerdown", choose); };
  }, [all]);
  const available = all.filter(visible);
  const entry = available.find(item => item.id === active) ?? available[available.length - 1];
  const detectedCodex = entry?.agents.some(agent => agent.id === "codex" && agent.available);
  const usage = useCodexUsage(!!entry && (entry.backend === "codex" || entry.provider === "codex-subscription" || !!detectedCodex), usageRevision);
  const windows = usage?.windows.filter(window => window.resetsAt * 1000 > clock) ?? [];
  const deepseek = entry?.provider === "deepseek" ? deepseekWindow(clock) : null;
  const quota = entry && (entry.backend === "codex" || entry.provider === "codex-subscription") ? windows.map(window => `${window.label} ${Math.floor(100 - window.used)}% left`).join(" · ") : "";
  const discount = entry?.provider === "deepseek" ? deepseek ? `Off-peak ${deepseek.peak ? "starts" : "ends"} in ${timeLeft(deepseek.changesAt, clock)}` : "DeepSeek pricing schedule unavailable" : "";
  useUiReading("visible agent status", { identity: entry?.identity ?? null, quota, discount, usage: usage ?? null });
  const visibilityActions = useUiCommands(entry ? [{ key: "agent-status:visibility", command: {
    label: "Set agent account details visibility", parent: "agent-status", subject: entry.id, parameters: { open: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: `${entry.id}:${entry.owner ?? ""}:${detailsOpen}`, enabled: visible(entry), value: detailsOpen }),
    execute: async args => { if (disclosure.current) disclosure.current.open = Boolean(args.open); setDetailsOpen(Boolean(args.open)); return { status: "completed", value: { open: Boolean(args.open) } }; },
  } }] : []);
  const invoke = useUiCommands(entry && detailsOpen ? [{ key: "agent-status:refresh", command: {
    label: "Refresh agent account usage", parent: "agent-status", subject: entry.id, parameters: {}, prerequisites: [],
    read: () => ({ revision: `${entry.id}:${entry.owner ?? ""}:${usageRevision}`, enabled: !!detectedCodex || entry.backend === "codex" || entry.provider === "codex-subscription" }),
    execute: async () => { setUsageRevision(value => value + 1); return { status: "completed", value: { refresh_requested: true } }; },
  } },{ key: "agent-status:settings", command: {
    label: "Change visible agent settings", parent: "agent-status", subject: entry.id, parameters: {}, prerequisites: [],
    read: () => ({ revision: `${entry.id}:${entry.owner ?? ""}:${entry.identity}`, enabled: visible(entry) }),
    execute: async () => { if (disclosure.current) disclosure.current.open = false; entry.node.current?.querySelector<HTMLButtonElement>('button[aria-label="Agent settings"]')?.click(); return { status: "completed", value: { settings_requested: true } }; },
  } }] : []);
  if (!entry) return null;
  return <details ref={disclosure} className="agent-status" onToggle={event => setDetailsOpen(event.currentTarget.open)} onKeyDown={event => {
    if (event.key === "Escape") { event.currentTarget.open = false; event.currentTarget.querySelector("summary")?.focus(); }
  }}>
    <summary className="status-bar__item" onClick={event => { event.preventDefault(); void visibilityActions("agent-status:visibility", { open: !detailsOpen }); }} title={[entry.identity, quota, discount, usage?.resets == null ? "Resets not reported" : `${usage.resets} resets available`, ...windows.map(window => `${window.label} resets ${new Date(window.resetsAt * 1000).toLocaleString()}`)].filter(Boolean).join(" · ")}>{[entry.identity, quota, discount].filter(Boolean).join(" · ")}</summary>
    <section className="agent-status__details" aria-label="Agent account details">
      <strong>{entry.identity}</strong>
      <p>Describes the last selected visible Agent pane.</p>
      {(entry.details || "Effort and speed are shown when reported by the agent.").split("\n").map((line, index) => <p key={index}>{line}</p>)}
      {windows.map(window => <p key={window.label}>Codex {window.label}: {Math.floor(100 - window.used)}% left · resets {new Date(window.resetsAt * 1000).toLocaleString()}</p>)}
      {usage && <p>{usage.resets === null ? "Codex resets available: not reported" : `Codex resets available: ${usage.resets}`} · checked {new Date(usage.checkedAt).toLocaleTimeString()}</p>}
      {usage?.error && <p>{usage.error}</p>}
      {(detectedCodex || entry.backend === "codex" || entry.provider === "codex-subscription") && <button type="button" onClick={() => { void invoke("agent-status:refresh"); }}>Refresh account usage</button>}
      {entry.agents.filter(agent => agent.available).map(agent => <p key={agent.id}>{agent.name}: {agent.id === "codex" && windows.length ? "limits above" : "account limits not reported by this connection"}</p>)}
      {entry.agents.some(agent => agent.available && agent.id === "claude") && <p><a href="https://claude.ai/settings/usage" target="_blank" rel="noreferrer">Check Claude account usage</a></p>}
      {entry.agents.some(agent => agent.available && agent.id === "grok") && <p><a href="https://grok.com/dashboard" target="_blank" rel="noreferrer">Check Grok account usage</a></p>}
      <p>Quota is account-wide and can change in other sessions. Session tokens and cost are separate.</p>
      {entry.provider === "deepseek" && <p>{discount}. Schedule estimate checked November 4, 2026; <a href="https://api-docs.deepseek.com/quick_start/pricing/" target="_blank" rel="noreferrer">check current DeepSeek pricing</a>.</p>}
      <button type="button" onClick={() => { void invoke("agent-status:settings"); }}>Change agent settings</button>
    </section>
  </details>;
}
</quiet:copy>

<quiet:file path="apps/web/src/shell/AgentStatus.tsx"><quiet:paste select="#quiet-20ef37b0ae72" /></quiet:file>

## apps/web/src/shell/quietWorkspace.test.tsx

These focused witnesses protect the quiet/default and revealed/detail boundary. They cover actual control clicks, agent-pane retirement, quota expiration and schedule boundaries. They do not measure startup latency, validate a paid account, or claim every language's tools install successfully.

<a id="output-quiet-af84974b9e11"></a>
<quiet:copy id="quiet-af84974b9e11">
// Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md
import { useRef } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { codexUsage, agentIdentity, deepseekWindow } from "../lib/agentStatus";
import { ProvenanceToggles } from "./ProvenanceToggles";
import { WorkspaceStatusExtras } from "../environments/WorkspaceStatusExtras";
import { AgentStatusBar, useAgentStatus } from "./AgentStatus";
import { BlankWelcome } from "../views/BlankWelcome";
import { FileToolMenu } from "./FileToolMenu";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import type { ProvenanceLayer } from "../lib/provenanceLayers";
import { request } from "../api/client";
import { MENU_EVENT } from "../lib/menuBridge";
vi.mock("../api/client", () => ({ request: vi.fn(), MOCK: false }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it("labels only painted families and leaves disabled preferences reachable separately", () => {
  const props = { layers: new Set(["lineage", "context"] as const), onToggle: vi.fn() };
  const ui = render(<ProvenanceToggles {...props} displayed={new Set(["context"])} />);
  expect(screen.queryByText("Lineage")).toBeNull();
  expect(screen.getByText("Context").tagName).toBe("SPAN");
  expect(screen.queryByRole("button")).toBeNull();
  ui.rerender(<ProvenanceToggles {...props} displayed={new Set()} />);
  expect(screen.queryByText("Context")).toBeNull();
  ui.rerender(<ProvenanceToggles {...props} />);
  fireEvent.click(screen.getByRole("button", { name: "Declared" }));
  expect(props.onToggle).toHaveBeenCalledWith("declared");
});

it("a healthy workspace has no environment or graph chrome", () => {
  const onEnvironments = vi.fn();
  const props = { provenance: { layers: new Set<ProvenanceLayer>(), displayed: new Set<ProvenanceLayer>(), onToggle: vi.fn() }, environmentCount: 0, onEnvironments };
  const ui = render(<WorkspaceStatusExtras {...props} />);
  expect(screen.queryByRole("button")).toBeNull();
  ui.rerender(<WorkspaceStatusExtras {...props} environmentCount={2} />);
  fireEvent.click(screen.getByRole("button", { name: "2 dependency problems" }));
  expect(onEnvironments).toHaveBeenCalledTimes(1);
});

it("preserves quota durations, real zeroes and unknown reset counts", () => {
  const now = Date.parse("2026-11-04T20:00:00Z"), future = now / 1000 + 100;
  const data = { rateLimits: { primary: { usedPercent: 25, windowDurationMins: 300, resetsAt: future }, secondary: { usedPercent: 0, windowDurationMins: 10080, resetsAt: future } }, rateLimitResetCredits: { availableCount: 0 } };
  expect(codexUsage(data, now)).toMatchObject({ windows: [{ label: "5h", used: 25 }, { label: "Week", used: 0 }], resets: 0 });
  expect(codexUsage(null, now)).toMatchObject({ windows: [], resets: null });
  expect(codexUsage({ rateLimits: { primary: { usedPercent: NaN, windowDurationMins: 300, resetsAt: future } } }, now).windows).toEqual([]);
  expect(codexUsage(data, future * 1000).windows).toEqual([]);
  expect(codexUsage({ rateLimits: { primary: { usedPercent: 10, windowDurationMins: 60, resetsAt: future } } }, now).windows[0].label).toBe("60m");
});

it("uses reported model, effort and speed, never model-name guesses", () => {
  expect(agentIdentity("codex", "", "ignored", { backend: "codex", ready: true, configOptions: [
    { id: "model", name: "Model", type: "select", currentValue: "m" },
    { id: "reasoning_effort", name: "Effort", type: "select", currentValue: "high" },
    { id: "fast_mode", name: "Speed", type: "boolean", currentValue: false },
  ] }, "Codex")).toBe("Codex via ACP · m · high · Default speed");
  expect(agentIdentity("claude", "", "ignored", null)).toBe("claude via ACP · Model not reported");
});

it("counts to the actual next DeepSeek transition across lunch, weekends and China holidays", () => {
  const at = (time: string) => deepseekWindow(Date.parse(time));
  expect(at("2026-11-04T00:59:59Z")).toEqual({ peak: false, changesAt: Date.parse("2026-11-04T01:00:00Z") });
  expect(at("2026-11-04T01:00:00Z")).toEqual({ peak: true, changesAt: Date.parse("2026-11-04T04:00:00Z") });
  expect(at("2026-11-04T04:00:00Z")).toEqual({ peak: false, changesAt: Date.parse("2026-11-04T06:00:00Z") });
  expect(at("2026-11-06T10:00:00Z")).toEqual({ peak: false, changesAt: Date.parse("2026-11-09T01:00:00Z") });
  expect(at("2026-10-01T01:00:00Z")).toEqual({ peak: false, changesAt: Date.parse("2026-10-08T01:00:00Z") });
  // A makeup working Saturday is still off-peak under the vendor's weekend rule.
  expect(at("2026-10-10T01:00:00Z")?.peak).toBe(false);
  expect(at("2027-01-01T01:00:00Z")).toBeNull();
});

function Conversation({ name, hidden = false, owner = name }: { name: string; hidden?: boolean; owner?: string }) {
  const node = useRef<HTMLElement>(null);
  useAgentStatus({ owner, identity: `${name} via ACP · model`, backend: name, provider: "", node, agents: [] }, true);
  return <section ref={node} hidden={hidden}><button>{name} conversation</button></section>;
}
it("follows the selected visible Agent pane and removes status when its owner closes", async () => {
  const ui = render(<><Conversation name="Claude" /><Conversation name="Grok" /><AgentStatusBar /></>);
  fireEvent.pointerDown(screen.getByRole("button", { name: "Claude conversation" }));
  expect(screen.getByText("Claude via ACP · model", { selector: "summary" })).toBeTruthy();
  ui.rerender(<><Conversation name="Claude" hidden /><Conversation name="Grok" /><AgentStatusBar /></>);
  await waitFor(() => expect(screen.getByText("Grok via ACP · model", { selector: "summary" })).toBeTruthy());
  ui.rerender(<AgentStatusBar />);
  expect(screen.queryByText(/via ACP/)).toBeNull();
  expect(request).not.toHaveBeenCalled();
});

it("retires status settings handles when the conversation owner changes with the same model", async () => {
  const registry = new UiActions({ workspace: "test", window: "test" }, () => true, async () => {});
  const ui = (owner: string) => <UiActionContext.Provider value={registry}><Conversation name="Claude" owner={owner} /><AgentStatusBar /></UiActionContext.Provider>;
  const rendered = render(ui("first-document"));
  await act(async () => fireEvent.click(screen.getByText("Claude via ACP · model", { selector: "summary" })));
  const settings = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Change visible agent settings");
  await waitFor(() => expect(settings()).toBeTruthy());
  const old = settings()!;
  rendered.rerender(ui("second-document"));
  expect(await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: old.handle, arguments: {}, request_id: "retired-owner" })).toMatchObject({ status: "stale" });
});

it("discloses file tools on request and refuses handles bound to a previous file", async () => {
  const registry = new UiActions({ workspace: "test", window: "test" }, () => true, async () => {});
  const open = vi.fn();
  const ui = (name: string) => <UiActionContext.Provider value={registry}><FileToolMenu name={name} document readRevision={() => name} onOpen={open} /></UiActionContext.Provider>;
  const rendered = render(ui("a.md"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  expect(actions().some(action => action.label === "Review this file")).toBe(false);
  await act(async () => fireEvent.click(screen.getByLabelText("Tools for a.md")));
  const review = actions().find(action => action.label === "Review this file")!;
  await act(async () => { await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: review.handle, arguments: {}, request_id: "review-a" }); });
  expect(open).toHaveBeenCalledWith("review");
  rendered.rerender(ui("b.md"));
  expect(await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: review.handle, arguments: {}, request_id: "old-a" })).toMatchObject({ status: "stale" });
  await act(async () => fireEvent.click(screen.getByLabelText("Tools for b.md")));
  expect(actions().some(action => action.label === "Review this file")).toBe(false);
});

it("requests real native pickers and creates a new unsaved file from the blank window", async () => {
  vi.mocked(request).mockResolvedValue({ native: true });
  const menu = vi.fn(); window.addEventListener(MENU_EVENT, menu);
  try {
    render(<BlankWelcome />);
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Open File…" })));
    expect(request).toHaveBeenCalledWith("POST", "/api/window/open-file", {});
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Open Folder…" })));
    expect(request).toHaveBeenLastCalledWith("POST", "/api/window/open-folder", {});
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "New File" })));
    expect((menu.mock.calls[0][0] as CustomEvent).detail).toBe("new");
  } finally { window.removeEventListener(MENU_EVENT, menu); }
});
</quiet:copy>

<quiet:file path="apps/web/src/shell/quietWorkspace.test.tsx"><quiet:paste select="#quiet-af84974b9e11" /></quiet:file>

## scripts/test-quiet-workspace.py

This system witness drives the same browser controls a person uses, in Python rather than the frontend's implementation language. It verifies a quiet initial footer, typing, connection-setting discovery, panel dismissal and contextual agent status without a paid request. The two recorded timings are a single development sample, not performance guarantees. Run only against the disposable fixture because the test types into its unsaved buffer.

The `--links` mode isolates the link-reading interaction in a disposable workspace.
It uses Python Playwright and public HTTP, so changing the editor framework does
not change its assertions. External browser launches are checked at the request
boundary; actual host browser installation remains a separate acceptance check.

<a id="output-quiet-c2657fb100b9"></a>
<quiet:copy id="quiet-c2657fb100b9">
#!/usr/bin/env python3
"""Focused real-browser witness. Run against a disposable workspace URL.

uv run --with playwright==1.63.0 python scripts/test-quiet-workspace.py URL
Uses the existing Playwright Chromium installation; no provider requests.
"""
import json
import sys
import time
from playwright.sync_api import sync_playwright, expect

# Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md

def command(page, text):
    page.keyboard.press("ControlOrMeta+Shift+P")
    field = page.get_by_role("textbox", name="Commands")
    field.fill(">" + text)
    option = page.get_by_role("option").filter(has_text=text).first
    expect(option).to_be_visible()
    field.press("Enter")


# Contract: .hickory-source/guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md
def check_links(page, origin):
    """Drive only public HTTP and visible UI, independent of editor internals."""
    source = ('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
              '# Link reading\n\nSee [related note](notes/other.md) and '
              '[the web](https://example.com).\n\n'
              '@notes/other.md\n\n<hick:include file="notes/other.md" />\n</hick:doc>\n')
    response = page.request.post(origin + "/api/projects/local/docs", data={"path": "links.md", "source": source})
    assert response.ok, response.text()
    address = origin + "/#/docs/" + response.json()["id"]
    page.goto(address)
    editor = page.locator('[data-spatial-key="document:links.md"] .cm-content').first
    label = editor.locator('.cm-md-link').filter(has_text="related note")
    expect(label).to_be_visible(timeout=45000)
    expect(editor).not_to_contain_text("](notes/other.md)")
    label.click()
    expect(editor).to_contain_text("[related note](notes/other.md)")
    expect(editor).not_to_contain_text("](https://example.com)")
    editor.locator('.cm-line').filter(has_text="Link reading").click()
    expect(editor).not_to_contain_text("](notes/other.md)")
    label.click(button="right")
    for name in ["Rename…", "Move to…", "Copy to…", "Delete", "Copy relative path", "Open in default program"]:
        expect(page.get_by_role("menuitem", name=name, exact=True)).to_be_visible()
    page.keyboard.press("Escape")
    label.dblclick()
    destination = page.locator('[data-spatial-key="document:notes/other.md"]')
    expect(destination).to_have_attribute("data-spatial-center", "true")
    page.goto(address)
    launches = []
    def launch(route):
        launches.append(route.request.post_data_json)
        route.fulfill(json={"ok": True})
    # A boundary witness records the requested browser without opening external apps.
    page.route("**/api/open-external", launch)
    web = page.locator('[data-spatial-key="document:links.md"] .cm-md-link').filter(has_text="the web")
    with page.expect_response(lambda response: response.url.endswith("/api/open-external")):
        web.dblclick()
    assert {"url": "https://example.com"} in launches, launches
    web.click(button="right")
    with page.expect_response(lambda response: response.url.endswith("/api/open-external")):
        page.get_by_role("menuitem", name="Open in Firefox", exact=True).click()
    assert {"url": "https://example.com", "browser": "firefox"} in launches, launches
    editor.locator('.cm-line').filter(has_text='file="notes/other.md"').dblclick()
    expect(destination).to_have_attribute("data-spatial-center", "true")
    page.goto(address)
    at_include = editor.locator('.cm-md-link').filter(has_text="@notes/other.md")
    expect(at_include).to_be_visible()
    at_include.click(button="right")
    expect(page.get_by_role("menuitem", name="Open in default program", exact=True)).to_be_visible()
    page.keyboard.press("Escape")
    at_include.dblclick()
    expect(destination).to_have_attribute("data-spatial-center", "true")
    print(json.dumps({"result": "PASS", "scope": "link labels, selection, file actions, double-click, include, @include, browser requests"}))


def main():
    if len(sys.argv) not in (2, 3):
        raise SystemExit("Usage: test-quiet-workspace.py DISPOSABLE_WORKSPACE_URL")
    with sync_playwright() as automation:
        browser = automation.chromium.launch()
        page = browser.new_page(viewport={"width": 1200, "height": 900})
        errors = []
        page.on("pageerror", lambda error: errors.append(str(error)))
        started = time.monotonic()
        if len(sys.argv) == 3 and sys.argv[2] == "--links":
            check_links(page, sys.argv[1].rstrip("/"))
            assert not errors, errors
            browser.close()
            return
        page.goto(sys.argv[1])
        editor = page.locator(".untitled-tab .cm-content").first
        expect(editor).to_be_visible(timeout=45000)
        open_ms = round((time.monotonic() - started) * 1000)
        footer = page.get_by_role("contentinfo")
        for label in ["Graph", "Environments", "Lineage", "Context", "Declared", "Continuity", "Execution", "Workspace tools"]:
            expect(footer.get_by_text(label, exact=True)).to_have_count(0)
        editor.click()
        page.keyboard.press("ControlOrMeta+End")
        text = "Quiet workspace typing witness."
        started = time.monotonic()
        page.keyboard.insert_text(text)
        expect(editor).to_contain_text(text)
        typing_ms = round((time.monotonic() - started) * 1000)
        command(page, "Connection display")
        panel = page.get_by_role("region", name="Connection display", exact=True)
        expect(panel).to_be_visible()
        expect(panel.get_by_role("button", name="Lineage", exact=True)).to_be_visible()
        panel.get_by_role("button", name="Close workspace tool").click()
        expect(panel).to_have_count(0)
        expect(editor).to_contain_text(text)
        command(page, "Agent")
        agent = page.get_by_role("region", name="Agent chat")
        expect(agent).to_be_visible()
        expect(agent.locator(".chat-identity")).to_have_count(0)
        expect(agent.get_by_role("checkbox", name="Measure this request with and without Hickory features")).to_have_count(0)
        status = footer.locator(".agent-status > summary")
        expect(status).to_be_visible()
        status.focus()
        page.keyboard.press("Enter")
        expect(footer.get_by_role("region", name="Agent account details")).to_be_visible()
        page.keyboard.press("Escape")
        expect(footer.get_by_role("region", name="Agent account details")).not_to_be_visible()
        agent.get_by_role("button", name="Agent settings").click()
        expect(page.get_by_role("checkbox", name="Measure this request with and without Hickory features")).to_be_visible()
        page.screenshot(path="/tmp/hickory-quiet-workspace.png", full_page=True)
        assert not errors, errors
        print(json.dumps({"result": "PASS", "first_editable_view_ms": open_ms, "insert_and_observe_ms": typing_ms,
                          "scope": "fresh development fixture, one sample; no release performance claim", "screenshot": "/tmp/hickory-quiet-workspace.png"}))
        browser.close()


if __name__ == "__main__":
    main()
</quiet:copy>

<quiet:file path="scripts/test-quiet-workspace.py"><quiet:paste select="#quiet-c2657fb100b9" /></quiet:file>

## scripts/test-account-usage.py

The HTTP witness uses a deliberately scripted native CLI, solely to exercise protocol success and unsupported-method failure without real account credentials. It asserts the exact allowed method set, the reset count projection and omission of account identity, opaque reset ids and upstream error contents. This fixture is a safety witness, not evidence that a particular installed CLI or subscription supports quotas.

<a id="output-quiet-e1c536330e37"></a>
<quiet:copy id="quiet-e1c536330e37">
#!/usr/bin/env python3
"""Read account quotas through the native HTTP API using a disposable CLI fixture.

python3 scripts/test-account-usage.py target/debug/examples/python_agent_ui_fixture
No real credentials, sign-in, inference, reset redemption or credit purchases.
"""
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request

# Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md
FAKE = r'''
import json,sys
from pathlib import Path
for line in sys.stdin:
    message=json.loads(line)
    method=message.get('method')
    with (Path(__file__).parent/'methods.jsonl').open('a') as log: log.write(json.dumps(method)+'\n')
    if 'id' not in message: continue
    if method == 'initialize': result={}
    elif method == 'account/rateLimits/read':
        if (Path(__file__).parent/'unsupported').exists():
            print(json.dumps({'id':message['id'],'error':{'code':-32601,'message':'sensitive upstream error'}}),flush=True)
            continue
        result={'rateLimits':{'primary':{'usedPercent':20,'windowDurationMins':300,'resetsAt':1999999999}},
                'rateLimitResetCredits':{'availableCount':2,'credits':[{'id':'opaque-reset-secret'}]},
                'email':'private@example.invalid','credentials':'credential-secret'}
    else: raise RuntimeError('Unexpected operation: '+str(method))
    print(json.dumps({'id':message['id'],'result':result}),flush=True)
'''


def main():
    engine = Path(sys.argv[1]).resolve()
    with tempfile.TemporaryDirectory(prefix="hickory-usage-") as folder:
        root = Path(folder)
        (root / "note.md").write_text("# Account usage witness\n")
        binaries = root / "agents/node_modules/.bin"
        binaries.mkdir(parents=True)
        cli = binaries / "codex"
        cli.write_text("#!" + sys.executable + "\n" + FAKE)
        cli.chmod(0o700)
        environment = {key: value for key, value in os.environ.items() if not key.endswith("_API_KEY")}
        process = subprocess.Popen([str(engine), str(root)], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, env=environment)
        try:
            assert select.select([process.stdout], [], [], 30)[0], "Fixture did not start"
            origin = process.stdout.readline().strip()
            assert origin.startswith("http://127.0.0.1:"), origin
            url = origin + "/api/agents/codex-subscription/usage"
            with urllib.request.urlopen(url, timeout=35) as response:
                data = json.load(response)
            assert data["rateLimits"]["primary"]["usedPercent"] == 20, data
            assert data["rateLimitResetCredits"] == {"availableCount": 2}, data
            assert not any(value in json.dumps(data) for value in ["private@example", "credential-secret", "opaque-reset-secret"]), data
            (binaries / "unsupported").touch()
            try:
                urllib.request.urlopen(url, timeout=35)
                raise AssertionError("Unsupported quota RPC must not look like zero usage")
            except urllib.error.HTTPError as error:
                assert error.code == 503, error.code
                message = error.read().decode()
                assert "usage is unavailable" in message and "sensitive upstream" not in message, message
            methods = [json.loads(line) for line in (binaries / "methods.jsonl").read_text().splitlines()]
            assert set(methods) <= {"initialize", "initialized", "account/rateLimits/read"}, methods
            assert methods.count("account/rateLimits/read") == 2, methods
            print("PASS native HTTP: quota/reset count, credential redaction, unsupported RPC, no login/inference/billing/reset side effects")
        finally:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill(); process.wait()


if __name__ == "__main__":
    main()
</quiet:copy>

<quiet:file path="scripts/test-account-usage.py"><quiet:paste select="#quiet-e1c536330e37" /></quiet:file>

## apps/web/src/shell/FileToolMenu.tsx

A file's tools use its own tab and revision, not the workspace's previously focused document. Native clicks and registered actions share the same handler. Opening a panel is navigation; each tool still validates its own execution or write. The shell wraps this component in visibility scope so hidden file tools cannot be invoked by stale agent handles.

<a id="output-quiet-2a402f70d3eb"></a>
<quiet:copy id="quiet-2a402f70d3eb">
import { useId } from "react";
import { MoreActions } from "../components/MoreActions";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
type Props = {
  name: string; document: boolean; readRevision: () => unknown;
  onOpen: (tool: "connections" | "execution" | "review" | "export") => void;
};
export function FileToolMenu(props: Props) {
  return <MoreActions label={`Tools for ${props.name}`}><FileToolButtons {...props} /></MoreActions>;
}
function FileToolButtons({ name, document, readRevision, onOpen }: Props) {
  const parent = `file-tools:${useId()}`;
  const revision = useUiRevision(() => [name, document, readRevision()]);
  const actions = [ { id: "connections" as const, label: "Connection display" }, { id: "execution" as const, label: "Tests and execution" },
    { id: "review" as const, label: "Review this file" }, ...(document ? [{ id: "export" as const, label: "Export document snapshot" }] : []) ];
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    label: action.label, parent, subject: parent, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async () => { onOpen(action.id); return { status: "completed", value: { panel_requested: action.id, file: name } }; },
  } })));
  return <>{actions.map(action => <button key={action.id} type="button" onClick={() => { void invoke(`${parent}:${action.id}`); }}>{action.label}</button>)}</>;
}
</quiet:copy>

<quiet:file path="apps/web/src/shell/FileToolMenu.tsx"><quiet:paste select="#quiet-2a402f70d3eb" /></quiet:file>

</quiet:doc>
