// docs/guarantees/collaboration/a-machine-is-a-keypair.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { api } from "../api/client";
import { FleetPane } from "./FleetPane";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import type { FleetPairingJob } from "../api/fleetPairings";
import type { FleetMachine } from "../api/types";

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.spyOn(api, "fleetPairings").mockResolvedValue({ jobs: [] });
  vi.spyOn(api, "cancelFleetPairing").mockResolvedValue({ job: job("cancelled") });
});

function job(status: FleetPairingJob["status"] = "running"): FleetPairingJob {
  return { id: "host-job", request_id: "request", mode: "host", status, phase: "waiting_for_peer", cancel_requested: status === "cancelled", rollback: false };
}
function renderFleet() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  return render(<UiActionContext.Provider value={registry}><FleetPane /></UiActionContext.Provider>);
}

function fleet(machines: FleetMachine[] = []) {
  return vi.spyOn(api, "fleet").mockResolvedValue({
    this_machine: { name: "laptop", fingerprint: "SHA256:abc" },
    machines,
    reach: "default",
    reach_note:
      "Reachable directly where possible, and through number0's relays otherwise. Neither is a server we run, and both are somebody's: this machine's addresses are published to number0's DNS, and traffic transits their relays when no direct path exists — encrypted end to end, but transiting.",
    note: "These are the machines whose keys this one holds. Run `hick fleet serve` here to make this session reachable by them.",
  });
}

const desktop: FleetMachine = {
  name: "desktop",
  public_key: "AAAABBBBCCCCDDDD",
  kind: "desktop",
  grants: ["view", "edit"],
  added: "2026-08-24",
};

