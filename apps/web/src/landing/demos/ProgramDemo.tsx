// Demo 2 — the same machinery pointed at software: a document that explains an
// algorithm and IS the program, woven into a module and its test.
//
// The fragment `#search` is pasted into two files. That is the case worth
// showing: one colour, two ribbons, and no second copy of the code for the
// prose to drift away from.

import { useState } from "react";
import { emit } from "../../analytics/events";
import { DemoSplit } from "./DemoSplit";
import { PROGRAM_DOC_PATH, PROGRAM_SOURCE, PROGRAM_TRANSCRIPT } from "./scripts";

export function ProgramDemo() {
  const [source, setSource] = useState(PROGRAM_SOURCE);
  const [ran, setRan] = useState(false);
  const [rejected, setRejected] = useState<string | null>(null);
  const [engaged, setEngaged] = useState(false);

  const engage = (step: string) => {
    if (engaged) return;
    setEngaged(true);
    emit({ name: "demo_engaged", demo_id: "program", step });
  };

  return (
    <section className="demo" aria-label="A document that is also a program">
      <div className="demo-bar">
        <div className="demo-bar-nav">
          <button
            className="btn btn-primary"
            onClick={() => {
              setRan(true);
              engage("tangle-and-run");
            }}
          >
            {ran ? "Run again" : "Weave, tangle, run"}
          </button>
          <button
            className="btn btn-quiet"
            onClick={() => {
              setSource(PROGRAM_SOURCE);
              setRan(false);
              setRejected(null);
            }}
          >
            Reset
          </button>
        </div>
        <p className="demo-hint">
          Edit the prose, the fragments, or the generated Python — the other side follows. Change
          <span className="mono"> mid = (lo + hi) // 2 </span> on either side and watch both files
          move together.
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
            </div>
          ))}
          <p className="muted demo-transcript-note">
            A recorded transcript, replayed here. In the app this is the real execution, captured
            on every run and compared against what the document claims.
          </p>
        </div>
      )}

      {rejected && (
        <p className="demo-reject" role="status">
          {rejected}
        </p>
      )}

      <DemoSplit
        testId="demo-program"
        docPath={PROGRAM_DOC_PATH}
        source={source}
        onSourceChange={(next) => {
          setSource(next);
          engage("edited-document");
        }}
        sourceCaption={PROGRAM_DOC_PATH}
        nodeHeading="Tangled files"
        editableOutput
        onOutputMappedBack={() => {
          setRejected(null);
          engage("edited-output");
        }}
        onOutputRejected={setRejected}
      />
    </section>
  );
}
