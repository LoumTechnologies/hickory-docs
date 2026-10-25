<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/terminal/AttentionCard.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Answer Does Not Cost You Your Place](../../../../../../guarantees/terminal/an-answer-does-not-cost-you-your-place/contract.md)
- [Turbo Never Answers A Prompt It Did Not Parse](../../../../../../guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse/contract.md)

Captured SHA-256: `5f63a3e0789253eef6ae8493ad23ba88f848f4c48eeee675b19edc3f686c7987`.

<book:entry path="apps/web/src/terminal/AttentionCard.tsx">
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
</book:entry>

</book:doc>
