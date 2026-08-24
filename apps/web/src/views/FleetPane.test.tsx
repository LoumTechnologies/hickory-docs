// docs/guarantees/collaboration/a-machine-is-a-keypair.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { api } from "../api/client";
import { FleetPane } from "./FleetPane";
import type { FleetMachine } from "../api/types";

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

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
    render(<FleetPane />);
    await waitFor(() => expect(screen.getByText(/number0/)).toBeTruthy());
    expect(screen.getByText(/encrypted end to end, but transiting/)).toBeTruthy();
  });

  it("tells an empty fleet that pairing is mutual", async () => {
    fleet();
    render(<FleetPane />);
    await waitFor(() =>
      expect(screen.getByText(/each machine accepts the other/)).toBeTruthy(),
    );
  });

  it("says plainly that reaching a session is still the command line", async () => {
    // The boundary, stated rather than discovered by hunting for a button.
    fleet();
    render(<FleetPane />);
    await waitFor(() =>
      expect(screen.getByText(/Reaching a session is still the command line/)).toBeTruthy(),
    );
  });

  it("draws a grant that is OFF as off, rather than omitting it", async () => {
    // "This machine cannot execute" is the fact worth seeing; an absent row
    // would read as an unanswered question.
    fleet([desktop]);
    render(<FleetPane />);
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
    render(<FleetPane />);
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
    render(<FleetPane />);
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
    render(<FleetPane />);
    await waitFor(() => expect(screen.getByRole("button", { name: "execute" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "execute" }));
    await waitFor(() => expect(grant).toHaveBeenCalledWith("desktop", "execute", true));
  });

  it("shows a refusal in full rather than swallowing it", async () => {
    // A phone declining `execute` and a name already taken both land here,
    // and both are worth reading.
    fleet([{ ...desktop, name: "phone", kind: "phone", grants: ["view"] }]);
    vi.spyOn(api, "fleetGrant").mockRejectedValue(
      new Error("phone is a phone, and a phone has no executor to grant."),
    );
    render(<FleetPane />);
    await waitFor(() => expect(screen.getByRole("button", { name: "execute" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "execute" }));
    await waitFor(() => expect(screen.getByText(/no executor to grant/)).toBeTruthy());
  });

  it("removes a machine, and says that is the whole of the revocation", async () => {
    fleet([desktop]);
    const remove = vi.spyOn(api, "fleetRemove").mockResolvedValue({
      removed: true,
      note: "Its key is gone from this machine, and that is the whole of the revocation — no server holds a session you cannot reach.",
    });
    render(<FleetPane />);
    await waitFor(() => expect(screen.getByRole("button", { name: "remove" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "remove" }));
    await waitFor(() => expect(remove).toHaveBeenCalledWith("desktop"));
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
    vi.spyOn(api, "fleetHost").mockReturnValue(new Promise(() => {}));
    render(<FleetPane />);
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
    const pair = vi.spyOn(api, "fleetPair").mockResolvedValue({
      machine: desktop,
      their_fingerprint: "SHA256:them",
      our_fingerprint: "SHA256:us",
      note: "Compare those two fingerprints on both screens.",
    });
    render(<FleetPane />);
    await waitFor(() => expect(screen.getByLabelText(/a pairing phrase to dial/i)).toBeTruthy());
    fireEvent.change(screen.getByLabelText(/a pairing phrase to dial/i), {
      target: { value: "bagel-cherry-mahogany-cutlass-93" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Pair" }));
    await waitFor(() =>
      expect(pair).toHaveBeenCalledWith("bagel-cherry-mahogany-cutlass-93"),
    );
    await waitFor(() => expect(screen.getByText(/Compare those two fingerprints/)).toBeTruthy());
    expect(screen.getByText(/SHA256:them/)).toBeTruthy();
  });

  it("keeps the invitation flow, folded away for machines that cannot reach each other", async () => {
    fleet();
    render(<FleetPane />);
    await waitFor(() =>
      expect(screen.getByText(/cannot reach each other/)).toBeTruthy(),
    );
  });
});
