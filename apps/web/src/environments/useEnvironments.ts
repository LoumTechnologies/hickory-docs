import { useCallback, useEffect, useRef, useState } from "react";
import { environmentApi, type EnvironmentFinding, type EnvironmentStatus } from "../api/environments";
import { showTerminalRequest } from "../lib/revealLine";

export function useEnvironments() {
  const [status, setStatus] = useState<EnvironmentStatus>({ findings: [], running: {} });
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());
  const live = useRef(false);
  const checking = useRef(false);
  const refresh = useCallback(async (force = false) => {
    if (checking.current) return;
    checking.current = true;
    try {
      const result = await environmentApi.inspect(force);
      if (live.current) { setStatus(result); setError(null); }
    } catch (e) {
      if (live.current) setError(e instanceof Error ? e.message : String(e));
    } finally { checking.current = false; }
  }, []);
  useEffect(() => {
    live.current = true;
    void refresh();
    const timer = window.setInterval(() => { if (!document.hidden) void refresh(); }, 15000);
    const focus = () => void refresh(true);
    // File writes arrive in bursts; coalesce them before requesting a check.
    let debounce: ReturnType<typeof setTimeout> | undefined;
    const files = () => {
      clearTimeout(debounce);
      debounce = setTimeout(() => void refresh(true), 1000);
    };
    window.addEventListener("focus", focus);
    window.addEventListener("hickory:files-changed", files);
    return () => {
      live.current = false;
      clearTimeout(debounce);
      window.clearInterval(timer);
      window.removeEventListener("focus", focus);
      window.removeEventListener("hickory:files-changed", files);
    };
  }, [refresh]);
  const running = Object.keys(status.running).length > 0;
  useEffect(() => {
    if (!running) return;
    const timer = window.setInterval(() => void refresh(), 1000);
    return () => window.clearInterval(timer);
  }, [running, refresh]);
  const act = async (finding: EnvironmentFinding, action: string) => {
    setPending(finding.project);
    try {
      const session = await environmentApi.act(finding, action);
      showTerminalRequest(session.id, session.title);
      if (live.current) setStatus((s) => ({ ...s, running: { ...s.running, [finding.project]: session.id } }));
      void refresh();
    } catch (e) { if (live.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (live.current) setPending(null); }
  };
  return {
    ...status, error, pending, refresh, act,
    notices: status.findings.filter((f) => f.state !== "ready" && f.state !== "unknown" && !dismissed.has(f.revision)),
    dismiss: (f: EnvironmentFinding) => setDismissed((s) => new Set([...s, f.revision])),
    choose: async (finding: EnvironmentFinding, choice: string) => {
      try { await environmentApi.choose(finding, choice); await refresh(true); }
      catch (e) { if (live.current) setError(e instanceof Error ? e.message : String(e)); }
    },
  };
}
export type Environments = ReturnType<typeof useEnvironments>;
