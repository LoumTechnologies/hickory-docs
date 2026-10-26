<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/FleetPane.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6e68b623335b"></a>
## `apps/web/src/views/FleetPane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Pair identity without silently granting execution](../../../../../guarantees/collaboration/a-machine-is-a-keypair/contract.md), [The Peer Channel Admits Only Keys You Hold, And Serves Them Only What Their Grants Allow](../../../../../guarantees/collaboration/the-peer-channel-carries-grants/contract.md).

<a id="f-6e68b623335b-GRANT_TIP"></a>
### `GRANT_TIP`

The machines whose keys this one holds — and the place you enrol them.
A machine is a keypair; a fleet is a mutual list of public keys under the
engineer's own state directory. There is no account, no directory, and no
server to be signed in to — and revoking a machine is deleting its key,
which is complete, because no server holds a session you cannot reach.
**Enrolling and granting happen HERE, standing at the machine**, and
deliberately cannot happen over the peer channel: a peer that could POST to
`/api/fleet` would grant itself `execute` from inside the very channel
those grants exist to bound. Reading the roster is a `view` matter; writing
it is not reachable remotely at all.
docs/specs/freeform/one-engineer-many-machines.md

<storycode:copy id="f-6e68b623335b-GRANT_TIP">
// The machines whose keys this one holds — and the place you enrol them.
//
// A machine is a keypair; a fleet is a mutual list of public keys under the
// engineer's own state directory. There is no account, no directory, and no
// server to be signed in to — and revoking a machine is deleting its key,
// which is complete, because no server holds a session you cannot reach.
//
// **Enrolling and granting happen HERE, standing at the machine**, and
// deliberately cannot happen over the peer channel: a peer that could POST to
// `/api/fleet` would grant itself `execute` from inside the very channel
// those grants exist to bound. Reading the roster is a `view` matter; writing
// it is not reachable remotely at all.
//
// docs/specs/freeform/one-engineer-many-machines.md

import { useState } from "react";
import { EvidenceDisclosure } from "../components/EvidenceDisclosure";
import { useFleetControls, FLEET_GRANTS as GRANTS } from "./useFleetControls";

/** What each grant means, said where the grant is toggled. */
const GRANT_TIP: Record<string, string> = {
  view: "See documents, outputs, ribbons, transcripts, and a running terminal's bytes.",
  edit: "Write into the room, and thereby into the file. Not harmless: it can write a document you later run, which is the same trust as accepting a pull request.",
  execute:
    "Run a cell or the up-loop, type into a shell, start or steer an agent turn. Off unless given deliberately — what it costs is a property of that machine, not of the grant: a cell under the sandbox or Docker is confined to its own workdir, and a shell is not.",
};


</storycode:copy>
<a id="f-6e68b623335b-FleetPane"></a>
### `FleetPane`

