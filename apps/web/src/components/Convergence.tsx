// The page's actual thesis, stated before any command is shown.
//
// The claim is NOT "we have a good edit format". Hash-anchored edits are table
// stakes in 2026 — every serious harness has one, and a page that pitches its
// anchor scheme as the innovation reads as five years late to anybody who
// would install this.
//
// The claim is that three things which are currently separate, and two of
// which are currently thrown away, are one artifact: the session (what the
// agent did), the reasoning (why), and the edit (what changed). A `.md`
// document is the place where they converge, and MCP is how an agent you
// already run produces one without being told to.
//
// So this component is deliberately the first thing under the hero, and the
// tool surface below it is demoted to *how* rather than *why*.

/** What happens to each of the three today, when they are kept apart. */
const TODAY: { thing: string; fate: string }[] = [
  {
    thing: "The session",
    fate: "Lives in a harness's scrollback. It is not in your repository, it does not survive the tab, and nobody can review it or run it again.",
  },
  {
    thing: "The reasoning",
    fate: "Discarded. What survives is a commit message written after the fact, by someone reconstructing a decision they no longer have the context for.",
  },
  {
    thing: "The edit",
    fate: "Lands as a diff with no memory of either. Six months later the code is the only evidence left, and it cannot say why it is shaped this way.",
  },
];

export function Convergence() {
  return (
    <div className="converge">
      <ul className="converge-today">
        {TODAY.map((item) => (
          <li key={item.thing} className="converge-card">
            <h3>{item.thing}</h3>
            <p>{item.fate}</p>
          </li>
        ))}
      </ul>

      <p className="converge-arrow" aria-hidden="true">
        ↓
      </p>

      <div className="converge-into">
        <h3>One document, in your repository</h3>
        <p>
          The prose that explains the decision, the calls the agent made, the output those calls
          actually produced, and the code — all of it woven into one <span className="mono">
            .md
          </span>{" "}
          file that git already versions. Read it as a document. Review it as a diff. Run it again
          and watch it either reproduce or disagree.
        </p>
        <p>
          Not a transcript archive beside the code, and not a comment block pretending to be a
          record. The generated files are <em>projections</em> of that document, editable from
          either end — so keeping the record and shipping the code are the same act rather than two
          competing ones.
        </p>
      </div>
    </div>
  );
}
