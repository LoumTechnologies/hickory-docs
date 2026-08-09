// Demo 3 — two people in one document, and git underneath it.
//
// Both panes are live editors on the SAME document: type in either and the
// other moves, with the other person's caret where they left it. Underneath,
// the git strip is the durable half — a document is a file in a repository,
// and "who changed this" is a commit, not a revision blob in someone's
// database.

import { useEffect, useMemo, useReducer, useState } from "react";
import type { Room } from "./collabRoom";
import { emit } from "../../analytics/events";
import { DemoSplit } from "./DemoSplit";
import { createRoom, type Peer } from "./collabRoom";
import { INITIAL_GIT, gitReducer, pushState } from "./git";
import { COLLAB_DOC_PATH, COLLAB_SOURCE, REMOTE_PULL_EDIT } from "./scripts";

const PEOPLE = [
  { name: "You", color: "#8f6f3f" },
  { name: "Priya", color: "#3f6f8f" },
] as const;

function Side({
  peer,
  onEdited,
  label,
}: {
  peer: Peer;
  onEdited: () => void;
  label: string;
}) {
  const [source, setSource] = useState(() => peer.ytext.toString());
  // A fresh object here would change the editor's `collab` prop identity on
  // every render, and the editor rebuilds itself when that binding changes —
  // so the pane would be torn down and recreated on every keystroke.
  const collab = useMemo(() => ({ ytext: peer.ytext, awareness: peer.awareness }), [peer]);

  // The CRDT is seeded a tick after the room is built, and a remote edit can
  // land while this pane is untouched — mirror the text from the document
  // itself rather than only from this pane's own keystrokes.
  useEffect(() => {
    const observe = () => setSource(peer.ytext.toString());
    peer.ytext.observe(observe);
    observe();
    return () => peer.ytext.unobserve(observe);
  }, [peer]);

  return (
    <div className="demo-side">
      <p className="demo-side-who">
        <span className="demo-dot" style={{ background: peer.color }} aria-hidden="true" />
        {label}
      </p>
      <DemoSplit
        testId={`demo-collab-${peer.id}`}
        compact
        docPath={COLLAB_DOC_PATH}
        source={source}
        onSourceChange={(next) => {
          setSource(next);
          onEdited();
        }}
        collab={collab}
        nodeHeading="Generated"
      />
    </div>
  );
}

export function CollaborationDemo() {
  // The room is built in an effect, not a memo.
  //
  // A `useMemo` + "destroy on cleanup" pair looks equivalent and is not: React
  // StrictMode mounts every component twice in development, and a memo is not
  // re-evaluated on the second mount. The cleanup from the first mount then
  // tears down the relay that the surviving editors are still bound to, and
  // the two clients silently stop seeing each other — a demo about live
  // collaboration that does not collaborate. Creating it here means every
  // mount gets a room, and every unmount destroys the one it made.
  const [room, setRoom] = useState<Room | null>(null);
  useEffect(() => {
    const created = createRoom(COLLAB_SOURCE, PEOPLE[0], PEOPLE[1]);
    setRoom(created);
    return () => created.destroy();
  }, []);

  const [git, dispatch] = useReducer(gitReducer, INITIAL_GIT);
  const [message, setMessage] = useState("Raise the scale plan's limit");
  const [engaged, setEngaged] = useState(false);
  const [note, setNote] = useState<string | null>(null);

  const engage = (step: string) => {
    emit({ name: "demo_engaged", demo_id: "collaboration", step });
  };
  const onEdited = () => {
    dispatch({ type: "edited" });
    if (engaged) return;
    setEngaged(true);
    engage("typed");
  };

  const push = pushState(git);

  const pull = () => {
    if (!room) return;
    const [author] = room.peers;
    const dirtyBefore = git.dirty;
    const text = author.ytext.toString();
    const at = text.indexOf(REMOTE_PULL_EDIT.find);
    if (at < 0) {
      setNote(
        "That line is gone from the document, so the remote commit no longer applies — which is exactly the conflict git would show you.",
      );
      return;
    }
    room.edit(author, at, at + REMOTE_PULL_EDIT.find.length, REMOTE_PULL_EDIT.replace);
    dispatch({
      type: "pull",
      message: REMOTE_PULL_EDIT.message,
      author: REMOTE_PULL_EDIT.author,
      dirtyBefore,
    });
    setNote(
      `Pulled ${REMOTE_PULL_EDIT.author}'s commit. It landed in both panes at once, and the generated table already quotes it.`,
    );
    engage("pull");
  };

  return (
    <section className="demo" aria-label="Live collaboration backed by git">
      <div className="demo-bar demo-bar-git">
        <label className="demo-commit-msg">
          <span className="muted">Commit message</span>
          <input
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            aria-label="Commit message"
          />
        </label>
        <div className="demo-bar-nav">
          <button
            className="btn"
            disabled={!git.dirty}
            onClick={() => {
              dispatch({ type: "commit", message, author: "you" });
              setNote(null);
              engage("commit");
            }}
          >
            {git.dirty ? "Commit" : "Nothing to commit"}
          </button>
          <button
            className="btn"
            disabled={!push.enabled}
            onClick={() => {
              dispatch({ type: "push" });
              setNote(null);
              engage("push");
            }}
          >
            {push.label}
          </button>
          <button className="btn btn-quiet" onClick={pull}>
            Pull
          </button>
        </div>
      </div>

      <ol className="demo-log" aria-label="Commit log">
        {[...git.commits].reverse().map((commit, i) => {
          const index = git.commits.length - 1 - i;
          return (
            <li key={commit.sha} className={index < git.pushed ? "" : "unpushed"}>
              <span className="mono demo-log-sha">{commit.sha}</span>
              <span className="demo-log-msg">{commit.message}</span>
              <span className="muted demo-log-who">{commit.author}</span>
              {index >= git.pushed && <span className="demo-log-flag">not pushed</span>}
            </li>
          );
        })}
      </ol>

      {note && (
        <p className="demo-reconcile" role="status">
          {note}
        </p>
      )}

      <div className="demo-sides">
        {(room?.peers ?? []).map((peer, i) => (
          <Side
            key={peer.id}
            peer={peer}
            label={i === 0 ? "You — this browser" : `${PEOPLE[1].name} — another browser`}
            onEdited={onEdited}
          />
        ))}
      </div>

      <p className="demo-foot muted">
        Two independent CRDT clients, wired to each other in this page. The git strip is
        simulated; in the app a commit goes to your own GitHub repository, and the document is
        just a file in it.
      </p>
    </section>
  );
}
