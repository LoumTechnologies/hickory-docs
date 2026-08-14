// The home page's only demo: one document, its woven files, and its executable
// cells — the whole mechanism in one picture.
//
// The fragment `#search` is pasted into two files. That is the case worth
// showing: one colour, two ribbons, and no second copy of the code for the
// prose to drift away from. The run is a recording and says so; everything
// else — parse, weave, provenance, the reverse edit — is the code the app
// runs, in the visitor's browser.

import { useMemo, useState } from "react";
import { emit } from "../../analytics/events";
import { LineageColumns } from "../../lineage/LineageColumns";
import { buildModel } from "../../lineage/build";
import { weaveOutputs } from "../../lib/weave";
import { DemoSplit } from "./DemoSplit";
import { PROGRAM_DOC_PATH, PROGRAM_SOURCE, PROGRAM_TRANSCRIPT } from "./scripts";

export function ProgramDemo() {
  const [source, setSource] = useState(PROGRAM_SOURCE);
  // Two ways to look at one document, both driven from the same source text
  // in this tab. "Lineage" is the browser the desktop app ships; "Edit" is
  // the round trip — change a generated file, watch the fragment move — and
  // neither is a mockup of the other.
  const [mode, setMode] = useState<"lineage" | "edit">("lineage");
  const model = useMemo(
    () =>
      buildModel([
        { path: PROGRAM_DOC_PATH, source, outputs: weaveOutputs(source, PROGRAM_DOC_PATH) },
      ]),
    [source],
  );
  const [ran, setRan] = useState(false);
  const [rejected, setRejected] = useState<string | null>(null);
  const [mappedBack, setMappedBack] = useState(false);
  const [engaged, setEngaged] = useState(false);

  const engage = (step: string) => {
    if (engaged) return;
    setEngaged(true);
    emit({ name: "demo_engaged", demo_id: "program", step });
  };

  return (
    <section className="demo" aria-label="One document, its files, and its runs">
      <div className="demo-bar">
        <div className="demo-bar-nav">
          <div className="demo-modes" role="group" aria-label="How to look at this document">
            <button
              className="btn btn-quiet"
              aria-pressed={mode === "lineage"}
              onClick={() => {
                setMode("lineage");
                engage("mode-lineage");
              }}
            >
              Lineage
            </button>
            <button
              className="btn btn-quiet"
              aria-pressed={mode === "edit"}
              onClick={() => {
                setMode("edit");
                engage("mode-edit");
              }}
            >
              Edit both ends
            </button>
          </div>
          <button
            className="btn btn-primary"
            onClick={() => {
              setRan(true);
              engage("run-cells");
            }}
          >
            {ran ? "Run the cells again" : "Run the cells"}
          </button>
          <button
            className="btn btn-quiet"
            onClick={() => {
              setSource(PROGRAM_SOURCE);
              setRan(false);
              setRejected(null);
              setMappedBack(false);
            }}
          >
            Reset
          </button>
        </div>
        <p className="demo-hint">
          Type on either side. Change
          <span className="mono"> mid = (lo + hi) // 2 </span>
          in the fragment on the left and both generated files move; change it in the generated
          Python on the right and the fragment moves. There is one copy of the code, and you are
          always editing it.
        </p>
      </div>

      {ran && (
        <div className="demo-transcript" aria-label="Captured run">
          {PROGRAM_TRANSCRIPT.map((entry) => (
            <div key={entry.cmd}>
              <p className="mono demo-transcript-cmd">$ {entry.cmd}</p>
              {entry.out.map((line, i) => (
                <p key={i} className="mono demo-transcript-out">
                  {line}
                </p>
              ))}
              <p className="demo-transcript-verdict">✓ {entry.verdict}</p>
            </div>
          ))}
          <p className="muted demo-transcript-note">
            A recorded transcript, replayed here — a browser tab cannot start a container. On your
            machine <span className="mono">hick run</span> executes these cells for real and writes
            the output back into the document, and{" "}
            <span className="mono">hick test</span> re-runs them and exits non-zero when the
            document and reality disagree. That is what a pre-commit hook and CI run.
          </p>
        </div>
      )}

      {mappedBack && (
        <p className="demo-reconcile" role="status">
          <strong>That edit went into the document.</strong> You changed a generated file, and the
          change was resolved backwards through provenance into the fragment it came from — so the
          other file that pastes the same fragment moved with it.
        </p>
      )}
      {rejected && (
        <p className="demo-reject" role="status">
          {rejected}
        </p>
      )}

      {mode === "lineage" ? (
        <div className="demo-lineage">
          <p className="demo-hint">
            One column per stage: the document and the files it weaves. Fold from the line numbers,
            move either edge of a hole, and click a fragment to see what it feeds — the brackets are
            real provenance, computed in this tab.
          </p>
          <LineageColumns model={model} />
        </div>
      ) : (
      <DemoSplit
        testId="demo-program"
        docPath={PROGRAM_DOC_PATH}
        source={source}
        onSourceChange={(next) => {
          setSource(next);
          engage("edited-document");
        }}
        sourceCaption={PROGRAM_DOC_PATH}
        nodeHeading="Woven files"
        editableOutput
        onOutputMappedBack={() => {
          setRejected(null);
          setMappedBack(true);
          engage("edited-output");
        }}
        onOutputRejected={(message) => {
          setMappedBack(false);
          setRejected(message);
        }}
      />
      )}
    </section>
  );
}
