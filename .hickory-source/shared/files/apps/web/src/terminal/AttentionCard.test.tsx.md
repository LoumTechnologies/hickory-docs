<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/terminal/AttentionCard.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Answer Does Not Cost You Your Place](../../../../../../guarantees/terminal/an-answer-does-not-cost-you-your-place/contract.md)
- [Turbo Never Answers A Prompt It Did Not Parse](../../../../../../guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse/contract.md)

Captured SHA-256: `f37e1253ec21f9020c233ad116fe7a78cf7e04953be090def322ae19bfe52881`.

<book:entry path="apps/web/src/terminal/AttentionCard.test.tsx">
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

it("shares prompt reading and preparation while preserving actual human decision authority", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const receipts: import("../lib/uiActions").UiReceipt[] = [];
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async receipt => { receipts.push(receipt); });
  const answer = vi.fn().mockResolvedValue(false), open = vi.fn(), next = vi.fn(), dismiss = vi.fn(), interrupt = vi.fn().mockResolvedValue(true);
  const panel = (question: string) => <UiActionContext.Provider value={registry}><AttentionCard session={session({ prompt: { question, choices: [], source: "guessed" } })} position={1} total={2}
    onAnswer={answer} onOpen={open} onNext={next} onDismiss={dismiss} onInterrupt={interrupt} /></UiActionContext.Provider>;
  const view = render(panel("Question one?"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  const reading = await run("Read terminal attention card bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes))).toContain("Question one?");
  expect(await run("Prepare terminal attention reply draft", { text: "prepared" })).toMatchObject({ status: "result", outcome: { value: { answer_sent: false, approval: false } } });
  expect((screen.getByLabelText(/type the answer/i) as HTMLInputElement).value).toBe("prepared");
  fireEvent.change(screen.getByLabelText(/type the answer/i), { target: { value: "reviewed" } });
  expect(screen.getByRole("note").textContent).toContain("Prepared by an agent");
  const required = await run("Send attention prompt reply") as { outcome: { request: string } };
  expect(required).toMatchObject({ status: "result", outcome: { status: "requires_input", request: expect.stringContaining("terminal-decision:") } });
  expect(answer).not.toHaveBeenCalled();
  await act(async () => { fireEvent.submit(screen.getByRole("button", { name: "Send" }).closest("form")!); });
  expect(answer).toHaveBeenCalledExactlyOnceWith("reviewed\n");
  expect((screen.getByLabelText(/type the answer/i) as HTMLInputElement).value).toBe("reviewed");
  expect(screen.getByRole("alert").textContent).toContain("refused");
  answer.mockResolvedValueOnce(true);
  await act(async () => { fireEvent.submit(screen.getByRole("button", { name: "Send" }).closest("form")!); });
  expect(receipts.some(receipt => receipt.actor.kind === "human" && receipt.stage === "result" && JSON.stringify(receipt.result).includes(required.outcome.request)
    && JSON.stringify(receipt.result).includes('"host_request_completed":true') && JSON.stringify(receipt.result).includes('"approval_attested":false'))).toBe(true);
  const old = actions().find(action => action.label === "Send attention prompt reply")!.handle;
  view.rerender(panel("Question two?"));
  expect((screen.getByLabelText(/type the answer/i) as HTMLInputElement).value).toBe("");
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "changed-question" })).not.toMatchObject({ status: "result" });
  await run("Open attention terminal"); await run("Show next terminal attention request"); await run("Interrupt attention terminal"); await run("Dismiss terminal attention card");
  expect(open).toHaveBeenCalledTimes(1); expect(next).toHaveBeenCalledTimes(1); expect(interrupt).toHaveBeenCalledTimes(1); expect(dismiss).toHaveBeenCalledTimes(1);
});
</book:entry>

</book:doc>
