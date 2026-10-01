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
        "`.gitattributes` routes `*.md` at the `hick` merge driver, but this clone has not defined it — so git is SILENTLY falling back to its line merge. Next step: run `hick init` in this repository.",
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
    expect(screen.getByRole("button", { name: "Run hick init" })).toBeTruthy();
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

// docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md
describe("the banner's button", () => {
  it("runs hick init in the engine and says what it wrote", async () => {
    answer({});
    const init = vi.spyOn(api, "initRepository").mockResolvedValue({
      changed: {
        hook: true,
        gitignore: false,
        gitattributes: false,
        merge_driver: true,
        agents_md: false,
        mcp_json: false,
      },
      hook_path: ".git/hooks/pre-commit",
      status: { repository: true, attributes: true, configured: true, summary: "ok" },
      ok: true,
    });
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Run hick init" }));
    });
    expect(init).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(screen.getByText(/hick init ran: hook, merge.driver/)).toBeTruthy());
    expect(screen.queryByText(/SILENTLY/)).toBeNull();
  });

  it("keeps the warning and says why when hick init fails", async () => {
    answer({});
    vi.spyOn(api, "initRepository").mockRejectedValue(new Error("not a git work tree"));
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Run hick init" }));
    });
    await waitFor(() => expect(screen.getByText(/hick init failed: not a git work tree/)).toBeTruthy());
    expect(screen.getByRole("button", { name: "Run hick init" })).toBeTruthy();
  });
});
