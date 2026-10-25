<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ReviewPanel.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Review investigations link observed agent work without resolving human review](../../../../../../guarantees/authoring/review-investigations-link-observed-agent-work/contract.md)
- [Review threads preserve the questioned target](../../../../../../guarantees/authoring/review-threads-preserve-the-questioned-target/contract.md)

Captured SHA-256: `502c6eb28d823874aa98d3b26aaf7cadef9bb5ac2b5644851d6259e36d1b5de0`.

<book:entry path="apps/web/src/components/ReviewPanel.test.tsx">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { afterEach, expect, it, vi } from "vitest";
const request = vi.hoisted(() => vi.fn());
vi.mock("../api/client", () => ({ request }));
import { ReviewPanel } from "./ReviewPanel";
import { openReview } from "../lib/reviews";
afterEach(() => { cleanup(); request.mockReset(); });
// Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
it("attributes an agent comment without granting human resolution", async () => {
  const target = { kind: "claim", path: "spec.md", quote: "Claim", source_hash: "a".repeat(64), lines: [1, 1] };
  const original = { id: "original", thread: "thread", target, action: "comment", body: "Human criterion", actor: "user-declared", recorded_at: "1" };
  request.mockImplementation(async (method: string) => method === "GET" ? { scope: "workspace", events: [original], targets: [] } : { id: "agent-comment", thread: "thread", actor: "agent-declared" });
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><ReviewPanel /></UiActionContext.Provider>);
  fireEvent.click(screen.getByRole("button", { name: "Review threads" }));
  fireEvent.click(await screen.findByRole("button", { name: "spec.md: Human criterion" }));
  let n = 0;
  const invoke = async (label: string, arguments_ = {}) => {
    const parent = registry.groups().groups.find(group => group.startsWith("reviews:"))!;
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); }); return result;
  };
  await invoke("Set review comment or reason", { text: "Actual agent finding" });
  expect(await invoke("Resolve after human review")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(await invoke("Save review comment")).toMatchObject({ status: "result", outcome: { value: { id: "agent-comment", actor: "agent-declared" } } });
  expect(request).toHaveBeenCalledWith("POST", "/api/reviews", { scope: "workspace", thread: "thread", target: null, action: "comment", body: "Actual agent finding", agent: true });
  expect(request.mock.calls.some(call => call[0] === "POST" && call[2].action === "resolve")).toBe(false);
});
// Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
it("posts the captured target and workspace scope and retains its quotation", async () => {
  request.mockImplementation(async (method: string) => method === "GET" ? { scope:"workspace",events:[],targets:[] } : {thread:"review-1"});
  render(<ReviewPanel />);
  const target={kind:"claim",path:"spec.md",quote:"The whole operation is bounded",source_hash:"a".repeat(64),lines:[1,1] as [number,number]};
  openReview(target);
  await screen.findByText(target.quote);
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET","/api/reviews"));
  fireEvent.change(screen.getByLabelText("Comment or reason"),{target:{value:"Only each attempt is bounded"}});
  fireEvent.click(screen.getByRole("button",{name:"Save comment"}));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST","/api/reviews",{scope:"workspace",thread:null,target,action:"comment",body:"Only each attempt is bounded"}));
});

// Guarantee: docs/guarantees/authoring/review-investigations-link-observed-agent-work.md
it("dispatches only to the explicitly selected conversation and keeps human review open", async () => {
  const { registerInvestigationConversation } = await import("../lib/reviewInvestigations");
  const first = vi.fn(async (_prompt: string) => "turn-a");
  const second = vi.fn(async (_prompt: string) => "turn-b");
  const unregisterA = registerInvestigationConversation({ id: "conversation-a", label: "Conversation A", send: first });
  const unregisterB = registerInvestigationConversation({ id: "conversation-b", label: "Conversation B", send: second });
  const target = { kind: "claim", path: "spec.md", quote: "Total deadline", source_hash: "a".repeat(64), lines: [1, 1] };
  const events = [{ id: "original", thread: "thread-1", target, action: "comment", body: "Only each attempt is bounded", actor: "user-declared", recorded_at: "1" }];
  request.mockImplementation(async (method: string) => method === "GET" ? {
    scope: "workspace", events, targets: [{ thread: "thread-1", current_state: "ambiguous" }], investigations: [],
  } : { thread: "thread-1" });
  try {
    render(<ReviewPanel />);
    fireEvent.click(screen.getByRole("button", { name: "Review threads" }));
    fireEvent.click(await screen.findByRole("button", { name: /spec.md: Only each attempt/ }));
    expect(first).not.toHaveBeenCalled(); expect(second).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Investigation conversation"), { target: { value: "conversation-b" } });
    fireEvent.click(screen.getByRole("button", { name: "Send investigation to selected conversation" }));
    await waitFor(() => expect(second).toHaveBeenCalledTimes(1));
    expect(first).not.toHaveBeenCalled();
    expect(second.mock.calls[0]?.[0]).toContain("ambiguous");
    expect(second.mock.calls[0]?.[0]).toContain("Only each attempt is bounded");
    await waitFor(() => expect(request.mock.calls.some((call: unknown[]) => call[0] === "POST" &&
      String((call[2] as { body: string }).body).includes('"turn":"turn-b"'))).toBe(true));
    expect(request.mock.calls.some((call: unknown[]) => call[0] === "POST" &&
      (call[2] as { action: string }).action === "resolve")).toBe(false);
  } finally { unregisterA(); unregisterB(); }
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares prepared-prompt copying and retains an explicit browser-permission request", async () => {
  const originalClipboard = Object.getOwnPropertyDescriptor(navigator, "clipboard");
  const copy = vi.fn().mockRejectedValue(new Error("Permission denied"));
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: copy } });
  try {
    const target = { kind: "claim", path: "spec.md", quote: "Claim", source_hash: "a".repeat(64), lines: [1, 1] };
    request.mockResolvedValue({ scope: "workspace", events: [{ id: "original", thread: "thread", target, action: "comment", body: "Human criterion", actor: "user-declared", recorded_at: "1" }], targets: [] });
    const registry = new UiActions({ workspace: "practice", window: "copy-prompt" }, () => true, async () => {});
    render(<UiActionContext.Provider value={registry}><ReviewPanel /></UiActionContext.Provider>);
    fireEvent.click(screen.getByRole("button", { name: "Review threads" }));
    fireEvent.click(await screen.findByRole("button", { name: "spec.md: Human criterion" }));
    fireEvent.click(screen.getByRole("button", { name: "Prepare agent investigation" }));
    const parent = registry.groups().groups.find(group => group.startsWith("reviews:"))!;
    const command = registry.observe(parent, 64).actions.find(action => action.label === "Copy prepared investigation prompt")!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { kind: "agent", id: "predictor" }, { handle: command.handle, arguments: {}, request_id: "copy-requires-gesture" }); });
    expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input", request: "review-prompt-clipboard" } });
    expect(screen.getByRole("alert").textContent).toContain("Permission denied");
    copy.mockResolvedValue(undefined);
    fireEvent.click(screen.getByRole("button", { name: "Copy prompt" }));
    await waitFor(() => expect(copy).toHaveBeenCalledTimes(2));
    expect(copy.mock.calls[0][0]).toBe(copy.mock.calls[1][0]);
    expect(copy.mock.calls[1][0]).toContain("Human criterion");
    expect(request.mock.calls.every(call => call[0] === "GET")).toBe(true);
  } finally {
    if (originalClipboard) Object.defineProperty(navigator, "clipboard", originalClipboard);
    else Reflect.deleteProperty(navigator, "clipboard");
  }
});
</book:entry>

</book:doc>
