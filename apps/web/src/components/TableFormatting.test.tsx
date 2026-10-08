// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { TablePanel } from "./TablePanel";
import { MarkdownTable } from "./MarkdownTable";
import { api } from "../api/client";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

it.each(["python", "javascript"])("pastes money as numeric input to %s formulas and formats their result", async language => {
  const evaluate = vi.spyOn(api, "evaluateFormulas").mockResolvedValue({ values: { B3: "2469" }, errors: {} });
  const save = vi.fn();
  function Grid() {
    const [source, setSource] = useState("name,amount\na,0\ntotal,=B2 * 2\n");
    const [formats, setFormats] = useState("B3:USD");
    return <TablePanel source={source} formats={formats} language={language}
      onChange={(next, fmt = "") => { save(next, fmt); setSource(next); setFormats(fmt); }} />;
  }
  render(<Grid />);
  fireEvent.click(screen.getByText("0"));
  fireEvent.paste(screen.getByTestId("table-panel"), { clipboardData: { getData: (type: string) => type === "text/plain" ? "$1,234.50" : "" } });
  expect(save).toHaveBeenLastCalledWith("name,amount\na,1234.50\ntotal,=B2 * 2\n", "B2:USD B3:USD");
  expect(screen.getByText("$1,234.50")).toBeTruthy();
  await waitFor(() => expect(evaluate).toHaveBeenCalledWith(language, [["name", "amount"], ["a", "1234.50"], ["total", "=B2 * 2"]]));
  await waitFor(() => expect(screen.getByText("$2,469.00")).toBeTruthy());
  fireEvent.doubleClick(screen.getByText("$1,234.50"));
  expect((screen.getByRole("textbox", { name: "Cell contents" }) as HTMLInputElement).value).toBe("1234.50");
});

it("saves Markdown formats adjacent to numeric cells and reloads them", () => {
  const save = vi.fn();
  const { rerender } = render(<MarkdownTable source={"| Cost |\n| ---: |\n| 1234.567 |"} onChange={save} />);
  fireEvent.click(screen.getByText("1234.567"));
  fireEvent.change(screen.getByLabelText("Cell number format"), { target: { value: "USD" } });
  const source = '<!-- hickory-table-formats: A2:USD -->\n| Cost |\n| ---: |\n| 1234.567 |';
  expect(save).toHaveBeenLastCalledWith(source);
  rerender(<MarkdownTable source={source} onChange={save} />);
  expect(screen.getByText("$1,234.57")).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Cell number format"), { target: { value: "general" } });
  expect(save.mock.lastCall?.[0]).toContain("A2:general");
  expect(save.mock.lastCall?.[0]).toContain("1234.567");
});
