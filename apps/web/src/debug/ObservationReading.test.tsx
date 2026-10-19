import readingFixture from "./fixtures/reading.json";
import { fireEvent, render, screen, cleanup } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { ObservationReading } from "./ObservationReading";
import { selectedObservations, type ObservationReading as Reading } from "./observations";
afterEach(cleanup);
const source = "BREAKPOINT HERE\nvalue += 1;\n";
const fixture = (): Reading => structuredClone(readingFixture);
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
it("shares only explicitly expanded evidence and preserves historical and changed state", () => {
  const reading = fixture();
  const view = render(<ObservationReading reading={reading} historical={false} />);
  expect(selectedObservations("demo.md", source)).toEqual([]);
  const details = view.container.querySelector("details")!;
  details.open = true; fireEvent(details, new Event("toggle"));
  expect(selectedObservations("demo.md", source)[0]).toMatchObject({ state: "captured", evidence: ["opaque-E"], reading });
  expect(screen.getByText(/value \+= 1/).closest("[data-location]")?.getAttribute("data-location")).toBe("opaque-L");
  expect(view.container.querySelectorAll("[data-evidence]")).toHaveLength(1);
  expect(selectedObservations("demo.md", "edited")[0].state).toBe("changed");
  view.rerender(<ObservationReading reading={reading} historical />);
  expect(selectedObservations("demo.md", source)[0].state).toBe("historical");
  expect(screen.getByText(/historical capture/)).toBeTruthy();
  view.unmount();
  expect(selectedObservations("demo.md", source)).toEqual([]);
});

it("sends opaque location and scope handles without projected line numbers", async () => {
  const { DebugClient } = await import("./client");
  const messages: unknown[] = [];
  const client = new DebugClient({ send: frame => messages.push(JSON.parse(new TextDecoder().decode(frame.subarray(1)))) });
  const reading = fixture();
  client.located("fixture-session", reading.locations[0].id, reading.scope, "jump");
  expect(messages).toEqual([{ op: "located", session: "fixture-session", location: "opaque-L", scope: reading.scope, how: "jump" }]);
  expect(messages[0]).not.toHaveProperty("line");
});

it("keeps independently expanded stops selected when another view closes", () => {
  const old = fixture(); old.scope.stop = "old-stop"; old.annotations[0].scope.stop = "old-stop";
  const first = render(<ObservationReading reading={old} historical />);
  const second = render(<ObservationReading reading={fixture()} historical={false} />);
  for (const view of [first, second]) {
    const details = view.container.querySelector("details")!;
    details.open = true; fireEvent(details, new Event("toggle"));
  }
  expect(selectedObservations("demo.md", source)).toHaveLength(2);
  first.unmount();
  expect(selectedObservations("demo.md", source)).toHaveLength(1);
  expect(selectedObservations("demo.md", source)[0].state).toBe("captured");
  second.unmount();
  expect(selectedObservations("demo.md", source)).toEqual([]);
});

// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("pins collapsed observations without changing evidence or precision", async () => {
  const reading = fixture();
  const view = render(<ObservationReading reading={reading} historical={false} />);
  fireEvent.click(screen.getByRole("checkbox", { name: /Pin observations/ }));
  const { observationContext } = await import("./observations");
  const context = observationContext("demo.md", source)[0];
  expect(context.reading.scope).toEqual(reading.scope);
  expect(context.reading.excerpts[0].location).toEqual(reading.locations[0]);
  expect(context.reading.annotations).toEqual(reading.annotations);
  expect(view.container.querySelector("details")!.open).toBe(false);
  fireEvent.click(screen.getByRole("checkbox", { name: /Pin observations/ }));
  expect(observationContext("demo.md", source)).toEqual([]);
});

// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("serializes synthetic output, exception and profile records without losing run or measurement precision", async () => {
  const reading = fixture();
  reading.annotations = [
    { ...reading.annotations[0], evidence: "output", payload: { kind: "output", text: "bounded", truncated: true, omitted_bytes: 120 } },
    { ...reading.annotations[0], evidence: "exception", payload: { kind: "exception", message: "failure", precision: "block" } },
    { ...reading.annotations[0], evidence: "profile", payload: { kind: "profile", interval_ns: 1000, aggregation: "sampled", precision: "statistical" } },
  ];
  const view = render(<ObservationReading reading={reading} historical />);
  fireEvent.click(screen.getByRole("checkbox", { name: /Pin observations/ }));
  const { observationContext } = await import("./observations");
  const selected = observationContext("demo.md", source)[0];
  expect(selected.state).toBe("historical");
  expect(selected.reading.source.revision).toBe(reading.source.revision);
  expect(selected.reading.annotations).toEqual(reading.annotations);
  expect(selected.reading.scope).toEqual(reading.scope);
  view.unmount();
});
