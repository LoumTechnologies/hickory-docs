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

import { useCallback, useEffect, useState } from "react";

import { api } from "../api/client";
import type { FleetResponse } from "../api/types";

/** What each grant means, said where the grant is toggled. */
const GRANT_TIP: Record<string, string> = {
  view: "See documents, outputs, ribbons, transcripts, and a running terminal's bytes.",
  edit: "Write into the room, and thereby into the file. Not harmless: it can write a document you later run, which is the same trust as accepting a pull request.",
  execute:
    "Run a cell or the up-loop, type into a shell, start or steer an agent turn. Off unless given deliberately — what it costs is a property of that machine, not of the grant: a cell under the sandbox or Docker is confined to its own workdir, and a shell is not.",
};

const GRANTS = ["view", "edit", "execute"] as const;

export function FleetPane() {
  const [fleet, setFleet] = useState<FleetResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [invitation, setInvitation] = useState<string | null>(null);
  const [pasted, setPasted] = useState("");
  const [dialled, setDialled] = useState("");
  /** The phrase this machine is currently hosting, while it waits. */
  const [hosting, setHosting] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const reload = useCallback(() => {
    api.fleet().then(
      (answer) => setFleet(answer),
      (e: unknown) => setError(e instanceof Error ? e.message : String(e)),
    );
  }, []);

  useEffect(reload, [reload]);

  const act = useCallback(
    async (what: () => Promise<string | null>) => {
      setBusy(true);
      setError(null);
      try {
        const said = await what();
        if (said) setNote(said);
        reload();
      } catch (e) {
        // A refusal is the answer, not a crash: a phone declining `execute`
        // and a name already taken both arrive here and both are worth
        // reading in full.
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setBusy(false);
      }
    },
    [reload],
  );

  if (error && !fleet) return <p className="error">{error}</p>;
  if (!fleet) return <p className="muted">Reading this machine’s identity…</p>;

  return (
    <div className="fleet-pane">
      <header className="fleet-pane__head">
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
            disabled={busy}
            onClick={() =>
              act(async () => {
                const { phrase, seconds } = await api.fleetPhrase();
                setHosting(phrase);
                // Read it out, then wait here for the other end to answer.
                void api.fleetHost(phrase).then(
                  (answer) => {
                    setHosting(null);
                    setNote(
                      `Paired with “${answer.machine.name}”. Compare the fingerprints: ` +
                        `them ${answer.their_fingerprint}, you ${answer.our_fingerprint}.`,
                    );
                    reload();
                  },
                  (e: unknown) => {
                    setHosting(null);
                    setError(e instanceof Error ? e.message : String(e));
                  },
                );
                return `Read this out and type it on the other machine. ${seconds} seconds.`;
              })
            }
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
            value={dialled}
            onChange={(e) => setDialled(e.target.value)}
            aria-label="A pairing phrase to dial"
          />
          <button
            type="button"
            disabled={busy || !dialled.trim()}
            onClick={() =>
              act(async () => {
                const answer = await api.fleetPair(dialled.trim());
                setDialled("");
                return (
                  `Paired with “${answer.machine.name}”. ${answer.note} ` +
                  `Them ${answer.their_fingerprint}, you ${answer.our_fingerprint}.`
                );
              })
            }
          >
            Pair
          </button>
        </div>
        <details className="fleet-pair__fallback">
          <summary className="muted">
            Machines that cannot reach each other
          </summary>
        <div className="fleet-pair__row">
          <button
            type="button"
            disabled={busy}
            onClick={() =>
              act(async () => {
                const answer = await api.fleetInvite();
                setInvitation(answer.invitation);
                return answer.note;
              })
            }
          >
            Show this machine’s invitation
          </button>
          {invitation && (
            <button
              type="button"
              onClick={() => {
                void navigator.clipboard?.writeText(invitation);
                setNote("Copied. Paste it on the other machine.");
              }}
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
            value={pasted}
            onChange={(e) => setPasted(e.target.value)}
            aria-label="An invitation to accept"
          />
          <button
            type="button"
            disabled={busy || !pasted.trim()}
            onClick={() =>
              act(async () => {
                const answer = await api.fleetAccept(pasted.trim());
                setPasted("");
                return (
                  `Added “${answer.machine.name}”. It can view and edit; ` +
                  "execute is off until you give it — and remember to accept " +
                  "this machine's invitation over there too, or it will be " +
                  "refused when it dials."
                );
              })
            }
          >
            Accept
          </button>
        </div>
        </details>
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
                      onClick={() =>
                        act(async () => {
                          await api.fleetGrant(machine.name, grant, !on);
                          return null;
                        })
                      }
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
                  onClick={() =>
                    act(async () => {
                      const answer = await api.fleetRemove(machine.name);
                      return answer.note;
                    })
                  }
                >
                  remove
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}

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
