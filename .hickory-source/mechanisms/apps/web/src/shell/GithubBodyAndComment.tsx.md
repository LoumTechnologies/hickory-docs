<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/GithubBodyAndComment.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-66578f096780"></a>
## `apps/web/src/shell/GithubBodyAndComment.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-66578f096780-GithubBodyAndComment"></a>
### `GithubBodyAndComment`

<storycode:copy id="f-66578f096780-GithubBodyAndComment">
import { useEffect, useId, useRef, useState } from "react";
import { api } from "../api/client";
import type { GithubComment } from "../api/github";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
import { useUiReading } from "../lib/uiReading";

export function GithubBodyAndComment({ kind, repository, number, body, comments, onChanged, readOnly = false }: {
  kind: "pr" | "issue"; repository: string; number: number; body: string;
  comments: GithubComment[]; readOnly?: boolean; onChanged: () => void;
}) {
  const identity = `${kind}:${repository}#${number}`;
  const parent = `github-body:${useId()}`;
  const live = useRef({ identity, body }); live.current = { identity, body };
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const [draft, setDraft] = useState({ identity, baseline: body, text: body });
  const [comment, setComment] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [receipt, setReceipt] = useState<UiValue>(null);
  useEffect(() => {
    setDraft(current => current.identity !== identity || current.text === current.baseline ? { identity, baseline: body, text: body } : current);
  }, [identity, body]);
  const changeBody = (text: string) => { if (!pending.current) setDraft(current => ({ ...current, text })); };
  const changeComment = (text: string) => { if (!pending.current) setComment(text); };
  const submit = async (field: "body" | "comment", signal: AbortSignal): Promise<UiOutcome> => {
    if (readOnly || pending.current || signal.aborted || draft.identity !== identity) return { status: "unavailable", reason: "The provider editor changed or is busy" };
    if (field === "body" && draft.baseline !== body) return { status: "requires_input", request: `${parent}:changed-body`, reason: "The provider body changed. Read it and explicitly reset or revise this retained draft before saving." };
    const value = field === "body" ? draft.text : comment;
    if (field === "comment" && !value.trim()) return { status: "unavailable", reason: "Enter a comment first" };
    pending.current = true; setBusy(true); setNotice(null);
    const capturedIdentity = identity;
    try {
      const response = field === "body" ? await api.editGithubObject(kind, repository, number, "body", value)
        : await api.commentOnGithubObject(kind, repository, number, value);
      const result = { identity: capturedIdentity, field, response: response as unknown as UiValue,
        provider_reply_received: true, independently_verified: false };
      if (mounted.current && live.current.identity === capturedIdentity) {
        setReceipt(result);
        setNotice(field === "body" ? "Body saved." : "Comment added.");
        if (field === "comment") setComment(current => current === value ? "" : current);
        else if (live.current.body === body) setDraft(current => current.text === value ? { ...current, baseline: value } : current);
        onChanged();
      }
      return { status: "completed", value: result };
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error);
      if (mounted.current && live.current.identity === capturedIdentity) {
        setNotice(reason); setReceipt({ identity: capturedIdentity, field, error: reason, effects: "Uncertain; inspect the provider before retrying" });
      }
      return { status: "unavailable", reason: `${reason}. Inspect the provider before retrying; a failed request may have applied effects.` };
    } finally {
      pending.current = false;
      if (mounted.current) setBusy(false);
    }
  };
  const revision = useUiRevision(() => [identity, body, draft, comment, pending.current, comments, receipt, readOnly]);
  const actions: { id: string; label: string; parameters?: UiCommand["parameters"]; enabled?: () => boolean; execute: UiCommand["execute"] }[] = [
    { id: "body", label: `Set GitHub body draft ${identity}`, parameters: { text: { type: "string", maxLength: 1048576 } },
      execute: async args => { changeBody(args.text as string); return { status: "completed", value: { draft_updated: true, sent: false } }; } },
    { id: "comment", label: `Set GitHub comment draft ${identity}`, parameters: { text: { type: "string", maxLength: 1048576 } },
      execute: async args => { changeComment(args.text as string); return { status: "completed", value: { draft_updated: true, sent: false } }; } },
    { id: "reset", label: `Reset GitHub body draft ${identity}`, execute: async () => {
      setDraft({ identity, baseline: body, text: body }); return { status: "completed", value: { reset_to_current_body: true, sent: false } };
    } },
    { id: "save", label: `Save GitHub body ${identity}`, enabled: () => !readOnly && draft.text !== body,
      execute: async (_args, _actor, signal) => submit("body", signal) },
    { id: "send", label: `Add GitHub comment ${identity}`, enabled: () => !readOnly && !!comment.trim(),
      execute: async (_args, _actor, signal) => submit("comment", signal) },
  ];
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters ?? {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !pending.current && draft.identity === identity && (action.enabled?.() ?? true) }), execute: action.execute,
  } })));
  const human = (id: string) => void invoke(`${parent}:${id}`, {}).catch(error => setNotice(String(error)));
  useUiReading(`GitHub body and comments ${identity}`, { identity, committed_body: body, body_draft: draft.text,
    draft_baseline_current: draft.baseline === body, comment_draft: comment, comments, busy, notice, receipt, read_only: readOnly,
    scope: "Provider-reported content and replies; no independent verification or cross-client compare-and-swap" });
  return <>
    <li role="treeitem" className="github-tree__field">
      <label>Body<textarea value={draft.text} disabled={busy} onChange={event => changeBody(event.target.value)} /></label>
      <button type="button" disabled={readOnly || busy || draft.text === body} onClick={() => human("save")}>Save body</button>
      {draft.baseline !== body && <button type="button" disabled={busy} onClick={() => human("reset")}>Reset body draft to current</button>}
    </li>
    <li role="treeitem" className="github-tree__section"><span>Comments ({comments.length})</span>
      <ul role="group" className="github-tree__comments">
        {comments.map((item, index) => <li role="treeitem" key={item.id ?? item.url ?? index}>
          <span className="github-tree__byline">{item.author?.login ?? "unknown"}</span><p>{item.body}</p>
        </li>)}
        <li role="treeitem" className="github-tree__new-comment">
          <label>New comment<textarea value={comment} disabled={busy} onChange={event => changeComment(event.target.value)} /></label>
          <button type="button" disabled={readOnly || busy || !comment.trim()} onClick={() => human("send")}>Add comment</button>
        </li>
      </ul>
    </li>
    {notice && <li role="treeitem" className="github-tree__notice">{notice}</li>}
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/shell/GithubBodyAndComment.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GithubBodyAndComment.tsx"><storycode:paste select="#f-66578f096780-GithubBodyAndComment"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-e42c6eb9a161"></a>
## `apps/web/src/shell/GithubBodyAndComment.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is An Editor Buffer](../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md).

