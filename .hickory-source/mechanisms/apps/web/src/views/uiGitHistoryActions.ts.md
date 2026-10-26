<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiGitHistoryActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bea72417f873"></a>
## `apps/web/src/views/uiGitHistoryActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-bea72417f873-useGitHistoryActions"></a>
### `useGitHistoryActions`

<storycode:copy id="f-bea72417f873-useGitHistoryActions">
import { useId } from "react";
import type { GitLog } from "../api/types";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";

/** Only summaries and explicitly expanded details belong to this reading. */
export function useGitHistoryActions(state: {
  log: GitLog | null; error: string | null; open: ReadonlySet<string>; limit: number;
  toggle: (sha: string) => void; readCommit: (sha: string) => void;
  openFile?: (path: string) => void; story?: () => void; literate?: () => void;
  more: () => void; refresh: () => void;
}) {
  const parent = `git-history:${useId()}`;
  const revision = useUiRevision(() => [state.log, state.error, [...state.open], state.limit]);
  const visible = !!state.log?.repository && !state.error && state.log.commits.length > 0;
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute } });
  const done = () => ({ status: "completed" as const, value: { navigation_requested: true } });
  add("read", "Read Git history bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const commits = visible ? state.log!.commits.map(c => ({ sha: c.sha, short: c.short, parents: c.parents, author: c.author, time: c.time, subject: c.subject, refs: c.refs, added: c.added, removed: c.removed, file_count: c.files.length, draft: c.draft ?? false, expanded: state.open.has(c.sha), ...(state.open.has(c.sha) ? { body: c.body, files: c.files } : {}) })) : [];
    const bytes = new TextEncoder().encode(JSON.stringify({ repository: state.log?.repository ?? null, loading: !state.log && !state.error, error: state.error, commits, floor: visible ? state.log!.floor : null, more_available: visible && commits.length >= state.limit, scope: "Loaded history; collapsed commit bodies and files require expansion. Not a current file edit baseline." }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    return offset > bytes.length ? { status: "unavailable", reason: "Offset outside current history" } : { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("refresh", "Refresh Git history", {}, async () => { state.refresh(); return { status: "completed", value: { refresh_requested: true } }; });
  if (visible) {
    for (const c of state.log!.commits) {
      add(`toggle:${c.sha}`, `${state.open.has(c.sha) ? "Collapse" : "Expand"} commit ${c.short}`, {}, async () => { state.toggle(c.sha); return done(); });
      if (!state.open.has(c.sha)) continue;
      add(`commit:${c.sha}`, `Read commit ${c.short}`, {}, async () => { state.readCommit(c.sha); return done(); });
      if (state.openFile) for (const [i, file] of c.files.entries()) add(`file:${c.sha}:${i}`, `Open current file ${file.path} from commit ${c.short}`, {}, async () => { state.openFile!(file.path); return done(); });
    }
    if (state.story) add("story", "Read Git history as a story", {}, async () => { state.story!(); return done(); });
    if (state.literate) add("literate", "Open Git literate views", {}, async () => { state.literate!(); return done(); });
    if (state.log!.commits.length >= state.limit) add("more", "Show older commits", {}, async () => { state.more(); return { status: "completed", value: { load_requested: true } }; });
  }
  const invoke = useUiCommands(commands);
  return (key: string) => invoke(`${parent}:${key}`);
}
</storycode:copy>

### Assemble `apps/web/src/views/uiGitHistoryActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiGitHistoryActions.ts"><storycode:paste select="#f-bea72417f873-useGitHistoryActions"/></storycode:file>

</storycode:doc>
