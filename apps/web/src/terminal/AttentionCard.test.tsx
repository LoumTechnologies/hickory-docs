import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import type { TerminalSession } from "../api/types";
import { AttentionCard } from "./AttentionCard";

afterEach(cleanup);

function session(over: Partial<TerminalSession> = {}): TerminalSession {
  return {
    id: "term-1",
    title: "refactor the parser",
    cwd: "/w/app",
    monitor: false,
    state: "needs-you",
    since_ms: 0,
    branch: "side-task",
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
    ...over,
  };
}

function show(over: Partial<TerminalSession> = {}) {
  const onAnswer = vi.fn();
  render(
    <AttentionCard
      session={session(over)}
      position={1}
      total={3}
      onAnswer={onAnswer}
      onOpen={vi.fn()}
      onInterrupt={vi.fn()}
      onNext={vi.fn()}
      onDismiss={vi.fn()}
    />,
  );
  return onAnswer;
}

describe("the attention card", () => {
  it("offers the program's own choices, and sends exactly what they carry", () => {
    const onAnswer = show({
      prompt: {
        question: "May I write src/main.rs?",
        source: "declared",
        choices: [
          { label: "Allow", send: "1\n", destructive: false },
          { label: "Deny", send: "2\n", destructive: false },
        ],
      },
    });

    expect(screen.getByText("May I write src/main.rs?")).toBeTruthy();
    fireEvent.click(screen.getByText("Allow"));
    expect(onAnswer).toHaveBeenCalledWith("1\n");
  });

  it("never invents buttons for a prompt it only guessed at", () => {
    // We recognised the shape of a question. We do not know what the answers
    // mean, so the card offers a line to type — and says why.
    show({
      prompt: { question: "Delete build/? [y/N]", source: "guessed", choices: [] },
    });

    expect(screen.queryByText("Allow")).toBeNull();
    expect(screen.getByLabelText(/type the answer/i)).toBeTruthy();
  });

  it("marks a destructive choice apart from a safe one", () => {
    show({
      prompt: {
        question: "Force push?",
        source: "declared",
        choices: [
          { label: "Cancel", send: "n\n", destructive: false },
          { label: "Force push", send: "y\n", destructive: true },
        ],
      },
    });
    expect(screen.getByText("Force push").className).toContain("btn-danger");
    expect(screen.getByText("Cancel").className).not.toContain("btn-danger");
  });

  it("says what is left to decide when a finished session still has changes", () => {
    show({ state: "finished", prompt: null, dirty: true, exit_code: 0 });
    expect(screen.getByText(/uncommitted changes/i)).toBeTruthy();
  });

  it("names the session and its place in the queue", () => {
    show();
    expect(screen.getByText("refactor the parser")).toBeTruthy();
    expect(screen.getByText("1 of 3")).toBeTruthy();
  });
});
