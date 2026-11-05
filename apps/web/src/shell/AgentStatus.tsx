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
