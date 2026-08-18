// The card that answers a prompt without taking you to it.
//
// The point of the queue is that you keep working; a card that made you go to
// a terminal, read it, answer, and come back would just be a fancier way of
// checking on things. So the top of the queue comes to you, with the agent's
// own choices on it where the program declared them.
//
// Protects docs/guarantees/terminal/an-answer-does-not-cost-you-your-place.md

import { useEffect, useState } from "react";

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
  onAnswer: (send: string) => void;
  onOpen: () => void;
  onInterrupt: () => void;
  onNext: () => void;
  onDismiss: () => void;
}) {
  const [typed, setTyped] = useState("");
  // A new session under the card is a new question; never carry a half-typed
  // answer from one prompt to another.
  useEffect(() => setTyped(""), [session.id]);

  const prompt = session.prompt;
  const guessed = prompt?.source === "guessed";

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
              onClick={() => onAnswer(choice.send)}
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
            onAnswer(`${typed}\n`);
            setTyped("");
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
            autoFocus
            onChange={(e) => setTyped(e.target.value)}
          />
          <button className="btn btn-primary" type="submit">
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

      <footer className="attention-card-foot">
        <button className="btn" onClick={onOpen}>
          Open terminal
        </button>
        {session.state === "needs-you" && (
          <button className="btn" onClick={onInterrupt}>
            Interrupt
          </button>
        )}
        <button className="btn" onClick={onNext}>
          Next
        </button>
        <button className="btn" onClick={onDismiss}>
          Later
        </button>
      </footer>
    </div>
  );
}
