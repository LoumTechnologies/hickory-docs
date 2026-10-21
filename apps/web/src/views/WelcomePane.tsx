// The welcome pane: what an empty workspace says instead of nothing.
//
// An editor with no file open is a blank rectangle, and a blank rectangle is
// the least useful thing a program can show someone who has just launched it.
// This is the tab that fills it, and its whole job is to answer "what now?"
// with things that are one click away rather than with a tour.
//
// Two rules keep it from becoming the marketing page:
//
//  - **Every item here does something.** No screenshots, no feature list, no
//    "learn more". A row is a verb or a document you had open.
//  - **It closes like any other tab, and stays closed.** A welcome screen you
//    cannot dismiss is a welcome screen people learn to close angrily. The
//    preference lives beside the other appearance ones, and the checkbox is
//    on the pane itself so the place you turn it off is the place you are
//    annoyed by it.

import { loadShowWelcome, saveShowWelcome } from "../lib/welcomePref";
import { useId, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand } from "../lib/uiActions";

export interface WelcomeAction {
  id: string;
  label: string;
  hint: string;
  run: () => void;
}

export function WelcomePane({
  actions,
  recent,
  onOpenRecent,
  version,
}: {
  actions: readonly WelcomeAction[];
  /** Documents this folder holds, most recently changed first. */
  recent: readonly { id: string; path: string; updated_at?: string }[];
  onOpenRecent: (id: string) => void;
  version?: string;
}) {
  const [show, setShow] = useState(loadShowWelcome);
  const parent = `welcome:${useId()}`;
  const setPreference = (next: boolean) => { setShow(next); saveShowWelcome(next); };
  const visibleRecent = recent.slice(0, 8);
  const revision = useUiRevision(() => [actions, visibleRecent, show, version]);
  useUiReading("welcome page", { version: version ?? null, show_when_folder_opens: show,
    actions: actions.map(({ id, label, hint }) => ({ id, label, hint })), recent: visibleRecent });
  const command = (id: string, label: string, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) => ({ key: `${parent}:${id}`, command: {
    label, parent, subject: `${parent}:${id}`, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute,
  } });
  useUiCommands([
    ...actions.map(action => command(`start:${action.id}`, action.label, async () => { action.run(); return { status: "completed", value: { requested: action.id, operation_completed: false } }; })),
    ...visibleRecent.map(doc => command(`recent:${doc.id}`, `Open recent document ${doc.path}`, async () => { onOpenRecent(doc.id); return { status: "completed", value: { document_open_requested: doc.id } }; })),
    command("preference", "Show this page when a folder opens", async args => { setPreference(args.show as boolean); return { status: "completed", value: { show: args.show } }; }, { show: { type: "boolean" } }),
  ]);


  return (
    <section className="welcome" aria-label="Welcome">
      <div className="welcome__inner">
        <header className="welcome__head">
          <h1 className="welcome__title">Hickory Docs</h1>
          <p className="welcome__sub">Notes that run, and stay honest about it.</p>
        </header>

        <div className="welcome__columns">
          <section className="welcome__col" aria-label="Start">
            <h2 className="welcome__h2">Start</h2>
            <ul className="welcome__list">
              {actions.map((action) => (
                <li key={action.id}>
                  <button
                    type="button"
                    className="welcome__action"
                    onClick={action.run}
                    data-tip={action.hint}
                  >
                    {action.label}
                  </button>
                </li>
              ))}
            </ul>
          </section>

          <section className="welcome__col" aria-label="Documents in this folder">
            <h2 className="welcome__h2">In this folder</h2>
            {recent.length === 0 ? (
              <p className="muted welcome__empty">
                No documents yet — “New document” makes the first one.
              </p>
            ) : (
              <ul className="welcome__list">
                {visibleRecent.map((doc) => (
                  <li key={doc.id}>
                    <button
                      type="button"
                      className="welcome__action mono"
                      onClick={() => onOpenRecent(doc.id)}
                      data-tip={doc.path}
                    >
                      {doc.path}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>

        <footer className="welcome__foot">
          <label className="welcome__toggle">
            <input
              type="checkbox"
              checked={show}
              onChange={(event) => {
                setPreference(event.target.checked);
              }}
            />
            Show this page when a folder opens
          </label>
          {version && <span className="muted welcome__version">v{version}</span>}
        </footer>
      </div>
    </section>
  );
}
