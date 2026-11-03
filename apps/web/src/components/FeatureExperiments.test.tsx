import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";
import { FeatureExperiments } from "./FeatureExperiments";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));

afterEach(() => { cleanup(); vi.clearAllMocks(); });

function Composer({ backend = "codex", running = false }) {
  const [features, setFeatures] = useState<string[]>([]);
  return <FeatureExperiments doc="workspace" backend={backend} running={running} features={features} onFeatures={setFeatures} />;
}

describe("in-app feature measurement consent and evidence", () => {
  it("exports fresh recorded outcomes through the application transport", async () => {
    const initial = { records:[], features:[], coverage:[], storage:"local", limitations:"Recorded observations" };
    const latest = { ...initial, records:[{ id:"failed", document:"elsewhere", prompt:"Task", status:"error", created_at:"today", pairs:[] }] };
    vi.mocked(request).mockResolvedValueOnce(initial).mockResolvedValueOnce(latest);
    const create = vi.fn((_blob: Blob) => "blob:feature-dataset"), revoke = vi.fn();
    const OriginalURL = URL;
    vi.stubGlobal("URL", Object.assign(class extends OriginalURL {}, { createObjectURL:create, revokeObjectURL:revoke }));
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
    try {
      render(<Composer />);
      await act(async () => fireEvent.click(screen.getByRole("button", { name:"Measurements and dataset" })));
      await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
      await act(async () => fireEvent.click(screen.getByRole("button", { name:"Export dataset" })));
      expect(request).toHaveBeenCalledTimes(2);
      expect(click).toHaveBeenCalledOnce();
      const blob = create.mock.calls[0][0] as Blob;
      const text = await new Promise<string>((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result)); reader.onerror = reject; reader.readAsText(blob); });
      expect(JSON.parse(text)).toEqual(latest);
    } finally { click.mockRestore(); vi.unstubAllGlobals(); }
  });

  // Contract: .hickory-source/guarantees/agent/feature-experiments/contract.md
  it("starts unchecked, exposes extra run count, and lets the human select an ablation", async () => {
    render(<Composer />);
    const checkbox = screen.getByRole("checkbox", { name: "Measure this request with and without Hickory features" }) as HTMLInputElement;
    expect(checkbox.checked).toBe(false);
    await act(async () => fireEvent.click(checkbox));
    expect(checkbox.checked).toBe(true);
    expect(screen.getByText(/2 extra Codex runs/)).toBeTruthy();
    await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Lineage tools for generated code" })));
    expect(screen.getByText(/4 extra Codex runs/)).toBeTruthy();
    expect(request).not.toHaveBeenCalled();
  });

  it("requires a human to authorize extra runs through the agent UI bridge", async () => {
    const registry = new UiActions({ workspace:"p", window:"w" }, () => true, async () => {});
    render(<UiActionContext.Provider value={registry}><Composer /></UiActionContext.Provider>);
    const action = registry.groups().groups.flatMap(group => registry.observe(group,64).actions).find(item => item.label === "Measure this request with and without selected features")!;
    let outcome: unknown;
    await act(async () => { outcome = await registry.act(registry.scope, { id:"codex:fixture", kind:"agent" }, { handle:action.handle, arguments:{ enabled:true }, request_id:crypto.randomUUID() }); });
    expect(outcome).toMatchObject({ status:"result", outcome:{ status:"requires_input" } });
    expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
    expect(request).not.toHaveBeenCalled();
  });

  it("does not offer extra Codex runs through a different backend", () => {
    render(<Composer backend="python" />);
    expect((screen.getByRole("checkbox") as HTMLInputElement).disabled).toBe(true);
    expect(screen.getByText("Choose Codex ACP to collect comparisons.")).toBeTruthy();
  });

  it("keeps unreviewed correctness and unavailable usage visible", async () => {
    vi.mocked(request).mockResolvedValue({ records: [{ id:"abc", document:"workspace", prompt:"Implement quota", status:"completed", created_at:"today", pairs:[{feature:"literate-indexes", arms:[{arm:"enabled", status:"completed", correct:null, total_tokens:null, wall_seconds:1, answer:"Done"}], savings:null}] }], features:[], coverage:[{contract:"example",status:"unmeasured"}], storage:"local", limitations:"Not a proof" });
    render(<Composer />);
    await act(async () => fireEvent.click(screen.getByRole("button", { name:"Measurements and dataset" })));
    await waitFor(() => expect(screen.getByText(/unknown tokens · correctness unreviewed/)).toBeTruthy());
    expect(screen.getByRole("button", { name:"Export dataset" })).toBeTruthy();
    expect((screen.getByRole("button", { name:"Correct" }) as HTMLButtonElement).disabled).toBe(true);
  });
});
