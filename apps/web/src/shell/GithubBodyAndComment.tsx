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