<storycode:copy id="f-6e68b623335b-FleetPane">
export function FleetPane() {
  const { fleet, error, note, invitation, pasted, dialled, hosting, busy, setFallbackOpen, setPasted, setDialled, human, copy, receipt, hostResult, pairing } = useFleetControls(GRANT_TIP);

  if (error && !fleet) return <p className="error">{error}</p>;
  if (!fleet) return <p className="muted">Reading this machine’s identity…</p>;

  return (
    <div className="fleet-pane">
      <header className="fleet-pane__head">
        <button type="button" disabled={busy} onClick={() => human("reload")}>Refresh fleet roster</button>
        <span className="mono">{fleet.this_machine.name}</span>
        <span className="muted mono fleet-pane__fp">
          {fleet.this_machine.fingerprint}
        </span>
      </header>
      <p className="lineage-note" role="status">
        {fleet.note}
      </p>
      {/* Who is in the path, said where the machines are listed rather than
          left in a config file. The default routes through infrastructure
          neither party runs, and that is a fact to be told. */}
      <p
        className={
          fleet.reach === "invalid" ? "lineage-warning" : "merged-view__summary muted"
        }
        role="status"
      >
        {fleet.reach_note}
      </p>

      {error && <p className="error">{error}</p>}
      {note && <p className="lineage-note">{note}</p>}

      {/* ---- pairing by phrase: one exchange, both directions ------------ */}
      <section className="fleet-pair">
        <h3 className="fleet-pair__title">Pair another machine</h3>
        <p className="muted fleet-pair__hint">
          A phrase pairs both machines in one go. It works once and expires —
          anyone who learns it inside that window can pair too, which is why it
          is generated rather than chosen.
        </p>
        <div className="fleet-pair__row">
          <button
            type="button"
            disabled={busy || pairing.active}
            onClick={() => human("phrase")}
          >
            Get a pairing phrase
          </button>
          {hosting && (
            <span className="fleet-pair__phrase mono" aria-label="The pairing phrase">
              {hosting}
            </span>
          )}
        </div>
        <div className="fleet-pair__row">
          <input
            className="fleet-pair__input mono"
            placeholder="…or type the phrase the other machine showed"
            disabled={busy || pairing.active}
            value={dialled}
            onChange={(e) => setDialled(e.target.value)}
            aria-label="A pairing phrase to dial"
          />
          <button
            type="button"
            disabled={busy || pairing.active || !dialled.trim()}
            onClick={() => human("pair")}
          >
            Pair
          </button>
        </div>
        <button type="button" disabled={pairing.pending} onClick={() => pairing.human("refresh")}>Refresh pairing requests</button>
        {(pairing.pending || pairing.uncertain) && <button type="button" onClick={() => pairing.human("cancel-start")}>Request cancellation of pending start</button>}
        {pairing.jobs.map(job => <div key={job.id}>
          <p role="status">{job.mode} request {job.id}: {job.status} ({job.phase}){job.cancel_requested ? "; cancellation requested" : ""}</p>
          {job.status === "queued" || job.status === "running"
            ? <button type="button" onClick={() => pairing.human(`cancel:${job.id}`)}>Cancel pairing request</button>
            : <button type="button" onClick={() => pairing.human(`forget:${job.id}`)}>Forget pairing result</button>}
        </div>)}
        <EvidenceDisclosure className="fleet-pair__fallback" label="Machines that cannot reach each other" value={{ invitation, invitation_draft: pasted }} onOpenChange={setFallbackOpen}>
        <div className="fleet-pair__row">
          <button
            type="button"
            disabled={busy}
            onClick={() => human("invite")}
          >
            Show this machine’s invitation
          </button>
          {invitation && (
            <button
              type="button"
              onClick={() => void copy()}
            >
              Copy
            </button>
          )}
        </div>
        {invitation && (
          <textarea
            className="fleet-pair__invitation mono"
            readOnly
            rows={3}
            value={invitation}
            aria-label="This machine's invitation"
          />
        )}
        <div className="fleet-pair__row">
          <input
            className="fleet-pair__input mono"
            placeholder="hick-fleet:… — paste the other machine's invitation"
            disabled={busy}
            value={pasted}
            onChange={(e) => setPasted(e.target.value)}
            aria-label="An invitation to accept"
          />
          <button
            type="button"
            disabled={busy || !pasted.trim()}
            onClick={() => human("accept")}
          >
            Accept
          </button>
        </div>
        </EvidenceDisclosure>
      </section>

      {fleet.machines.length === 0 ? (
        <p className="muted">
          No other machines paired yet. Show this machine’s invitation, paste it
          on the other one, then bring its invitation back here — a fleet is
          mutual, so each machine accepts the other.
        </p>
      ) : (
        <ul className="fleet-list">
          {fleet.machines.map((machine) => (
            <li key={machine.public_key} className="fleet-machine">
              <span className="fleet-machine__name mono">{machine.name}</span>
              <span className="muted">{machine.kind}</span>
              <span className="muted mono fleet-machine__fp">
                {`key ${machine.public_key.slice(0, 12)}…`}
              </span>
              <span className="fleet-machine__grants">
                {GRANTS.map((grant) => {
                  const on = machine.grants.includes(grant);
                  return (
                    <button
                      key={grant}
                      type="button"
                      disabled={busy}
                      aria-pressed={on}
                      className={`fleet-grant${on ? " fleet-grant--on" : ""}`}
                      data-tip={GRANT_TIP[grant]}
                      onClick={() => human(`grant:${machine.public_key}:${grant}`, { on: !on })}
                    >
                      {grant}
                    </button>
                  );
                })}
                <button
                  type="button"
                  disabled={busy}
                  className="fleet-remove"
                  data-tip="Delete this machine's key. That is the whole of the revocation — no server holds a session you cannot reach."
                  onClick={() => human(`remove:${machine.public_key}`)}
                >
                  remove
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}

      {(receipt || hostResult) && <FleetReply value={{ receipt, hosting: hostResult }} />}

      {/* The boundary, said rather than discovered. */}
      <p className="muted fleet-pane__cli">
        Reaching a session is still the command line:{" "}
        <span className="mono">hick fleet serve</span> on the machine with the
        session, then{" "}
        <span className="mono">hick fleet attach &lt;machine&gt; --addr … --listen 127.0.0.1:7400</span>{" "}
        here, and point a window at that port.
      </p>
    </div>
  );
}

</storycode:copy>
<a id="f-6e68b623335b-FleetReply"></a>
### `FleetReply`

<storycode:copy id="f-6e68b623335b-FleetReply">
function FleetReply({ value }: { value: unknown }) {
  const [open, setOpen] = useState(false);
  return <EvidenceDisclosure label="Latest fleet operation reply" value={value} onOpenChange={setOpen}>
    {open && <pre>{JSON.stringify(value, null, 2)}</pre>}
  </EvidenceDisclosure>;
}
</storycode:copy>

### Assemble `apps/web/src/views/FleetPane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/FleetPane.tsx"><storycode:paste select="#f-6e68b623335b-GRANT_TIP"/><storycode:paste select="#f-6e68b623335b-FleetPane"/><storycode:paste select="#f-6e68b623335b-FleetReply"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-ef08e5ce40d1"></a>
## `apps/web/src/views/FleetPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Pair identity without silently granting execution](../../../../../guarantees/collaboration/a-machine-is-a-keypair/contract.md), [One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted](../../../../../guarantees/collaboration/a-phrase-pairs-both-machines/contract.md).

<a id="f-ef08e5ce40d1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ef08e5ce40d1-expression_statement">
// docs/guarantees/collaboration/a-machine-is-a-keypair.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

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

</storycode:copy>
<a id="f-ef08e5ce40d1-job"></a>
### `job`

<storycode:copy id="f-ef08e5ce40d1-job">
function job(status: FleetPairingJob["status"] = "running"): FleetPairingJob {
  return { id: "host-job", request_id: "request", mode: "host", status, phase: "waiting_for_peer", cancel_requested: status === "cancelled", rollback: false };
}
</storycode:copy>
<a id="f-ef08e5ce40d1-renderFleet"></a>
### `renderFleet`

<storycode:copy id="f-ef08e5ce40d1-renderFleet">
function renderFleet() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  return render(<UiActionContext.Provider value={registry}><FleetPane /></UiActionContext.Provider>);
}

</storycode:copy>
<a id="f-ef08e5ce40d1-fleet"></a>
### `fleet`

<storycode:copy id="f-ef08e5ce40d1-fleet">
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

</storycode:copy>
<a id="f-ef08e5ce40d1-desktop"></a>
### `desktop`

<storycode:copy id="f-ef08e5ce40d1-desktop">
const desktop: FleetMachine = {
  name: "desktop",
  public_key: "AAAABBBBCCCCDDDD",
  kind: "desktop",
  grants: ["view", "edit"],
  added: "2026-08-24",
};

</storycode:copy>
<a id="f-ef08e5ce40d1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ef08e5ce40d1-expression_statement-2">
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

</storycode:copy>
<a id="f-ef08e5ce40d1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ef08e5ce40d1-expression_statement-3">
describe("pairing from the pane", () => {
  it("shows this machine's invitation and says a fleet is mutual", async () => {
    fleet();
    vi.spyOn(api, "fleetInvite").mockResolvedValue({
      invitation: "hick-fleet:abc",
      fingerprint: "SHA256:abc",
      note: "Then do the same in the other direction — a fleet is a MUTUAL list of keys.",
    });
    renderFleet();
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
    // A phone declining `execute` and a name already taken both land here,
    // and both are worth reading.
    fleet([{ ...desktop, name: "phone", kind: "phone", grants: ["view"] }]);
    vi.spyOn(api, "fleetGrant").mockRejectedValue(
      new Error("phone is a phone, and a phone has no executor to grant."),
    );
    renderFleet();
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
    renderFleet();
    await waitFor(() => expect(screen.getByRole("button", { name: "remove" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "remove" }));
    await waitFor(() => expect(remove).toHaveBeenCalledWith("desktop", desktop.public_key));
    await waitFor(() =>
      expect(screen.getByText(/whole of the revocation/)).toBeTruthy(),
    );
  });
});

</storycode:copy>
<a id="f-ef08e5ce40d1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ef08e5ce40d1-expression_statement-4">
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

</storycode:copy>
<a id="f-ef08e5ce40d1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-ef08e5ce40d1-expression_statement-5">
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

</storycode:copy>
<a id="f-ef08e5ce40d1-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-ef08e5ce40d1-expression_statement-6">
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
</storycode:copy>

### Assemble `apps/web/src/views/FleetPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/FleetPane.test.tsx"><storycode:paste select="#f-ef08e5ce40d1-expression_statement"/><storycode:paste select="#f-ef08e5ce40d1-job"/><storycode:paste select="#f-ef08e5ce40d1-renderFleet"/><storycode:paste select="#f-ef08e5ce40d1-fleet"/><storycode:paste select="#f-ef08e5ce40d1-desktop"/><storycode:paste select="#f-ef08e5ce40d1-expression_statement-2"/><storycode:paste select="#f-ef08e5ce40d1-expression_statement-3"/><storycode:paste select="#f-ef08e5ce40d1-expression_statement-4"/><storycode:paste select="#f-ef08e5ce40d1-expression_statement-5"/><storycode:paste select="#f-ef08e5ce40d1-expression_statement-6"/></storycode:file>


</storycode:doc>
