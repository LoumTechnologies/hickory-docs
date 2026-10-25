<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/tour/scenario.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The engineer tour distinguishes replay from execution](../../../../../contract.md)

Captured SHA-256: `3803b2805860db02a57fe12a21480e54baaceccc175d84e95292744af59244a1`.

<book:entry path="apps/web/src/tour/scenario.ts">
export const meeting = "Sam: The total request deadline is 100 milliseconds, including retries.\nAlex: Add a test where two attempts would each take 80 milliseconds.";
export const requirement = "R1: A request must consume no more than its total budget across all attempts.";
export const INVESTIGATION_TASK = "Investigate the isolated retry deadline problem before editing. Read meeting.vtt, AGENTS.md and deadline-spec.md. The customer needs one 100ms total budget including retries; two 80ms attempts must not consume 160ms. Inspect/setup the environment through registered controls, reproduce the failure and use the source-linked debugger to bind a breakpoint and inspect a real paused stop. Use ui_observe(root), then the actual disclosed actions. Report exact source/run/stop receipts and the missing qualification. Do not make corrections yet; distinguish deterministic budget allocation from real network timing.";
export const CORRECTION_TASK = "The human decision is to retain one total deadline including retries. Propose a focused correction subtracting elapsed time from the remaining budget, preserving short-attempt and empty-sequence checks. Use the normal independent edit review. After approval, weave as necessary and invoke Verify saved document through its shared action handle. Inspect the actual current-source run receipt and report failures or unavailable evidence honestly. Do not resolve the human review thread or claim launch readiness.";
export const COACH_STEPS = [
  { title: "Delegate investigation and setup", text: "Ask the agent to investigate the customer problem, establish its environment and reproduce the failure before proposing changes." },
  { title: "Interpret the counterexample and paused source", text: "Inspect the agent's source-bound failure and actual bound/paused debugger evidence. Human time goes to understanding the finding, not translating source coordinates." },
  { title: "Decide what the customer meant", text: "Choose the total deadline including retries and challenge the exact missed qualification in the normal review thread." },
  { title: "Delegate correction and focused verification", text: "Ask for the smallest correction. Review the actual preview, then delegate the saved-source check and inspect its receipt." },
  { title: "Review and resolve", text: "Decide whether the focused diff and evidence meet the criterion; resolve the real thread yourself." },
  { title: "Hand over matching published evidence", text: "Use the existing reviewed publication pipeline and compare the source and tested revision. Local success and a loaded artifact remain separate facts." },
] as const;
export function reconciledCoachStep(saved: number, gates: readonly boolean[]): number {
  if (!Number.isInteger(saved) || saved < 0 || saved >= gates.length) return 0;
  const missing = gates.findIndex(gate => !gate);
  return missing < 0 ? saved : Math.min(saved, missing);
}
export function scenarioSource(corrected: boolean): string {
  return `# A deadline that includes retries\n\n## Meeting evidence\n\n${meeting}\n\n## Requirement\n\n${requirement}\n\n## Recorded example agent proposal\n\nThe deadline bounds the entire operation.\n\n<hick:file id="deadline" path="deadline.js">\nfunction elapsed(attempts, budget) {\n  var total = 0;\n  for (var i = 0; i < attempts.length; i++) {\n    var allowed = ${corrected ? "Math.max(0, budget - total)" : "budget"};\n    total += Math.min(attempts[i], allowed);\n  }\n  return total;\n}\nvar actual = elapsed([80, 80], 100);\nconsole.log("two 80ms attempts, 100ms total budget: " + actual);\nif (actual !== 100) throw new Error("R1 failed: expected 100, got " + actual);\nif (elapsed([20, 30], 100) !== 50) throw new Error("short attempts must remain 50");\nif (elapsed([], 100) !== 0) throw new Error("no attempts must consume zero");\nconsole.log("R1: all three checks passed");\n</hick:file>\n\n## Executable witness\n\nRequires Node.js. The browser tour executes deadline.js with the browser debugger.\nFirst run hick weave deadline-spec.md to materialize deadline.js, then hick test deadline-spec.md. The check runs that file; drift checking keeps it tied to this document.\n\n<hick:container name="shell" image="node:22-alpine" />\n<hick:volume name="src" input="." />\n<hick:exec container="shell" mount="src:project">\nnode project/deadline.js\n<hick:expect match="exact">two 80ms attempts, 100ms total budget: 100\nR1: all three checks passed\n</hick:expect>\n</hick:exec>\n\n## Scope\n\nThis deterministic model checks budget allocation, not OS scheduling or real network timing.\n`;
}
export const TOUR_STEPS = [
  { title: "Start with evidence", text: "Read the meeting excerpt. The deadline includes retries; keep that qualification when turning it into a requirement." },
  { title: "Inspect the agent’s claim", text: "The recorded example proposal says the whole operation is bounded. Its implementation gives every attempt the full budget. Context can be genuine while the conclusion is wrong." },
  { title: "Challenge precisely", text: "Write the objection you would send to the agent. In your workspace, select an actual sentence and choose Comment on selection, or right-click its provenance brace." },
  { title: "Run the counterexample", text: "Run the original document below. Two 80ms attempts exceed the 100ms budget. Continue after the browser reports a failed run." },
  { title: "Inspect and run the correction", text: "The recorded correction subtracts elapsed time from the remaining budget. Run the corrected document. Continue only after the three checks pass." },
  { title: "Review and retain evidence", text: "A passing check supports R1 within the stated model. Review the change and resolve the real thread yourself. Download the executable spec to run locally with hick test." },
] as const;


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
</book:entry>

</book:doc>
