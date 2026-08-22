// A conversation, rendered: the turns of a session and, inside each, the
// steps the agent took. ONE renderer for two surfaces — the chat dock (live,
// over the turn tree the server holds) and a session file opened as a
// document (after the fact, over the same file the dock wrote). A session
// looks like the chat it was because it is drawn by the same code.
//
// Reasoning folds by default: it is the model thinking, not the answer, and
// a reader who wants it opens it. Tool calls, scripts, and what they returned
// are the work; they fold too, under "show work", so the answer reads first.

import { useEffect, useState, type ReactNode } from "react";
import { api } from "../api/client";
import type { SessionStep, SessionTurn, SessionView } from "../api/types";

/** Reasoning, folded by default. Exported so the dock can show it live. */
export function Reasoning({
  text,
  open = false,
}: {
  text: string;
  open?: boolean;
}) {
  if (!text.trim()) return null;
  return (
    <details className="chat-reasoning" open={open}>
      <summary>reasoning</summary>
      <pre className="chat-reasoning__text">{text}</pre>
    </details>
  );
}

function StepView({ step }: { step: SessionStep }) {
  switch (step.kind) {
    case "reasoning":
      return <Reasoning text={step.text} />;
    case "prose":
      return <p className="chat-step chat-step--prose">{step.text}</p>;
    case "action":
      return (
        <details className="chat-step chat-step--action">
          <summary>
            ran <code>{step.lang}</code>
          </summary>
          <pre>{step.code}</pre>
        </details>
      );
    case "observation":
      return (
        <details
          className={`chat-step chat-step--observation${step.exit && step.exit !== "0" ? " chat-step--failed" : ""}`}
        >
          <summary>
            output{step.exit !== null ? ` · exit ${step.exit}` : ""}
          </summary>
          <pre>{step.text}</pre>
        </details>
      );
    case "tool":
      return (
        <details className="chat-step chat-step--tool">
          <summary>
            <code>{step.name}</code>
            {step.args.map(([k, v]) => (
              <span key={k} className="chat-step__arg">
                {" "}
                {k}=<code>{v.length > 40 ? `${v.slice(0, 40)}…` : v}</code>
              </span>
            ))}
          </summary>
          {step.input !== null && <pre>{step.input}</pre>}
        </details>
      );
    case "tool-result":
      return (
        <details
          className={`chat-step chat-step--result${step.ok ? "" : " chat-step--failed"}`}
        >
          <summary>
            {step.name} — {step.ok ? "ok" : "refused"}
          </summary>
          <pre>{step.text}</pre>
        </details>
      );
    case "read":
      return (
        <p
          className="chat-step chat-step--read muted"
          data-tip={`sha256 ${step.sha256}${step.commit ? ` · commit ${step.commit}` : ""}`}
        >
          read <code>{step.file}</code> lines {step.lines}
        </p>
      );
    case "wrote":
      return (
        <p className="chat-step chat-step--wrote muted">
          wrote <code>{step.file}</code> lines {step.lines}
        </p>
      );
    default:
      return null;
  }
}

/** The body of one turn: reasoning and steps under "show work", then the
 * answer. `extra` (e.g. rewind controls) renders in the user row. */
export function TurnCard({
  turn,
  extra,
  defaultOpen = false,
  live,
  loadWork,
}: {
  turn: SessionTurn;
  extra?: ReactNode;
  defaultOpen?: boolean;
  /** A turn still streaming: its text and reasoning so far. */
  live?: { text: string; reasoning: string } | null;
  /** The dock's turns carry no steps until asked: this fetches them from
   * the session file the turn was recorded in. */
  loadWork?: () => Promise<SessionStep[]>;
}) {
  const [open, setOpen] = useState(defaultOpen);
  const [loaded, setLoaded] = useState<SessionStep[] | null>(null);
  const steps = loaded ?? turn.steps;
  const work = steps.filter(
    (s) => s.kind !== "prose" || s.text !== turn.answer,
  );
  const canLoad = loadWork && loaded === null && turn.steps.length === 0;
  return (
    <article className="chat-turn" data-turn={turn.id}>
      <div className="chat-msg chat-user">
        <span className="chat-role">you</span>
        <p>{turn.prompt}</p>
        {extra}
      </div>
      <div className="chat-msg chat-agent">
        <span className="chat-role">
          agent
          {turn.model ? <span className="muted"> · {turn.model}</span> : null}
        </span>
        {live ? (
          <>
            <Reasoning text={live.reasoning} open />
            <pre className="chat-stream">
              {live.text}
              <span className="t-cursor">▋</span>
            </pre>
          </>
        ) : (
          <>
            {(work.length > 0 || canLoad) && (
              <button
                type="button"
                className="btn-link chat-work-toggle"
                onClick={() => {
                  if (canLoad) {
                    void loadWork().then((s) => {
                      setLoaded(s);
                      setOpen(true);
                    });
                    return;
                  }
                  setOpen((o) => !o);
                }}
              >
                {open
                  ? "hide work"
                  : canLoad
                    ? "show work"
                    : `show work (${work.length} step${work.length === 1 ? "" : "s"})`}
              </button>
            )}
            {open && (
              <div className="chat-work">
                {work.map((s, i) => (
                  <StepView key={i} step={s} />
                ))}
              </div>
            )}
            {turn.answer ? (
              <p className="chat-answer">{turn.answer}</p>
            ) : (
              <p className="muted">no answer recorded</p>
            )}
          </>
        )}
      </div>
    </article>
  );
}

/** A session FILE opened in the app, drawn as the conversation it records —
 * the same cards the dock draws, because it is the same conversation. */
export function SessionDocView({ path }: { path: string }) {
  const [view, setView] = useState<SessionView | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    api.sessionView(path).then(
      (r) => live && setView(r.view),
      (e: unknown) =>
        live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [path]);
  if (error) return <p className="error chat-empty">{error}</p>;
  if (!view) return <p className="muted chat-empty">Reading the session…</p>;
  return (
    <div className="session-doc">
      <p className="muted session-doc__head">
        {view.doc ? (
          <>
            A conversation about <code>{view.doc}</code>
          </>
        ) : (
          "A conversation"
        )}
        {view.start ? ` · started ${view.start}` : ""} · {view.turns.length}{" "}
        turn{view.turns.length === 1 ? "" : "s"}
      </p>
      <SessionTurns turns={view.turns} />
    </div>
  );
}

/** A whole session file as a conversation. */
export function SessionTurns({ turns }: { turns: SessionTurn[] }) {
  if (turns.length === 0)
    return <p className="muted chat-empty">This session holds no turns yet.</p>;
  return (
    <div className="chat-log chat-log--session">
      {turns.map((t) => (
        <TurnCard key={`${t.id}:${t.session_line}`} turn={t} />
      ))}
    </div>
  );
}
