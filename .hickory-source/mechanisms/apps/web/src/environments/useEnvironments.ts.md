<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/environments/useEnvironments.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fc03d5c53bd5"></a>
## `apps/web/src/environments/useEnvironments.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md).

<a id="f-fc03d5c53bd5-useEnvironments"></a>
### `useEnvironments`

<storycode:copy id="f-fc03d5c53bd5-useEnvironments">
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
      return { session };
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      if (live.current) setError(error);
      return { error };
    }
    finally { if (live.current) setPending(null); }
  };
  return {
    ...status, error, pending, refresh, act,
    notices: status.findings.filter((f) => f.state !== "ready" && f.state !== "unknown" && !dismissed.has(f.revision)),
    dismiss: (f: EnvironmentFinding) => setDismissed((s) => new Set([...s, f.revision])),
    choose: async (finding: EnvironmentFinding, choice: string) => {
      try { await environmentApi.choose(finding, choice); await refresh(true); return { ok: true }; }
      catch (e) { const error = e instanceof Error ? e.message : String(e); if (live.current) setError(error); return { ok: false, error }; }
    },
  };
}
</storycode:copy>
<a id="f-fc03d5c53bd5-Environments"></a>
### `Environments`

<storycode:copy id="f-fc03d5c53bd5-Environments">
export type Environments = ReturnType<typeof useEnvironments>;
</storycode:copy>

### Assemble `apps/web/src/environments/useEnvironments.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/useEnvironments.ts"><storycode:paste select="#f-fc03d5c53bd5-useEnvironments"/><storycode:paste select="#f-fc03d5c53bd5-Environments"/></storycode:file>

</storycode:doc>
