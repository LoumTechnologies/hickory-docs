// docs/guarantees/collaboration/hick-documents-merge-through-hick.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { api } from "../api/client";
import { MergeDriverNotice } from "./MergeDriverNotice";

const answer = (over: Partial<{ repository: boolean; attributes: boolean; configured: boolean }>) =>
  vi.spyOn(api, "mergeDriver").mockResolvedValue({
    status: {
      repository: true,
      attributes: true,
      configured: false,
      summary:
        "`.gitattributes` routes `*.hick` at the `hick` merge driver, but this clone has not defined it — so git is SILENTLY falling back to its line merge. Next step: run `hick init` in this repository.",
      ...over,
    },
    ok: false,
  });

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("a clone whose merge driver is not defined", () => {
  it("is told at open, because a check at commit time tells you after the damage", async () => {
    answer({});
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    expect(screen.getByText(/hick init/)).toBeTruthy();
  });

  it("can be dismissed for this window", async () => {
    answer({});
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    act(() => {
      fireEvent.click(screen.getByRole("button", { name: /dismiss/i }));
    });
    await waitFor(() => expect(screen.queryByText(/SILENTLY/)).toBeNull());
  });
});

describe("a clone that is wired up", () => {
  it("says nothing — a banner confirming things work is one people learn to skip", async () => {
    vi.spyOn(api, "mergeDriver").mockResolvedValue({
      status: { repository: true, attributes: true, configured: true, summary: "fine" },
      ok: true,
    });
    const { container } = render(<MergeDriverNotice />);
    await waitFor(() => expect(container.textContent).toBe(""));
  });

  it("says nothing for a folder that is not a repository", async () => {
    vi.spyOn(api, "mergeDriver").mockRejectedValue(new Error("not a repository"));
    const { container } = render(<MergeDriverNotice />);
    await waitFor(() => expect(container.textContent).toBe(""));
  });
});
