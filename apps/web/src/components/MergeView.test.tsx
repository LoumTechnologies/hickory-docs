import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";

import { MergeView } from "./MergeView";

afterEach(() => {
  cleanup();
});

const view = (props: Partial<React.ComponentProps<typeof MergeView>> = {}) =>
  render(
    <MergeView
      path="notes.hick"
      base={"one\ntwo\nthree\n"}
      ours={"one\nOURS\nthree\n"}
      theirs={"one\nTHEIRS\nthree\n"}
      onAccept={() => {}}
      onCancel={() => {}}
      {...props}
    />,
  );

describe("the merge view says how much of this is your problem", () => {
  it("leads with the count that has to be answered", () => {
    // Before reading a word of the merge, the reader needs to know whether
    // this is two decisions or forty.
    view();
    expect(screen.getByRole("status").textContent).toMatch(/1 of 1 still needs you/i);
  });

  it("says so plainly when there is nothing to decide", () => {
    view({ ours: "one\nOURS\nthree\n", theirs: "one\ntwo\nthree\n" });
    expect(screen.getByRole("status").textContent).toMatch(/merged cleanly/i);
    expect(screen.queryAllByRole("group", { name: /conflict/i })).toHaveLength(0);
  });

  it("counts down as answers are given", () => {
    view();
    fireEvent.click(screen.getByRole("button", { name: "Keep theirs" }));
    expect(screen.getByRole("status").textContent).toMatch(/0 of 1 still need you/i);
  });
});

describe("the merge view never takes a side silently", () => {
  it("shows an auto-merged region and names who it came from", () => {
    // A merge tool that silently took a side is a merge tool nobody trusts
    // twice.
    view({ ours: "ONE\ntwo\nthree\n", theirs: "one\ntwo\nTHREE\n" });
    const labels = screen.getAllByText(/^Merged —/).map((el) => el.textContent);
    expect(labels.join(" ")).toMatch(/only your unsaved changes touched this/i);
    expect(labels.join(" ")).toMatch(/only the file on disk changed this/i);
  });

  it("shows both sides of a conflict, labelled", () => {
    view();
    const conflict = screen.getByRole("group", { name: /conflict 1/i });
    expect(within(conflict).getByText("Your unsaved changes")).toBeTruthy();
    expect(within(conflict).getByText("The file on disk")).toBeTruthy();
  });
});

describe("answering a conflict", () => {
  it("produces the chosen text on accept", () => {
    const onAccept = vi.fn();
    view({ onAccept });
    fireEvent.click(screen.getByRole("button", { name: "Keep theirs" }));
    fireEvent.click(screen.getByRole("button", { name: /accept/i }));
    expect(onAccept).toHaveBeenCalledWith("one\nTHEIRS\nthree\n");
  });

  it("keeps our side for anything left undecided, and says so on the button", () => {
    // Accepting with an unanswered conflict is a real choice, not a mistake to
    // block: it means "everything I did not answer stays mine".
    const onAccept = vi.fn();
    view({ onAccept });
    const accept = screen.getByRole("button", { name: /accept, keeping mine where undecided/i });
    fireEvent.click(accept);
    expect(onAccept).toHaveBeenCalledWith("one\nOURS\nthree\n");
  });

  it("lets an answer be taken back by pressing it again", () => {
    view();
    const theirs = screen.getByRole("button", { name: "Keep theirs" });
    fireEvent.click(theirs);
    expect(theirs.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(theirs);
    expect(theirs.getAttribute("aria-pressed")).toBe("false");
  });

  it("cancels without touching anything", () => {
    const onCancel = vi.fn();
    const onAccept = vi.fn();
    view({ onCancel, onAccept });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onAccept).not.toHaveBeenCalled();
  });
});

describe("with no common ancestor", () => {
  it("says why every difference is a question", () => {
    view({ base: "" });
    expect(screen.getByText(/no common ancestor/i)).toBeTruthy();
  });

  it("offers no 'keep neither', because there is no before to go back to", () => {
    view({ base: "" });
    expect(screen.queryByRole("button", { name: "Keep neither" })).toBeNull();
  });
});