describe("what the pane says", () => {
  it("names who is in the path, where the machines are listed", async () => {
    // The default routes through infrastructure neither party runs. That is a
    // fact to be told, not one to be found in a config file.
    fleet();
    renderFleet();
    await waitFor(() => expect(screen.getByText(/number0/)).toBeTruthy());
    expect(screen.getByText(/encrypted end to end, but transiting/)).toBeTruthy();
  });

  it("tells an empty fleet that pairing is mutual", async () => {
    fleet();
    renderFleet();
    await waitFor(() =>
      expect(screen.getByText(/each machine accepts the other/)).toBeTruthy(),
    );
  });

  it("says plainly that reaching a session is still the command line", async () => {
    // The boundary, stated rather than discovered by hunting for a button.
    fleet();
    renderFleet();
    await waitFor(() =>
      expect(screen.getByText(/Reaching a session is still the command line/)).toBeTruthy(),
    );
  });

  it("draws a grant that is OFF as off, rather than omitting it", async () => {
    // "This machine cannot execute" is the fact worth seeing; an absent row
    // would read as an unanswered question.
    fleet([desktop]);
    renderFleet();
    // "desktop" is both the machine's NAME and its KIND, so match the row by
    // the control that is unique to it.
    await waitFor(() => expect(screen.getByRole("button", { name: "execute" })).toBeTruthy());
    expect(screen.getByRole("button", { name: "execute" }).getAttribute("aria-pressed")).toBe(
      "false",
    );
    expect(screen.getByRole("button", { name: "view" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
  });
});

describe("pairing from the pane", () => {
  it("shows this machine's invitation and says a fleet is mutual", async () => {
    fleet();
    vi.spyOn(api, "fleetInvite").mockResolvedValue({
      invitation: "hick-fleet:abc",
      fingerprint: "SHA256:abc",
      note: "Then do the same in the other direction — a fleet is a MUTUAL list of keys.",
    });
    renderFleet();
    const disclosure = await screen.findByText("Machines that cannot reach each other");
    await act(async () => {
      fireEvent.click(disclosure);
      await new Promise(resolve => setTimeout(resolve, 0));
    });
    await waitFor(() => expect(screen.getByRole("button", { name: /invitation/i })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: /invitation/i }));
    await waitFor(() =>
      expect((screen.getByLabelText(/this machine's invitation/i) as HTMLTextAreaElement).value).toBe(
        "hick-fleet:abc",
      ),
    );
    expect(screen.getByText(/MUTUAL/)).toBeTruthy();
  });

  it("accepts a pasted invitation, and says execute is still off", async () => {
    fleet();
    const accept = vi.spyOn(api, "fleetAccept").mockResolvedValue({ machine: desktop });
    renderFleet();
    const disclosure = await screen.findByText("Machines that cannot reach each other");
    await act(async () => {
      fireEvent.click(disclosure);
      await new Promise(resolve => setTimeout(resolve, 0));
    });
    await waitFor(() => expect(screen.getByLabelText(/an invitation to accept/i)).toBeTruthy());
    fireEvent.change(screen.getByLabelText(/an invitation to accept/i), {
      target: { value: "hick-fleet:xyz" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Accept" }));
    await waitFor(() => expect(accept).toHaveBeenCalledWith("hick-fleet:xyz"));
    await waitFor(() => expect(screen.getByText(/execute is off/)).toBeTruthy());
  });

  it("toggles a grant through the API", async () => {
    fleet([desktop]);
    const grant = vi
      .spyOn(api, "fleetGrant")
      .mockResolvedValue({ machine: { ...desktop, grants: ["view", "edit", "execute"] } });
    renderFleet();
    await waitFor(() => expect(screen.getByRole("button", { name: "execute" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "execute" }));
    await waitFor(() => expect(grant).toHaveBeenCalledWith("desktop", "execute", true, desktop.public_key));
  });

  it("shows a refusal in full rather than swallowing it", async () => {
    // A changed fleet target is refused even when the UI has an older roster.
    fleet([{ ...desktop, name: "phone", kind: "phone", grants: ["view"] }]);
    vi.spyOn(api, "fleetGrant").mockRejectedValue(
      new Error("The paired phone key changed; refresh the fleet before granting access."),
    );
    renderFleet();
    await waitFor(() => expect(screen.getByRole("button", { name: "execute" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "execute" }));
    await waitFor(() => expect(screen.getByText(/phone key changed/)).toBeTruthy());
  });

  it("removes a machine, and says that is the whole of the revocation", async () => {
    fleet([desktop]);
    const remove = vi.spyOn(api, "fleetRemove").mockResolvedValue({
      removed: true,
      note: "Its key is gone from this machine, and that is the whole of the revocation — no server holds a session you cannot reach.",
    });
    renderFleet();
    await waitFor(() => expect(screen.getByRole("button", { name: "remove" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "remove" }));
    await waitFor(() => expect(remove).toHaveBeenCalledWith("desktop", desktop.public_key));
    await waitFor(() =>
      expect(screen.getByText(/whole of the revocation/)).toBeTruthy(),
    );
  });
});

// docs/guarantees/collaboration/a-phrase-pairs-both-machines.md
describe("pairing by phrase", () => {
  it("shows a phrase to read out, and says it is one-time", async () => {
    fleet();
    vi.spyOn(api, "fleetPhrase").mockResolvedValue({
      phrase: "bagel-cherry-mahogany-cutlass-93",
      seconds: 120,
      note: "Read this to the other machine.",
    });
    // Hosting blocks for the window; never resolving is the realistic case.
    vi.spyOn(api, "startFleetPairing").mockResolvedValue({ started: true, job: job() });
    vi.spyOn(api, "fleetPairing").mockResolvedValue(job());
    renderFleet();
    await waitFor(() => expect(screen.getByRole("button", { name: /pairing phrase/i })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: /pairing phrase/i }));
    await waitFor(() =>
      expect(screen.getByLabelText(/the pairing phrase/i).textContent).toBe(
        "bagel-cherry-mahogany-cutlass-93",
      ),
    );
    // The security property is stated where the phrase is shown.
    expect(screen.getByText(/works once and expires/)).toBeTruthy();
  });

  it("dials a phrase and tells you to compare fingerprints", async () => {
    // Comparing them is the only check that catches somebody who guessed.
    fleet();
    const pair = vi.spyOn(api, "startFleetPairing").mockResolvedValue({ started: true, job: { ...job("paired"), mode: "dial", result: {
      machine: desktop,
      their_fingerprint: "SHA256:them",
      our_fingerprint: "SHA256:us",
      note: "Compare those two fingerprints on both screens.",
    } } });
    renderFleet();
    await waitFor(() => expect(screen.getByLabelText(/a pairing phrase to dial/i)).toBeTruthy());
    fireEvent.change(screen.getByLabelText(/a pairing phrase to dial/i), {
      target: { value: "bagel-cherry-mahogany-cutlass-93" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Pair" }));
    await waitFor(() =>
      expect(pair).toHaveBeenCalledWith("w", expect.any(String), "dial", "bagel-cherry-mahogany-cutlass-93"),
    );
    await waitFor(() => expect(screen.getByText(/Compare those two fingerprints/)).toBeTruthy());
    expect(screen.getByText(/SHA256:them/)).toBeTruthy();
  });

  it("keeps the invitation flow, folded away for machines that cannot reach each other", async () => {
    fleet();
    renderFleet();
    await waitFor(() =>
      expect(screen.getByText(/cannot reach each other/)).toBeTruthy(),
    );
  });
});

it("shares disclosed fleet invitation and grant controls with exact mocked replies", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const roster = fleet([desktop]);
  vi.spyOn(api, "fleetInvite").mockResolvedValue({ invitation: "hick-fleet:local", fingerprint: "SHA256:us", note: "Mutual invitation" });
  const accept = vi.spyOn(api, "fleetAccept").mockResolvedValue({ machine: desktop });
  const grant = vi.spyOn(api, "fleetGrant").mockResolvedValue({ machine: { ...desktop, grants: ["view", "edit", "execute"] } });
  render(<UiActionContext.Provider value={registry}><FleetPane /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "execute" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  expect(actions().some(action => action.label === "Show local fleet invitation")).toBe(false);
  fireEvent.click(screen.getByText("Machines that cannot reach each other", { selector: "summary" }));
  await waitFor(() => expect(actions().some(action => action.label === "Show local fleet invitation")).toBe(true));
  expect(await run("Show local fleet invitation")).toMatchObject({ status: "result", outcome: { value: { response: { invitation: "hick-fleet:local" } } } });
  await run("Set fleet invitation draft", { text: "hick-fleet:peer" });
  fireEvent.click(screen.getByRole("button", { name: "Accept" }));
  await waitFor(() => expect(accept).toHaveBeenCalledExactlyOnceWith("hick-fleet:peer"));
  await waitFor(() => expect((screen.getByRole("button", { name: "execute" }) as HTMLButtonElement).disabled).toBe(false));
  const old = actions().find(action => action.label === "Set fleet grant desktop execute")!.handle;
  expect(await run("Set fleet grant desktop execute", { on: true })).toMatchObject({ status: "result", outcome: { value: { response: { machine: { grants: ["view", "edit", "execute"] } } } } });
  expect(grant).toHaveBeenCalledExactlyOnceWith("desktop", "execute", true, desktop.public_key);
  roster.mockResolvedValueOnce({ ...(await api.fleet()), machines: [{ ...desktop, public_key: "REPLACED-KEY" }] });
  await run("Refresh local fleet roster");
  expect(await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, { handle: old, arguments: { on: true }, request_id: "replaced" })).not.toMatchObject({ status: "result" });
  await run("Set evidence visibility: Machines that cannot reach each other", { open: false });
  expect(actions().some(action => action.label === "Show local fleet invitation")).toBe(false);
});

it("excludes concurrent fleet mutations and reports clipboard authority without a false copy claim", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  fleet([desktop]);
  let complete!: (value: { removed: boolean; note: string }) => void;
  vi.spyOn(api, "fleetRemove").mockImplementation(() => new Promise(done => { complete = done; }));
  const grant = vi.spyOn(api, "fleetGrant");
  vi.spyOn(api, "fleetInvite").mockResolvedValue({ invitation: "hick-fleet:local", fingerprint: "SHA256:us", note: "Invitation ready" });
  render(<UiActionContext.Provider value={registry}><FleetPane /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "remove" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Remove fleet machine desktop")!.handle, arguments: {}, request_id: "remove",
  }); });
  expect(await registry.act(registry.scope, { id: "python:other", kind: "agent" }, {
    handle: actions().find(action => action.label === "Set fleet grant desktop execute")!.handle, arguments: { on: true }, request_id: "grant",
  })).toMatchObject({ status: "disabled" });
  expect(grant).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText("A pairing phrase to dial"), { target: { value: "late-edit" } });
  expect((screen.getByLabelText("A pairing phrase to dial") as HTMLInputElement).value).toBe("");
  await act(async () => { complete({ removed: true, note: "Removed" }); await pending; });
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: label }); }); return result; };
  await run("Set evidence visibility: Machines that cannot reach each other", { open: true });
  await run("Show local fleet invitation");
  const previous = Object.getOwnPropertyDescriptor(navigator, "clipboard");
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: undefined });
  try {
    expect(await run("Copy local fleet invitation")).toMatchObject({ status: "result", outcome: { status: "requires_input", request: expect.stringContaining("clipboard") } });
    expect(screen.queryByText("Copied. Paste it on the other machine.")).toBeNull();
  } finally {
    if (previous) Object.defineProperty(navigator, "clipboard", previous); else Reflect.deleteProperty(navigator, "clipboard");
  }
});
