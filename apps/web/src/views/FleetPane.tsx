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


export function FleetPane() {
  const { fleet, error, note, invitation, pasted, dialled, hosting, busy, setFallbackOpen, setPasted, setDialled, human, copy, receipt, hostResult } = useFleetControls(GRANT_TIP);

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
            disabled={busy || !!hosting}
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
            disabled={busy}
            value={dialled}
            onChange={(e) => setDialled(e.target.value)}
            aria-label="A pairing phrase to dial"
          />
          <button
            type="button"
            disabled={busy || !dialled.trim()}
            onClick={() => human("pair")}
          >
            Pair
          </button>
        </div>
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

function FleetReply({ value }: { value: unknown }) {
  const [open, setOpen] = useState(false);
  return <EvidenceDisclosure label="Latest fleet operation reply" value={value} onOpenChange={setOpen}>
    {open && <pre>{JSON.stringify(value, null, 2)}</pre>}
  </EvidenceDisclosure>;
}
