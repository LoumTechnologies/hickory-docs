<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/GithubTreeControls.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `83769ea34569ce1724d24438b374dd84f390fd1d52acbd2d90139d3969bb8416`.

<book:entry path="apps/web/src/shell/GithubTreeControls.tsx">
import { useEffect, useId, useRef, useState } from "react";
import { api } from "../api/client";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";

export function GithubNotificationRead({ thread, onChanged }: { thread: string; onChanged: () => void }) {
  const parent = `github-notification:${useId()}`;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const live = useRef<string | null>(thread); live.current = thread;
  useEffect(() => { live.current = thread; return () => { live.current = null; }; }, []);
  const revision = useUiRevision(() => [thread, pending.current, error]);
  const invoke = useUiCommands([{ key: parent, command: { parent, subject: parent,
    label: `Mark GitHub notification ${thread} read`, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !pending.current, value: { thread, busy, error } }),
    execute: async (_args, _actor, signal) => {
      if (pending.current || signal.aborted || live.current !== thread) return { status: "unavailable", reason: "Notification changed or is busy" };
      pending.current = true; setBusy(true); setError(null);
      try {
        const response = await api.markGithubNotificationRead(thread);
        if (live.current === thread) onChanged();
        return { status: "completed", value: { thread, response } };
      } catch (cause) {
        const reason = cause instanceof Error ? cause.message : String(cause);
        if (live.current === thread) setError(reason);
        return { status: "unavailable", reason };
      } finally { pending.current = false; if (live.current === thread) setBusy(false); }
    },
  } }]);
  useUiReading(`GitHub notification ${thread}`, { thread, busy, error });
  return <li role="treeitem" className="github-tree__mark-read"><button type="button" disabled={busy}
    onClick={() => void invoke(parent, {}).catch(cause => setError(String(cause)))}>Mark notification read</button>{error && <span>{error}</span>}</li>;
}

export function GithubTreeLink({ url, label }: { url?: string | null; label: string }) {
  const parent = `github-tree-link:${useId()}`;
  let safe: string | null = null;
  try {
    const parsed = new URL(url ?? "");
    if (parsed.protocol === "https:" && parsed.hostname === "github.com" && !parsed.port && !parsed.username && !parsed.password) safe = parsed.href;
  } catch { /* Missing or malformed links stay readable, without navigation. */ }
  const revision = useUiRevision(() => [safe, label]);
  const open = async () => {
    if (!safe) return { status: "unavailable" as const, reason: "Provider did not supply a valid HTTPS GitHub link" };
    try {
      const handle = window.open(safe, "_blank", "noopener,noreferrer");
      return { status: "completed" as const, value: { url: safe, navigation_requested: true, tab_handle_returned: !!handle,
        scope: "Browser navigation requested; browser/OS may require human interaction, and page loading is not witnessed" } };
    } catch (cause) { return { status: "unavailable" as const, reason: String(cause) }; }
  };
  useUiCommands([{ key: parent, command: { parent, subject: parent, label: `Open GitHub link: ${label}`,
    parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!safe,
      reason: safe ? undefined : "Provider did not supply a valid HTTPS GitHub link", value: { url: safe } }), execute: open } }]);
  return safe ? <a href={safe} target="_blank" rel="noopener noreferrer"
    onClick={event => { event.preventDefault(); void open(); }}>{label}</a> : <span>{label} (link unavailable)</span>;
}
</book:entry>

</book:doc>
