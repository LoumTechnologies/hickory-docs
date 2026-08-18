// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ScratchpadPane } from "./ScratchpadPane";

const saveScratchpad = vi.fn();
vi.mock("../api/client", () => ({
  api: {
    saveScratchpad: (text: string) => saveScratchpad(text),
  },
}));

afterEach(() => {
  cleanup();
  saveScratchpad.mockReset();
});

describe("the scratchpad", () => {
  it("will not save nothing", async () => {
    render(<ScratchpadPane />);
    const button = screen.getByRole("button", {
      name: /save as note/i,
    }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it("saves what was typed and says where it landed", async () => {
    saveScratchpad.mockResolvedValue({ path: "2026-08-18-standup.hick" });
    render(<ScratchpadPane />);

    fireEvent.change(screen.getByLabelText(/scratchpad/i), {
      target: { value: "All green." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    await waitFor(() =>
      expect(saveScratchpad).toHaveBeenCalledWith("All green."),
    );
    expect(await screen.findByText(/2026-08-18-standup\.hick/)).toBeTruthy();
  });

  it("clears itself after a save, so the next thought starts empty", async () => {
    saveScratchpad.mockResolvedValue({ path: "note.hick" });
    render(<ScratchpadPane />);

    const box = screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: "Something" } });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    await waitFor(() => expect(box.value).toBe(""));
  });

  it("shows the refusal instead of losing the text", async () => {
    // The refusals this can hit all carry a next step, and the words the
    // person typed are still in the box to act on.
    saveScratchpad.mockRejectedValue(
      new Error("this text contains `<hick:` — Next step: reword it"),
    );
    render(<ScratchpadPane />);

    const box = screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: "I used a hick tag" } });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    expect(await screen.findByText(/Next step/)).toBeTruthy();
    expect(box.value).toBe("I used a hick tag");
  });
});
