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
