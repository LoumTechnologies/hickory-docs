<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/GithubTreeControls.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-61fcadacf5b1"></a>
## `apps/web/src/shell/GithubTreeControls.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-61fcadacf5b1-GithubNotificationRead"></a>
### `GithubNotificationRead`

<storycode:copy id="f-61fcadacf5b1-GithubNotificationRead">
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

</storycode:copy>
<a id="f-61fcadacf5b1-GithubTreeLink"></a>
### `GithubTreeLink`

<storycode:copy id="f-61fcadacf5b1-GithubTreeLink">
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
</storycode:copy>

### Assemble `apps/web/src/shell/GithubTreeControls.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GithubTreeControls.tsx"><storycode:paste select="#f-61fcadacf5b1-GithubNotificationRead"/><storycode:paste select="#f-61fcadacf5b1-GithubTreeLink"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-66d5cea236ab"></a>
## `apps/web/src/shell/GithubTreeControls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is An Editor Buffer](../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md).

<a id="f-66d5cea236ab-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-66d5cea236ab-expression_statement">
// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { GithubTreeLink } from "./GithubTreeControls";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
</storycode:copy>
<a id="f-66d5cea236ab-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-66d5cea236ab-expression_statement-2">
it("refuses provider links outside HTTPS GitHub for both actors", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const open = vi.spyOn(window, "open").mockReturnValue(null);
  render(<UiActionContext.Provider value={registry}><GithubTreeLink url="javascript:alert(1)" label="Unsafe provider link" /></UiActionContext.Provider>);
  expect(screen.queryByRole("link")).toBeNull();
  const action = registry.groups().groups.flatMap(group => registry.observe(group).actions)[0];
  expect(action.enabled).toBe(false);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
    handle: action.handle, arguments: {}, request_id: "unsafe",
  })).toMatchObject({ status: "disabled" });
  expect(open).not.toHaveBeenCalled(); open.mockRestore();
});
</storycode:copy>

### Assemble `apps/web/src/shell/GithubTreeControls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GithubTreeControls.test.tsx"><storycode:paste select="#f-66d5cea236ab-expression_statement"/><storycode:paste select="#f-66d5cea236ab-expression_statement-2"/></storycode:file>


</storycode:doc>
