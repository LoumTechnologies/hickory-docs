<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Driving The Home-Page Demo Is Reported As Revealed Interest

For: Maintainers evaluating landing-page behavior and visitor privacy.

Given a visitor on the home page, when they run the executable cells, edit the
document, or edit a generated file, then a `demo_engaged` event is sent
carrying `demo_id` (which demo) and `step` (what they did), alongside the
attribution and declared-segment properties every landing event carries. It is
sent for the **first** interaction only, so the count means "this visitor tried
the product" rather than "this visitor typed n characters".

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/analytics/landing-demo-engagement.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-852e99e545f6"></a>
## `apps/web/src/landing/demos/ProgramDemo.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Driving The Home-Page Demo Is Reported As Revealed Interest](contract.md).

<a id="f-852e99e545f6-ProgramDemo"></a>
### `ProgramDemo`

The home page's only demo: one document, its woven files, and its executable
cells — the whole mechanism in one picture.
The fragment `#search` is pasted into two files. That is the case worth
showing: one colour, two ribbons, and no second copy of the code for the
prose to drift away from. The run is a recording and says so; everything
else — parse, weave, provenance, the reverse edit — is the code the app
runs, in the visitor's browser.

<storycode:copy id="f-852e99e545f6-ProgramDemo">
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
</storycode:copy>

### Assemble `apps/web/src/landing/demos/ProgramDemo.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/landing/demos/ProgramDemo.tsx"><storycode:paste select="#f-852e99e545f6-ProgramDemo"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/analytics/events.ts](../../../mechanisms/apps/web/src/analytics/events.ts.md#output-776c02024e1f)
- [apps/web/src/analytics/sink.ts](../../../mechanisms/apps/web/src/analytics/sink.ts.md#output-d259c2b0da38)
- [apps/web/src/landing/demos/ProgramDemo.tsx](contract.md#output-852e99e545f6)

### Tests and fixtures

- [apps/web/src/landing/demos/demos.test.tsx](../../../mechanisms/apps/web/src/landing/demos/demos.test.tsx.md#output-e2fe96c33971)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
