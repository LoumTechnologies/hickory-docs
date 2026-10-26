<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The engineer tour distinguishes replay from execution

For: Authors editing executable documents and the files those documents produce.

Given the AI engineer tour on the homepage or in the application, the same
scenario identifies meeting evidence, a requirement, a recorded agent mistake,
a precise objection, a failing counterexample and a corrected implementation.
Agent proposals are labeled recorded examples. Browser execution uses the product
browser debugger. The counterexample step requires observed failure and the
correction step requires observed success before Continue is enabled.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-86702cf814fb"></a>
## `apps/web/src/components/WorkflowTour.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The engineer tour distinguishes replay from execution](contract.md).

<a id="f-86702cf814fb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-86702cf814fb-expression_statement">
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
vi.mock("../embed/DebuggableDocument", () => ({ DebuggableDocument: ({ revision, onExecutionResult }: {revision:string;onExecutionResult:(r:{revision:string;exitCode:number})=>void}) => <button onClick={() => onExecutionResult({ revision, exitCode: revision === "original-example" ? 1 : 0 })}>Run fixture</button> }));
</storycode:copy>
<a id="f-86702cf814fb-import_statement"></a>
### `import_statement`

<storycode:copy id="f-86702cf814fb-import_statement">
import { WorkflowTour } from "./WorkflowTour";
</storycode:copy>
<a id="f-86702cf814fb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-86702cf814fb-expression_statement-2">
afterEach(() => { cleanup(); localStorage.clear(); });
</storycode:copy>
<a id="f-86702cf814fb-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-86702cf814fb-expression_statement-3">
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
it("requires observed failure and success rather than only clicking Next", () => {
  render(<WorkflowTour surface="site" />);
  fireEvent.click(screen.getByRole("button", { name: "AI engineer tour" }));
  fireEvent.click(screen.getByText("Continue")); fireEvent.click(screen.getByText("Continue"));
  expect((screen.getByText("Continue") as HTMLButtonElement).disabled).toBe(true);
  fireEvent.change(screen.getByLabelText("Your objection"), { target: { value: "Retries exceed the budget" } });
  fireEvent.click(screen.getByText("Continue"));
  expect((screen.getByText("Continue") as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByText("Run fixture")); fireEvent.click(screen.getByText("Continue"));
  expect((screen.getByText("Continue") as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByText("Run fixture")); fireEvent.click(screen.getByText("Continue"));
  expect(screen.getByText("Review and retain evidence")).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/components/WorkflowTour.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/WorkflowTour.test.tsx"><storycode:paste select="#f-86702cf814fb-expression_statement"/><storycode:paste select="#f-86702cf814fb-import_statement"/><storycode:paste select="#f-86702cf814fb-expression_statement-2"/><storycode:paste select="#f-86702cf814fb-expression_statement-3"/></storycode:file>

<a id="output-512ddbf29209"></a>
## `apps/web/src/tour/scenario.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The engineer tour distinguishes replay from execution](contract.md).

<a id="f-512ddbf29209-ambient_declaration"></a>
### `ambient_declaration`

<storycode:copy id="f-512ddbf29209-ambient_declaration">
import { expect, it } from "vitest";
import { scenarioSource, reconciledCoachStep } from "./scenario";
declare const require: (id: string) => { readFileSync(p: string, encoding: string): string };
</storycode:copy>
<a id="f-512ddbf29209-ambient_declaration-2"></a>
### `ambient_declaration`

<storycode:copy id="f-512ddbf29209-ambient_declaration-2">
declare const process: { cwd(): string };
</storycode:copy>
<a id="f-512ddbf29209-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-512ddbf29209-expression_statement">
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
it("downloads the specification exercised by the native engine", () => {
  const fixture = require("node:fs").readFileSync(`${process.cwd()}/../../examples/ai-engineer-deadline.md`, "utf8");
  expect(scenarioSource(true)).toBe(fixture);
});
</storycode:copy>
<a id="f-512ddbf29209-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-512ddbf29209-expression_statement-2">
it("reconciles saved guidance to the first missing actual evidence", () => {
  expect(reconciledCoachStep(4, [true, false, true, true, false, false])).toBe(1);
  expect(reconciledCoachStep(1, [true, true, true, true, true, true])).toBe(1);
  expect(reconciledCoachStep(99, [true, true])).toBe(0);
});
</storycode:copy>

### Assemble `apps/web/src/tour/scenario.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/tour/scenario.test.ts"><storycode:paste select="#f-512ddbf29209-ambient_declaration"/><storycode:paste select="#f-512ddbf29209-ambient_declaration-2"/><storycode:paste select="#f-512ddbf29209-expression_statement"/><storycode:paste select="#f-512ddbf29209-expression_statement-2"/></storycode:file>

<a id="output-599b7f973135"></a>
## `apps/web/src/tour/scenario.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The engineer tour distinguishes replay from execution](contract.md).

<a id="f-599b7f973135-meeting"></a>
### `meeting`

<storycode:copy id="f-599b7f973135-meeting">
export const meeting = "Sam: The total request deadline is 100 milliseconds, including retries.\nAlex: Add a test where two attempts would each take 80 milliseconds.";
</storycode:copy>
<a id="f-599b7f973135-requirement"></a>
### `requirement`

<storycode:copy id="f-599b7f973135-requirement">
export const requirement = "R1: A request must consume no more than its total budget across all attempts.";
</storycode:copy>
<a id="f-599b7f973135-INVESTIGATION_TASK"></a>
### `INVESTIGATION_TASK`

<storycode:copy id="f-599b7f973135-INVESTIGATION_TASK">
export const INVESTIGATION_TASK = "Investigate the isolated retry deadline problem before editing. Read meeting.vtt, AGENTS.md and deadline-spec.md. The customer needs one 100ms total budget including retries; two 80ms attempts must not consume 160ms. Inspect/setup the environment through registered controls, reproduce the failure and use the source-linked debugger to bind a breakpoint and inspect a real paused stop. Use ui_observe(root), then the actual disclosed actions. Report exact source/run/stop receipts and the missing qualification. Do not make corrections yet; distinguish deterministic budget allocation from real network timing.";
</storycode:copy>
<a id="f-599b7f973135-CORRECTION_TASK"></a>
### `CORRECTION_TASK`

<storycode:copy id="f-599b7f973135-CORRECTION_TASK">
export const CORRECTION_TASK = "The human decision is to retain one total deadline including retries. Propose a focused correction subtracting elapsed time from the remaining budget, preserving short-attempt and empty-sequence checks. Use the normal independent edit review. After approval, weave as necessary and invoke Verify saved document through its shared action handle. Inspect the actual current-source run receipt and report failures or unavailable evidence honestly. Do not resolve the human review thread or claim launch readiness.";
</storycode:copy>
<a id="f-599b7f973135-COACH_STEPS"></a>
### `COACH_STEPS`

<storycode:copy id="f-599b7f973135-COACH_STEPS">
export const COACH_STEPS = [
  { title: "Delegate investigation and setup", text: "Ask the agent to investigate the customer problem, establish its environment and reproduce the failure before proposing changes." },
  { title: "Interpret the counterexample and paused source", text: "Inspect the agent's source-bound failure and actual bound/paused debugger evidence. Human time goes to understanding the finding, not translating source coordinates." },
  { title: "Decide what the customer meant", text: "Choose the total deadline including retries and challenge the exact missed qualification in the normal review thread." },
  { title: "Delegate correction and focused verification", text: "Ask for the smallest correction. Review the actual preview, then delegate the saved-source check and inspect its receipt." },
  { title: "Review and resolve", text: "Decide whether the focused diff and evidence meet the criterion; resolve the real thread yourself." },
  { title: "Hand over matching published evidence", text: "Use the existing reviewed publication pipeline and compare the source and tested revision. Local success and a loaded artifact remain separate facts." },
] as const;
</storycode:copy>
<a id="f-599b7f973135-reconciledCoachStep"></a>
### `reconciledCoachStep`

<storycode:copy id="f-599b7f973135-reconciledCoachStep">
export function reconciledCoachStep(saved: number, gates: readonly boolean[]): number {
  if (!Number.isInteger(saved) || saved < 0 || saved >= gates.length) return 0;
  const missing = gates.findIndex(gate => !gate);
  return missing < 0 ? saved : Math.min(saved, missing);
}
</storycode:copy>
<a id="f-599b7f973135-scenarioSource"></a>
### `scenarioSource`

<storycode:copy id="f-599b7f973135-scenarioSource">
export function scenarioSource(corrected: boolean): string {
  return `# A deadline that includes retries\n\n## Meeting evidence\n\n${meeting}\n\n## Requirement\n\n${requirement}\n\n## Recorded example agent proposal\n\nThe deadline bounds the entire operation.\n\n<hick:file id="deadline" path="deadline.js">\nfunction elapsed(attempts, budget) {\n  var total = 0;\n  for (var i = 0; i < attempts.length; i++) {\n    var allowed = ${corrected ? "Math.max(0, budget - total)" : "budget"};\n    total += Math.min(attempts[i], allowed);\n  }\n  return total;\n}\nvar actual = elapsed([80, 80], 100);\nconsole.log("two 80ms attempts, 100ms total budget: " + actual);\nif (actual !== 100) throw new Error("R1 failed: expected 100, got " + actual);\nif (elapsed([20, 30], 100) !== 50) throw new Error("short attempts must remain 50");\nif (elapsed([], 100) !== 0) throw new Error("no attempts must consume zero");\nconsole.log("R1: all three checks passed");\n</hick:file>\n\n## Executable witness\n\nRequires Node.js. The browser tour executes deadline.js with the browser debugger.\nFirst run hick weave deadline-spec.md to materialize deadline.js, then hick test deadline-spec.md. The check runs that file; drift checking keeps it tied to this document.\n\n<hick:container name="shell" image="node:22-alpine" />\n<hick:volume name="src" input="." />\n<hick:exec container="shell" mount="src:project">\nnode project/deadline.js\n<hick:expect match="exact">two 80ms attempts, 100ms total budget: 100\nR1: all three checks passed\n</hick:expect>\n</hick:exec>\n\n## Scope\n\nThis deterministic model checks budget allocation, not OS scheduling or real network timing.\n`;
}
</storycode:copy>
<a id="f-599b7f973135-TOUR_STEPS"></a>
### `TOUR_STEPS`

<storycode:copy id="f-599b7f973135-TOUR_STEPS">
export const TOUR_STEPS = [
  { title: "Start with evidence", text: "Read the meeting excerpt. The deadline includes retries; keep that qualification when turning it into a requirement." },
  { title: "Inspect the agent’s claim", text: "The recorded example proposal says the whole operation is bounded. Its implementation gives every attempt the full budget. Context can be genuine while the conclusion is wrong." },
  { title: "Challenge precisely", text: "Write the objection you would send to the agent. In your workspace, select an actual sentence and choose Comment on selection, or right-click its provenance brace." },
  { title: "Run the counterexample", text: "Run the original document below. Two 80ms attempts exceed the 100ms budget. Continue after the browser reports a failed run." },
  { title: "Inspect and run the correction", text: "The recorded correction subtracts elapsed time from the remaining budget. Run the corrected document. Continue only after the three checks pass." },
  { title: "Review and retain evidence", text: "A passing check supports R1 within the stated model. Review the change and resolve the real thread yourself. Download the executable spec to run locally with hick test." },
] as const;


</storycode:copy>
<a id="f-599b7f973135-APP_TOUR_PHASES"></a>
### `APP_TOUR_PHASES`

<storycode:copy id="f-599b7f973135-APP_TOUR_PHASES">
export const APP_TOUR_PHASES = [
  { title: "Import meeting evidence", text: "Import the transparent VTT fixture through the existing deterministic importer and inspect retained utterance citations." },
  { title: "Challenge the exact provenance claim", text: "Open the actual source and create a persistent review thread on the missed qualification; inspect Git credit and recorded-origin boundaries." },
  { title: "Observe the real counterexample", text: "Execute the current saved example with the shared browser debugger. A specific 160ms-versus-100ms failure is required. Inspect its actual source-linked output and paused breakpoint marker; expand shared observation handles rather than translating coordinates." },
  { title: "Update actual agent context", text: "Observe a real source read, edit the requirement, inspect its inline patch and deliver an explicitly selected update at the next safe boundary." },
  { title: "Investigate through the actual review thread", text: "Send the saved objection to a selected real conversation, then inspect the returned response and proposed correction. A model declaration is not executed verification." },
  { title: "Review and verify the correction", text: "Review a focused diff, approve the change and run the saved native executable witnesses. Total budget allocation remains separate from real network timing." },
  { title: "Measure and remeasure a real Python hotspot", text: "Profile saved hotspot.md with cProfile, select source-linked function evidence for agent context, make the optimization, and rerun correctness and profiling on a distinct source revision." },
  { title: "Resolve through human judgment", text: "Inspect actual receipts and comparisons, then explicitly resolve the original thread using the normal review controls." },
  { title: "Compare revision-bound publication", text: "Load an actual published verification artifact and compare its source hash with the saved corrected document. A historical pass for different bytes stays stale." },
] as const;
</storycode:copy>

### Assemble `apps/web/src/tour/scenario.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/tour/scenario.ts"><storycode:paste select="#f-599b7f973135-meeting"/><storycode:paste select="#f-599b7f973135-requirement"/><storycode:paste select="#f-599b7f973135-INVESTIGATION_TASK"/><storycode:paste select="#f-599b7f973135-CORRECTION_TASK"/><storycode:paste select="#f-599b7f973135-COACH_STEPS"/><storycode:paste select="#f-599b7f973135-reconciledCoachStep"/><storycode:paste select="#f-599b7f973135-scenarioSource"/><storycode:paste select="#f-599b7f973135-TOUR_STEPS"/><storycode:paste select="#f-599b7f973135-APP_TOUR_PHASES"/></storycode:file>

## Test the behavior: `deadline_witness_passes_and_exposes_the_original_bug`

This is the maintained test fragment from `crates/hickory-cli/tests/engineer_tour.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-ae34e687590d-deadline_witness_passes_and_exposes_the_original_bug"></a>
### `deadline_witness_passes_and_exposes_the_original_bug`

<storycode:copy id="f-ae34e687590d-deadline_witness_passes_and_exposes_the_original_bug">
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
#[tokio::test]
async fn deadline_witness_passes_and_exposes_the_original_bug() {
    let source = include_str!("../../../examples/ai-engineer-deadline.md");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("deadline.md");
    std::fs::write(&path, source).unwrap();
    let woven = run_doc(&path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    assert_eq!(
        woven
            .result
            .files
            .get("deadline.js")
            .and_then(|content| content.as_text()),
        Some(include_str!("../../../examples/deadline.js"))
    );
    hickory_cli::write_outputs(&woven, None).unwrap();
    let corrected = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        run_doc(&path, &[], RunMode::Verify, ExecutorChoice::Local),
    )
    .await
    .expect("tour check must finish")
    .unwrap();
    assert_eq!(corrected.result.expectations.len(), 1);
    assert!(
        corrected.result.expectations[0].passed,
        "{:?}",
        corrected.result.expectations
    );
    let broken = source.replace(
        "var allowed = Math.max(0, budget - total);",
        "var allowed = budget;",
    );
    assert_ne!(broken, source);
    std::fs::write(&path, broken).unwrap();
    let woven = run_doc(&path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    hickory_cli::write_outputs(&woven, None).unwrap();
    let broken = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        run_doc(&path, &[], RunMode::Verify, ExecutorChoice::Local),
    )
    .await
    .expect("counterexample must finish");
    match broken {
        Err(error) => assert!(
            error
                .to_string()
                .contains("R1 failed: expected 100, got 160"),
            "{error:#}"
        ),
        Ok(run) => assert!(run.result.expectations.iter().any(|check| !check.passed)),
    }
}

</storycode:copy>

## Test the behavior: `native_verification_receipt_names_the_exact_source_that_ran`

This is the maintained test fragment from `crates/hickory-cli/tests/engineer_tour.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-ae34e687590d-native_verification_receipt_names_the_exact_source_that_ran"></a>
### `native_verification_receipt_names_the_exact_source_that_ran`

<storycode:copy id="f-ae34e687590d-native_verification_receipt_names_the_exact_source_that_ran">
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
#[tokio::test]
async fn native_verification_receipt_names_the_exact_source_that_ran() {
    use hickory_cli::serve::{ServeOptions, prepare};
    let temporary = tempfile::tempdir().unwrap();
    let source = include_str!("../../../examples/ai-engineer-deadline.md");
    let path = temporary.path().join("deadline-spec.md");
    std::fs::write(&path, source).unwrap();
    let woven = run_doc(&path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .unwrap();
    hickory_cli::write_outputs(&woven, None).unwrap();
    let prepared = prepare(ServeOptions {
        target: temporary.path().to_path_buf(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: Some(temporary.path().join("keys.json")),
        ui_settings_path: Some(temporary.path().join("ui.json")),
    })
    .await
    .unwrap();
    let id = prepared.state.index.add("deadline-spec.md");
    let run = prepared.state.start_run(&id, true).await.unwrap();
    let receipt = tokio::time::timeout(std::time::Duration::from_secs(20), async {
        loop {
            let receipt = hickory_cli::serve::api::get_run(
                axum::extract::State(prepared.state.clone()),
                axum::extract::Path(run.clone()),
            )
            .await
            .unwrap()
            .0;
            if receipt["status"] != "running" {
                break receipt;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(receipt["status"], "ok");
    assert_eq!(
        receipt["source_hash"],
        hick_context_protocol::digest(source.as_bytes())
    );
    assert!(
        receipt["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["kind"] == "exec" && b["status"] == "ok")
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/WorkflowTour.tsx](../../verification/browser-predictions-use-the-shared-gate/contract.md#output-39bbc44cf019)
- [apps/web/src/embed/DebuggableDocument.tsx](../../../mechanisms/apps/web/src/embed/DebuggableDocument.tsx.md#output-b948f7ea7230)
- [apps/web/src/tour/scenario.ts](contract.md#output-599b7f973135)
- [apps/web/src/views/LandingView.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-e86ad48a0990)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hickory-cli/src/serve/engineer_tour.rs](../../../mechanisms/crates/hickory-cli/src/serve/engineer_tour.rs.md#output-f4de6cb618b8)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/e2e/privateCoachLayout.spec.ts](../../../mechanisms/apps/web/e2e/privateCoachLayout.spec.ts.md#output-3f8363e8ca8f)
- [apps/web/src/components/NativeWorkflowTour.test.tsx](../../../mechanisms/apps/web/src/components/NativeWorkflowTour.test.tsx.md#output-8a1a5703c15d)
- [apps/web/src/components/WorkflowTour.test.tsx](contract.md#output-86702cf814fb)
- [apps/web/src/tour/scenario.test.ts](contract.md#output-512ddbf29209)
- [crates/hickory-cli/tests/engineer_tour.rs](../../../mechanisms/crates/hickory-cli/tests/engineer_tour.rs.md#output-ae34e687590d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
