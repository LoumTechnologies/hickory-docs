<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/AgentSettings.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c3e11636cd88"></a>
## `apps/web/src/views/AgentSettings.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A desktop app bundles its own Python agent](../../../../../guarantees/agent/a-desktop-app-bundles-its-python-agent/contract.md), [Separate model snippets from trusted host operations](../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md).

<a id="f-c3e11636cd88-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c3e11636cd88-expression_statement">
// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { acpApi } from "../api/acp";
import { AgentSettings } from "./AgentSettings";
vi.mock("../api/acp", () => ({ acpApi: {
  catalogue: vi.fn().mockResolvedValue({ agents: [{ id: "python", name: "Hickory Python", command: "/old/venv/bin/python", args: [] }, { id: "codex", name: "Codex", command: "codex-acp", args: [], private_prediction_args: ["--fresh-private"], private_prediction_auth_method: "api-key" }] }),
  pythonSettings: vi.fn().mockResolvedValue({ python: "/old/venv/bin/python", available: true }),
  savePython: vi.fn(), save: vi.fn(),
} }));
</storycode:copy>
<a id="f-c3e11636cd88-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c3e11636cd88-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-c3e11636cd88-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c3e11636cd88-expression_statement-3">
it("uses the shared runtime check and refuses invalid ACP JSON before saving", async () => {
  vi.mocked(acpApi.savePython).mockResolvedValue({ python: "/new/venv/bin/python", available: true });
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><AgentSettings /></UiActionContext.Provider>);
  await waitFor(() => expect((screen.getByLabelText("Python executable") as HTMLInputElement).value).toBe("/old/venv/bin/python"));
  let n = 0;
  const invoke = async (label: string, args = {}) => {
    const action = registry.observe("agent-settings").actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); }); return result;
  };
  await invoke("Set Python runtime draft", { executable: "/new/venv/bin/python" });
  expect(await invoke("Save and check Python runtime")).toMatchObject({ status: "result", outcome: { value: { python: "/new/venv/bin/python", available: true } } });
  expect(acpApi.savePython).toHaveBeenCalledWith("/new/venv/bin/python");
  await invoke("Set ACP agent commands draft", { json: "invalid JSON" });
  expect(await invoke("Save ACP agent commands")).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(acpApi.save).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-c3e11636cd88-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-c3e11636cd88-expression_statement-4">
it("checks and stores the Python runtime separately from ACP commands", async () => {
  vi.mocked(acpApi.savePython).mockResolvedValue({ python: "/new/venv/bin/python", available: true });
  render(<AgentSettings />);
  await waitFor(() => expect((screen.getByRole("textbox", { name: "Python executable" }) as HTMLInputElement).value).toBe("/old/venv/bin/python"));
  expect((screen.getByRole("textbox", { name: "Agent commands" }) as HTMLTextAreaElement).value).not.toContain('"python"');
  const configured = JSON.parse((screen.getByRole("textbox", { name: "Agent commands" }) as HTMLTextAreaElement).value)[0];
  expect(configured.private_prediction_args).toEqual(["--fresh-private"]);
  expect(configured.private_prediction_auth_method).toBe("api-key");
  fireEvent.change(screen.getByRole("textbox", { name: "Python executable" }), { target: { value: "/new/venv/bin/python" } });
  fireEvent.click(screen.getByRole("button", { name: "Save Python runtime" }));
  await screen.findByRole("status");
  expect(acpApi.savePython).toHaveBeenCalledWith("/new/venv/bin/python");
  expect(acpApi.save).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-c3e11636cd88-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-c3e11636cd88-expression_statement-5">
it("keeps a failed runtime check visible without claiming it was saved", async () => {
  vi.mocked(acpApi.savePython).mockRejectedValue(new Error("Confinement unavailable"));
  render(<AgentSettings />);
  fireEvent.click(screen.getByRole("button", { name: "Save Python runtime" }));
  expect((await screen.findByRole("alert")).textContent).toBe("Confinement unavailable");
  expect(screen.queryByRole("status")).toBeNull();
});
</storycode:copy>
<a id="f-c3e11636cd88-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-c3e11636cd88-expression_statement-6">
// Guarantee: docs/guarantees/agent/a-desktop-app-bundles-its-python-agent.md
it("names the runtime bundled with the app without pretending it was configured", async () => {
  const bundled = "/Applications/Hickory Docs.app/Contents/Resources/python-agent/bin/python";
  vi.mocked(acpApi.pythonSettings).mockResolvedValueOnce({ python: "", available: true, bundled });
  render(<AgentSettings />);
  await screen.findByText(new RegExp("bundled with this app"));
  const field = screen.getByRole("textbox", { name: "Python executable" }) as HTMLInputElement;
  expect(field.value).toBe("");
  expect(field.placeholder).toBe(bundled);
});
</storycode:copy>

### Assemble `apps/web/src/views/AgentSettings.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/AgentSettings.test.tsx"><storycode:paste select="#f-c3e11636cd88-expression_statement"/><storycode:paste select="#f-c3e11636cd88-expression_statement-2"/><storycode:paste select="#f-c3e11636cd88-expression_statement-3"/><storycode:paste select="#f-c3e11636cd88-expression_statement-4"/><storycode:paste select="#f-c3e11636cd88-expression_statement-5"/><storycode:paste select="#f-c3e11636cd88-expression_statement-6"/></storycode:file>

</storycode:doc>