<a id="f-e42c6eb9a161-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e42c6eb9a161-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { GithubBodyAndComment } from "./GithubBodyAndComment";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { installMockHandler } from "../api/client";
afterEach(cleanup);

</storycode:copy>
<a id="f-e42c6eb9a161-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e42c6eb9a161-expression_statement-2">
it("shares provider drafts and exact replies while preserving a changed body draft", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const calls: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { calls.push(body); return { ok: true }; });
  const changed = vi.fn();
  const mount = (body: string) => <UiActionContext.Provider value={registry}><ul><GithubBodyAndComment kind="issue" repository="acme/widget" number={7}
    body={body} comments={[]} onChanged={changed} /></ul></UiActionContext.Provider>;
  const view = render(mount("Original"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Set GitHub body draft issue:acme/widget#7", { text: "Draft" });
  expect((screen.getByLabelText("Body") as HTMLTextAreaElement).value).toBe("Draft");
  view.rerender(mount("Provider changed"));
  expect(await run("Save GitHub body issue:acme/widget#7")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(calls).toEqual([]);
  expect((screen.getByLabelText("Body") as HTMLTextAreaElement).value).toBe("Draft");
  await run("Reset GitHub body draft issue:acme/widget#7");
  fireEvent.change(screen.getByLabelText("Body"), { target: { value: "Native draft" } });
  expect(await run("Save GitHub body issue:acme/widget#7")).toMatchObject({ status: "result", outcome: { value: { response: { ok: true }, independently_verified: false } } });
  await run("Set GitHub comment draft issue:acme/widget#7", { text: "Agent draft" });
  fireEvent.click(screen.getByRole("button", { name: "Add comment" }));
  await screen.findByText("Comment added.");
  expect(calls).toEqual([
    { kind: "issue", repository: "acme/widget", number: 7, field: "body", value: "Native draft" },
    { kind: "issue", repository: "acme/widget", number: 7, body: "Agent draft" },
  ]);
  expect(changed).toHaveBeenCalledTimes(2);
});

</storycode:copy>
<a id="f-e42c6eb9a161-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e42c6eb9a161-expression_statement-3">
it("preserves a failed comment draft and excludes a second submission", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let fail!: (reason: unknown) => void;
  installMockHandler(async () => new Promise((_done, reject) => { fail = reject; }));
  render(<UiActionContext.Provider value={registry}><ul><GithubBodyAndComment kind="pr" repository="acme/widget" number={12}
    body="body" comments={[]} onChanged={() => {}} /></ul></UiActionContext.Provider>);
  fireEvent.change(screen.getByLabelText("New comment"), { target: { value: "Keep this" } });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Add GitHub comment pr:acme/widget#12")!.handle, arguments: {}, request_id: "send",
  }); });
  expect(await registry.act(registry.scope, { id: "python:other", kind: "agent" }, {
    handle: actions().find(action => action.label === "Add GitHub comment pr:acme/widget#12")!.handle, arguments: {}, request_id: "other",
  })).toMatchObject({ status: "disabled" });
  let result: unknown;
  await act(async () => { fail(Error("Provider refused")); result = await pending; });
  expect(result).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("may have applied effects") } });
  expect((screen.getByLabelText("New comment") as HTMLTextAreaElement).value).toBe("Keep this");
});
</storycode:copy>

### Assemble `apps/web/src/shell/GithubBodyAndComment.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GithubBodyAndComment.test.tsx"><storycode:paste select="#f-e42c6eb9a161-expression_statement"/><storycode:paste select="#f-e42c6eb9a161-expression_statement-2"/><storycode:paste select="#f-e42c6eb9a161-expression_statement-3"/></storycode:file>


</storycode:doc>
