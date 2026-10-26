<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/terminal/AttentionCard.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-da1d3639d06b"></a>
## `apps/web/src/terminal/AttentionCard.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Answer Does Not Cost You Your Place](../../../../../guarantees/terminal/an-answer-does-not-cost-you-your-place/contract.md), [Turbo Never Answers A Prompt It Did Not Parse](../../../../../guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse/contract.md).

<a id="f-da1d3639d06b-AttentionCard"></a>
### `AttentionCard`

The card that answers a prompt without taking you to it.
The point of the queue is that you keep working; a card that made you go to
a terminal, read it, answer, and come back would just be a fancier way of
checking on things. So the top of the queue comes to you, with the agent's
own choices on it where the program declared them.
Protects docs/guarantees/terminal/an-answer-does-not-cost-you-your-place.md

<storycode:copy id="f-da1d3639d06b-AttentionCard">
// The card that answers a prompt without taking you to it.
//
// The point of the queue is that you keep working; a card that made you go to
// a terminal, read it, answer, and come back would just be a fancier way of
// checking on things. So the top of the queue comes to you, with the agent's
// own choices on it where the program declared them.
//
// Protects docs/guarantees/terminal/an-answer-does-not-cost-you-your-place.md

import { useId, useMemo, useRef, useState } from "react";

import { useTerminalChromeActions } from "./useTerminalChromeActions";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor } from "../lib/uiActions";
import type { TerminalSession } from "../api/types";

