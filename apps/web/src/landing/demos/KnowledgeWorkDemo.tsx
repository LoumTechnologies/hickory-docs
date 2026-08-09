// Demo 1 — knowledge work: meeting notes become Jira issues, and the issues
// stay attached to the sentence that produced them.
//
// The Jira side is FAKE and says so on screen: nothing here reaches Atlassian,
// and the "issues" are files this browser wove a moment ago. What is real is
// the part the product is actually about — the lineage between a note and a
// ticket description, and the round trip that carries an edit made downstream
// back to the note it came from.

import { useCallback, useEffect, useRef, useState } from "react";
import { emit } from "../../analytics/events";
import { DemoSplit } from "./DemoSplit";
import {
  KNOWLEDGE_DOC_PATH,
  KNOWLEDGE_STEPS,
  advance,
  changedRange,
  findRange,
} from "./scripts";

/** "jira/HD-412.md" → "HD-412". */
function issueKey(path: string): string {
  return path.replace(/^.*\//, "").replace(/\.md$/, "");
}

/**
 * How long autoplay lingers on a step. Long enough to read the hint and watch
 * the document move; a demo that outruns its reader teaches nothing. The last
 * two steps are hands-on, so autoplay stops before them rather than clicking
 * past the part it was trying to get you to try.
 */
const AUTOPLAY_MS = 4200;
const AUTOPLAY_LAST_STEP = 4;

export function KnowledgeWorkDemo() {
  const [state, setState] = useState({
    step: 0,
    source: KNOWLEDGE_STEPS[0].source,
    edited: false,
  });
  const [reveal, setReveal] = useState<[number, number] | null>(null);
  const [playing, setPlaying] = useState(false);
  const [reconciled, setReconciled] = useState<string | null>(null);
  const [rejected, setRejected] = useState<string | null>(null);

  const step = KNOWLEDGE_STEPS[state.step];

  // The live state, readable from callbacks that must not be rebuilt on every
  // render (the autoplay timer holds one of them).
  const stateRef = useRef(state);
  stateRef.current = state;

  const goTo = useCallback((index: number, viaAutoplay = false) => {
    const current = stateRef.current;
    if (index === current.step) return;
    const direction = index > current.step ? 1 : -1;
    let next = current;
    for (let i = current.step; i !== index; i += direction) next = advance(next, direction);
    setState(next);
    // A step's whole payload can land below the fold — the tool calls, the
    // woven ticket files. Point the editor at what actually changed, or the
    // click looks like it did nothing. A step that changes nothing says where
    // to look instead.
    const target = KNOWLEDGE_STEPS[index];
    setReveal(
      changedRange(current.source, next.source) ??
        (target.focus ? findRange(next.source, target.focus) : null),
    );
    setReconciled(null);
    setRejected(null);
    emit({
      name: "demo_engaged",
      demo_id: "knowledge-work",
      step: KNOWLEDGE_STEPS[index].id,
      autoplay: viaAutoplay,
    });
  }, []);

  // Autoplay. It stops itself at the hands-on steps, and any manual click
  // stops it too: fighting a visitor for control of the thing they are trying
  // to drive is worse than never having offered to drive it for them.
  useEffect(() => {
    if (!playing) return;
    const timer = window.setInterval(() => {
      const at = stateRef.current.step;
      if (at >= AUTOPLAY_LAST_STEP) {
        setPlaying(false);
        return;
      }
      goTo(at + 1, true);
    }, AUTOPLAY_MS);
    return () => clearInterval(timer);
  }, [playing, goTo]);

  const manual = (index: number) => () => {
    setPlaying(false);
    goTo(index);
  };

  const play = () => {
    if (playing) {
      setPlaying(false);
      return;
    }
    // Replaying from the end should start over rather than do nothing.
    if (state.step >= AUTOPLAY_LAST_STEP) goTo(0);
    setPlaying(true);
    emit({ name: "demo_engaged", demo_id: "knowledge-work", step: "autoplay" });
  };

  return (
    <section className="demo" aria-label="Meeting notes to Jira issues">
      <ol className="demo-steps">
        {KNOWLEDGE_STEPS.map((s, i) => (
          <li key={s.id}>
            <button
              type="button"
              className={`demo-step${i === state.step ? " on" : ""}${
                i < state.step ? " done" : ""
              }`}
              aria-current={i === state.step}
              onClick={manual(i)}
            >
              <span className="demo-step-n">{i + 1}</span>
              {s.label}
            </button>
          </li>
        ))}
      </ol>

      <div className="demo-bar">
        <div className="demo-bar-nav">
          <button className="btn btn-primary" onClick={play} aria-pressed={playing}>
            {playing
              ? "Pause"
              : state.step === 0
                ? "Play it for me"
                : // Past the point autoplay stops, pressing play starts over —
                  // so the label has to say so rather than promise to carry on.
                  state.step >= AUTOPLAY_LAST_STEP
                  ? "Play it again"
                  : "Play from here"}
          </button>
          <button
            className="btn btn-quiet"
            onClick={manual(state.step - 1)}
            disabled={state.step === 0}
          >
            Back
          </button>
          <button
            className="btn"
            onClick={manual(state.step + 1)}
            disabled={state.step === KNOWLEDGE_STEPS.length - 1}
          >
            Next
          </button>
        </div>
        <p className="demo-hint">{step.hint}</p>
      </div>

      {step.activity && (
        <ul className="demo-activity" aria-label="Agent activity">
          {step.activity.map((line) => (
            <li key={line} className="mono">
              {line}
            </li>
          ))}
        </ul>
      )}

      {reconciled && (
        <p className="demo-reconcile" role="status">
          <strong>Jira changed.</strong> The agent was prompted by the change — not by you — and
          wrote it back into <span className="mono">{reconciled}</span>. Look at the note on the
          left: it now reads the way the ticket does.
        </p>
      )}
      {rejected && (
        <p className="demo-reject" role="status">
          {rejected}
        </p>
      )}

      <DemoSplit
        testId="demo-knowledge"
        docPath={KNOWLEDGE_DOC_PATH}
        source={state.source}
        onSourceChange={(source) => setState((s) => ({ ...s, source, edited: true }))}
        sourceCaption={KNOWLEDGE_DOC_PATH}
        nodeHeading="Jira issues"
        nodeLabel={issueKey}
        outputCaption={(path) => `Jira · ${issueKey(path)} (simulated)`}
        editableOutput={step.invites === "edit"}
        revealSpan={reveal}
        onOutputMappedBack={({ path }) => {
          setRejected(null);
          setReconciled(issueKey(path));
          emit({ name: "demo_engaged", demo_id: "knowledge-work", step: "round-trip" });
        }}
        onOutputRejected={(message) => {
          setReconciled(null);
          setRejected(message);
        }}
      />

      <p className="demo-foot muted">
        The Jira half is simulated in this page — no issue tracker is contacted. Everything else
        (the weave, the provenance, the round trip) is the same code the app runs.
      </p>
    </section>
  );
}
