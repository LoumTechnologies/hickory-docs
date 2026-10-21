// Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import recorded from "../../../../examples/security-authorization/predictions.json";
import { PredictionEvidence, validatePredictionHistory } from "./PredictionHistory";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
it("accepts the recorded host pair and refuses raw answers, stale certification and missing seals", () => {
  validatePredictionHistory(recorded);
  expect(recorded.history.attempts.map(attempt => attempt.classification)).toEqual(["execution_rejected", "verified_after_correction"]);
  for (const corrupt of [
    (value: typeof recorded) => { value.history.ordering.find(entry => entry.event.kind === "finished")!.event.actual = "PRIVATE ANSWER"; },
    (value: typeof recorded) => { value.history.attempts[1].current_certification = true; },
    (value: typeof recorded) => { value.history.attempts[1].seal = "missing"; },
    (value: typeof recorded) => { value.history.ordering[1].sequence = value.history.ordering[0].sequence; },
    (value: typeof recorded) => { value.history.ordering = value.history.ordering.filter(entry => entry.event.kind !== "finished"); },
  ]) {
    const copy = structuredClone(recorded); corrupt(copy);
    expect(() => validatePredictionHistory(copy)).toThrow("Invalid selected prediction history");
  }
  expect(() => validatePredictionHistory({ ...recorded, actual: "PRIVATE ANSWER" })).toThrow();
});
it("discloses both revisions and reports a different source without certifying a current pass", async () => {
  validatePredictionHistory(recorded);
  const registry = new UiActions({ workspace: "published", window: "viewer" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PredictionEvidence value={recorded} sourceHash={"f".repeat(64)} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  expect(screen.getByText("Inspect sealed predictions and retained attempts").closest("details")!.open).toBe(false);
  expect(actions().some(action => action.label === "Read disclosed selected prediction chain bytes")).toBe(false);
  const disclosure = registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Set selected prediction history visibility")!;
  await act(async () => { await registry.act(registry.scope, { kind: "agent", id: "viewer-test" }, { handle: disclosure.handle, arguments: { open: true }, request_id: "inspect" }); });
  expect(screen.getByText(/execution_rejected · attempt/)).toBeTruthy();
  expect(screen.getByText(/verified_after_correction · attempt/)).toBeTruthy();
  expect(screen.getAllByText(/different revision; source bytes not included/)).toHaveLength(2);
  expect(screen.getAllByText(/Current certification: unavailable/)).toHaveLength(2);
  const details = screen.getByText("Inspect sealed predictions and retained attempts").closest("details")!;
  details.open = false; fireEvent(details, new Event("toggle"));
  expect(details.open).toBe(false);
  expect(actions().some(action => action.label === "Read disclosed selected prediction chain bytes")).toBe(false);
});

it("retains answer-free future-stop coordinates while refusing embedded prior visible values", () => {
  const copy = structuredClone(recorded) as unknown as Record<string, unknown>;
  const history = copy.history as { ordering: {event: Record<string, unknown>}[] };
  const event = history.ordering.find(entry => entry.event.kind === "sealed")!.event;
  event.future_stop = { run: "retained-host-run", thread: 1, frame: 2, hit: 1, prior_visible_digest: "a".repeat(64) };
  validatePredictionHistory(copy);
  (event.future_stop as Record<string, unknown>).prior_visible = "PRIVATE PREVIOUS STATE";
  expect(() => validatePredictionHistory(copy)).toThrow();
});