export function AttentionCard({
  session,
  position,
  total,
  onAnswer,
  onOpen,
  onInterrupt,
  onNext,
  onDismiss,
}: {
  session: TerminalSession;
  /** 1-based place in the queue, for "2 of 5". */
  position: number;
  total: number;
  onAnswer: (send: string) => boolean | void | Promise<boolean | void>;
  onOpen: () => void;
  onInterrupt: () => boolean | void | Promise<boolean | void>;
  onNext: () => void;
  onDismiss: () => void;
}) {
  const prompt = session.prompt;
  const promptKey = JSON.stringify([session.id, session.state, session.since_ms, prompt]);
  const latestPrompt = useRef(promptKey); latestPrompt.current = promptKey;
  const request = useMemo(() => `terminal-decision:${crypto.randomUUID()}`, [promptKey]);
  const [draft, setDraft] = useState({ owner: promptKey, text: "", prepared: false });
  const typed = draft.owner === promptKey ? draft.text : "";
  const prepared = draft.owner === promptKey && draft.prepared;
  const setTyped = (text: string, actor?: UiActor) => { if (text.length <= 8192) setDraft({ owner: promptKey, text, prepared: actor?.kind === "agent" || prepared }); };
  const guessed = prompt?.source === "guessed";
  const canAnswer = !!prompt && session.state === "needs-you";
  const send = async (text: string) => {
    if (!canAnswer) return false;
    const reply = await onAnswer(text);
    if (reply !== false && latestPrompt.current === promptKey) setDraft({ owner: promptKey, text: "", prepared: false });
    return reply;
  };
  const controls = useTerminalChromeActions("terminal attention card", {
    session: session.id, title: session.title, state: session.state, branch: session.branch, dirty: session.dirty, position, total,
    question: prompt?.question ?? session.preview, prompt_source: prompt?.source ?? null,
    choices: prompt?.choices.map(choice => ({ label: choice.label, destructive: choice.destructive })) ?? [],
    draft: typed, draft_prepared_by_agent: prepared, decision_request: prompt ? request : null,
    exit_code: session.exit_code, authority: "Existing human prompt-answer policy; this is an application boundary, not an OS limitation",
  }, [
    { id: "open", label: "Open attention terminal", run: onOpen },
    { id: "next", label: "Show next terminal attention request", run: onNext },
    { id: "dismiss", label: "Dismiss terminal attention card", run: onDismiss },
    ...(session.state === "needs-you" ? [{ id: "interrupt", label: "Interrupt attention terminal", run: onInterrupt }] : []),
    ...(canAnswer ? prompt!.choices.length ? prompt!.choices.map((choice, index) => ({ id: `choice:${index}`, label: `Answer attention prompt: ${choice.label}`, humanRequest: request, run: () => send(choice.send) }))
      : [{ id: "send", label: "Send attention prompt reply", humanRequest: request, run: () => send(`${typed}\n`) }] : []),
  ]);
  const parent = `attention-draft:${useId()}`;
  const revision = useUiRevision(() => [promptKey, typed, prepared]);
  useUiCommands(canAnswer && !prompt!.choices.length ? [{ key: parent, command: { parent, subject: parent, label: "Prepare terminal attention reply draft",
    parameters: { text: { type: "string", maxLength: 8192 } }, prerequisites: ["Preparing a draft does not send an answer or approve a decision"],
    read: () => ({ revision: revision(), enabled: !controls.pending }), execute: async (args, actor) => { setTyped(args.text as string, actor); return { status: "completed", value: { draft_updated: true, answer_sent: false, approval: false } }; },
  } }] : []);

  return (
    <div className="attention-card" role="dialog" aria-label={`${session.title} needs you`}>
      <header className="attention-card-head">
        <span className={`state-dot state-${session.state}`} aria-hidden="true" />
        <strong>{session.title}</strong>
        {session.branch && <code className="attention-branch">{session.branch}</code>}
        {session.dirty && (
          <span className="attention-dirty" data-tip="This working tree has uncommitted changes">
            uncommitted
          </span>
        )}
        <span className="attention-count">
          {position} of {total}
        </span>
      </header>

      <p className="attention-question">{prompt ? prompt.question : session.preview}</p>

      {prompt && prompt.choices.length > 0 && (
        <div className="attention-choices">
          {prompt.choices.map((choice) => (
            <button
              key={choice.label}
              className={choice.destructive ? "btn btn-danger" : "btn btn-primary"}
              disabled={!canAnswer || !!controls.pending}
              onClick={() => controls.human(`choice:${prompt.choices.indexOf(choice)}`)}
            >
              {choice.label}
            </button>
          ))}
        </div>
      )}

      {prompt && prompt.choices.length === 0 && (
        <form
          className="attention-reply"
          onSubmit={(e) => {
            e.preventDefault();
            controls.human("send");
          }}
        >
          {/* No choices means nobody told us what the answers are — so the
              card offers the same thing the terminal would: a line to type.
              Saying which is which matters: an invented button on a guessed
              prompt would be a lie about what we know. */}
          <label className="muted" htmlFor={`reply-${session.id}`}>
            {guessed ? "Looks like a question — type the answer" : "Answer"}
          </label>
          <input
            id={`reply-${session.id}`}
            value={typed}
            disabled={!canAnswer || !!controls.pending}
            maxLength={8192}
            autoFocus
            onChange={(e) => setTyped(e.target.value)}
          />
          {prepared && <p role="note">Prepared by an agent. Review this reply before sending.</p>}
          <button className="btn btn-primary" type="submit" disabled={!canAnswer || !!controls.pending}>
            Send
          </button>
        </form>
      )}

      {!prompt && (
        <p className="muted">
          {session.state === "failed"
            ? `Exited ${session.exit_code ?? "with an error"}.`
            : "Finished."}
          {session.dirty && " Its working tree still has uncommitted changes."}
        </p>
      )}

      {controls.error && <p role="alert">{controls.error}</p>}
      <footer className="attention-card-foot">
        <button className="btn" disabled={!!controls.pending} onClick={() => controls.human("open")}>
          Open terminal
        </button>
        {session.state === "needs-you" && (
          <button className="btn" disabled={!!controls.pending} onClick={() => controls.human("interrupt")}>
            Interrupt
          </button>
        )}
        <button className="btn" disabled={!!controls.pending} onClick={() => controls.human("next")}>
          Next
        </button>
        <button className="btn" disabled={!!controls.pending} onClick={() => controls.human("dismiss")}>
          Later
        </button>
      </footer>
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/terminal/AttentionCard.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/AttentionCard.tsx"><storycode:paste select="#f-da1d3639d06b-AttentionCard"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b44600c6bdc4"></a>
## `apps/web/src/terminal/AttentionCard.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Answer Does Not Cost You Your Place](../../../../../guarantees/terminal/an-answer-does-not-cost-you-your-place/contract.md), [Turbo Never Answers A Prompt It Did Not Parse](../../../../../guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse/contract.md).

<a id="f-b44600c6bdc4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b44600c6bdc4-expression_statement">
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import type { TerminalSession } from "../api/types";
import { AttentionCard } from "./AttentionCard";

afterEach(cleanup);

</storycode:copy>
<a id="f-b44600c6bdc4-session"></a>
### `session`

<storycode:copy id="f-b44600c6bdc4-session">
function session(over: Partial<TerminalSession> = {}): TerminalSession {
  return {
    id: "term-1",
    title: "refactor the parser",
    cwd: "/w/app",
    monitor: false,
    state: "needs-you",
    since_ms: 0,
    branch: "side-task",
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
    ...over,
  };
}

</storycode:copy>
<a id="f-b44600c6bdc4-show"></a>
### `show`

<storycode:copy id="f-b44600c6bdc4-show">
function show(over: Partial<TerminalSession> = {}) {
  const onAnswer = vi.fn();
  render(
    <AttentionCard
      session={session(over)}
      position={1}
      total={3}
      onAnswer={onAnswer}
      onOpen={vi.fn()}
      onInterrupt={vi.fn()}
      onNext={vi.fn()}
      onDismiss={vi.fn()}
    />,
  );
  return onAnswer;
}

</storycode:copy>
<a id="f-b44600c6bdc4-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b44600c6bdc4-expression_statement-2">
describe("the attention card", () => {
  it("offers the program's own choices, and sends exactly what they carry", () => {
    const onAnswer = show({
      prompt: {
        question: "May I write src/main.rs?",
        source: "declared",
        choices: [
          { label: "Allow", send: "1\n", destructive: false },
          { label: "Deny", send: "2\n", destructive: false },
        ],
      },
    });

    expect(screen.getByText("May I write src/main.rs?")).toBeTruthy();
    fireEvent.click(screen.getByText("Allow"));
    expect(onAnswer).toHaveBeenCalledWith("1\n");
  });

  it("never invents buttons for a prompt it only guessed at", () => {
    // We recognised the shape of a question. We do not know what the answers
    // mean, so the card offers a line to type — and says why.
    show({
      prompt: { question: "Delete build/? [y/N]", source: "guessed", choices: [] },
    });

    expect(screen.queryByText("Allow")).toBeNull();
    expect(screen.getByLabelText(/type the answer/i)).toBeTruthy();
  });

  it("marks a destructive choice apart from a safe one", () => {
    show({
      prompt: {
        question: "Force push?",
        source: "declared",
        choices: [
          { label: "Cancel", send: "n\n", destructive: false },
          { label: "Force push", send: "y\n", destructive: true },
        ],
      },
    });
    expect(screen.getByText("Force push").className).toContain("btn-danger");
    expect(screen.getByText("Cancel").className).not.toContain("btn-danger");
  });

  it("says what is left to decide when a finished session still has changes", () => {
    show({ state: "finished", prompt: null, dirty: true, exit_code: 0 });
    expect(screen.getByText(/uncommitted changes/i)).toBeTruthy();
  });

  it("names the session and its place in the queue", () => {
    show();
    expect(screen.getByText("refactor the parser")).toBeTruthy();
    expect(screen.getByText("1 of 3")).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-b44600c6bdc4-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b44600c6bdc4-expression_statement-3">
it("shares prompt reading and preparation while preserving actual human decision authority", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const receipts: import("../lib/uiActions").UiReceipt[] = [];
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async receipt => { receipts.push(receipt); });
  const answer = vi.fn().mockResolvedValue(false), open = vi.fn(), next = vi.fn(), dismiss = vi.fn(), interrupt = vi.fn().mockResolvedValue(true);
  const panel = (question: string) => <UiActionContext.Provider value={registry}><AttentionCard session={session({ prompt: { question, choices: [], source: "guessed" } })} position={1} total={2}
    onAnswer={answer} onOpen={open} onNext={next} onDismiss={dismiss} onInterrupt={interrupt} /></UiActionContext.Provider>;
  const view = render(panel("Question one?"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  const reading = await run("Read terminal attention card bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes))).toContain("Question one?");
  expect(await run("Prepare terminal attention reply draft", { text: "prepared" })).toMatchObject({ status: "result", outcome: { value: { answer_sent: false, approval: false } } });
  expect((screen.getByLabelText(/type the answer/i) as HTMLInputElement).value).toBe("prepared");
  fireEvent.change(screen.getByLabelText(/type the answer/i), { target: { value: "reviewed" } });
  expect(screen.getByRole("note").textContent).toContain("Prepared by an agent");
  const required = await run("Send attention prompt reply") as { outcome: { request: string } };
  expect(required).toMatchObject({ status: "result", outcome: { status: "requires_input", request: expect.stringContaining("terminal-decision:") } });
  expect(answer).not.toHaveBeenCalled();
  await act(async () => { fireEvent.submit(screen.getByRole("button", { name: "Send" }).closest("form")!); });
  expect(answer).toHaveBeenCalledExactlyOnceWith("reviewed\n");
  expect((screen.getByLabelText(/type the answer/i) as HTMLInputElement).value).toBe("reviewed");
  expect(screen.getByRole("alert").textContent).toContain("refused");
  answer.mockResolvedValueOnce(true);
  await act(async () => { fireEvent.submit(screen.getByRole("button", { name: "Send" }).closest("form")!); });
  expect(receipts.some(receipt => receipt.actor.kind === "human" && receipt.stage === "result" && JSON.stringify(receipt.result).includes(required.outcome.request)
    && JSON.stringify(receipt.result).includes('"host_request_completed":true') && JSON.stringify(receipt.result).includes('"approval_attested":false'))).toBe(true);
  const old = actions().find(action => action.label === "Send attention prompt reply")!.handle;
  view.rerender(panel("Question two?"));
  expect((screen.getByLabelText(/type the answer/i) as HTMLInputElement).value).toBe("");
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "changed-question" })).not.toMatchObject({ status: "result" });
  await run("Open attention terminal"); await run("Show next terminal attention request"); await run("Interrupt attention terminal"); await run("Dismiss terminal attention card");
  expect(open).toHaveBeenCalledTimes(1); expect(next).toHaveBeenCalledTimes(1); expect(interrupt).toHaveBeenCalledTimes(1); expect(dismiss).toHaveBeenCalledTimes(1);
});
</storycode:copy>

### Assemble `apps/web/src/terminal/AttentionCard.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/AttentionCard.test.tsx"><storycode:paste select="#f-b44600c6bdc4-expression_statement"/><storycode:paste select="#f-b44600c6bdc4-session"/><storycode:paste select="#f-b44600c6bdc4-show"/><storycode:paste select="#f-b44600c6bdc4-expression_statement-2"/><storycode:paste select="#f-b44600c6bdc4-expression_statement-3"/></storycode:file>


</storycode:doc>
