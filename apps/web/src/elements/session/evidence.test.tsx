// Guarantee: docs/guarantees/agent/historical-evidence-stays-historical.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { SourceChips } from "./view";
import { openReading, publishReading } from "../../lib/readingViews";
import { openLocation } from "../../lib/revealLine";
vi.mock("../../lib/readingViews", () => ({ publishReading: vi.fn(), openReading: vi.fn() }));
vi.mock("../../lib/revealLine", () => ({ openLocation: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it("opens the captured historical passage with its citing paragraph and comparison facts", () => {
  render(<SourceChips links={[{ family: "declared", span: [10,40], lines: [3,3], to: {path:"meeting.md",lines:[2,2]},title:"Declared reference",historical: {record_kind:"read-snapshot",session_line:2,path:"meeting.md",content:"Approval pending",sha256:"old",first_line:2,last_line:2,current_state:"changed",claim:'Legal said “approved”.',quoted_literals:[{literal:"approved",present_in_recorded_passage:false}]} }]} />);
  fireEvent.click(screen.getByRole("button"));
  const view = vi.mocked(publishReading).mock.calls[0][1];
  expect(view.after).toContain("Approval pending");
  expect(view.after).toContain('Legal said “approved”.');
  expect(view.after).toContain('"approved": absent from recorded passage');
  expect(view.status).toContain("semantic support not assessed");
  expect(view.after).toBe(view.before);
  expect(openReading).toHaveBeenCalled();
  expect(openLocation).not.toHaveBeenCalled();
});
it("uses a report label for adapter assertions", () => {
  render(<SourceChips links={[{family:"declared",span:[1,2],lines:[1,1],to:{path:"file.md"},title:"ACP-reported file edit"}]} />);
  expect(screen.getByText("reports")).toBeTruthy();
});
