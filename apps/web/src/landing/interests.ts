// The discovery page's content.
//
// Sections are titled by the JOB or the PAIN, never by who we guess is
// standing there. "Docs that fail the build when they lie" — not "For
// Platform Engineers". An audience-labelled section re-imposes our own guess
// about who cares, and a visitor who does not recognise the label skips a
// section that was actually about their problem. Titled by the job, people
// self-select by what they reach for, and a segment nobody named can show up
// as a dense cluster in the interest vector.
//
// `id` values are the analytics dimension. They are permanent: renaming one
// silently splits a funnel in two. Removing one ENDS that funnel, which is the
// honest thing to do when the claim behind it was never true of the product.
//
// Every claim below must be true of the shipped binary. There is no server, no
// account and nothing to buy (docs/specs/freeform/local-only.md), so no
// section may link to pricing, imply a plan, or describe a capacity we
// operate. Hick fragments must be valid hick — a visitor pastes them.

export type InterestLink = { id: string; label: string; href: string };

export type Interest = {
  id: string;
  /** The job or pain, in the visitor's words. */
  title: string;
  /** One line, visible while collapsed — enough to decide whether to open. */
  teaser: string;
  /** Shown on expand. Plain paragraphs. */
  body: string[];
  /** Optional literal `.hick` fragment. Byte-for-byte, no escaping. */
  sample?: string;
  links?: InterestLink[];
};

export const INTERESTS: Interest[] = [
  {
    id: "agent-output-review",
    title: "You cannot review what your coding agent actually did",
    teaser: "Its session lands in git as a document you read in a diff, not a chat log.",
    body: [
      "Point your agent at a repository with `hick init` and set `HICKORY_SESSION`. Every tool call it makes — what it read, what it changed, what the run printed — is appended to a `.hick` document in your own repository.",
      "That artifact is reviewable the way code is reviewable: it is a file, it is in the diff, and the commands in it re-run. A messy exploratory session can be promoted with `hick promote` into a clean pipeline that reproduces the same result without the dead ends, so the record of how it was found and the artifact you maintain are both kept, separately.",
      "Your own reasoning stays yours — the session records what was done to the document, not what the model was thinking.",
    ],
  },
  {
    id: "agent-edits-land-wrong",
    title: "Your agent's patch landed two functions away from where it meant to",
    teaser: "Edits anchor on content hashes, so a stale edit is refused, never misapplied.",
    body: [
      "`hick doc read` prefixes every line with a 4-hex hash of its content, and those hashes are the anchors an edit takes. An edit written against a line that has changed since the agent read it is refused — it does not get applied one function down and discovered a week later.",
      "A refusal is routing rather than a dead end: it names the document location to use instead. The workflow that falls out of it is read the surface, edit against the hashes you just read, verify.",
      "`hick mcp` serves the same five tools over stdio for any MCP client and keeps the edit session open between calls. Without MCP they are plain subcommands, so any agent that can run a command can drive them.",
    ],
  },
  {
    id: "ci-drift",
    title: "Your README's examples stopped working and nobody noticed",
    teaser: "Every command in the docs runs on every commit. Drift is a red build.",
    body: [
      "A code block in a README is a claim nobody checks. Hickory runs it. Each example is executed, its real output captured, and compared against what the document says it produces.",
      "`hick test` exits non-zero when a command's output drifts from the committed one, so the same failure that catches a broken test catches a lying paragraph. It runs in CI and as the pre-commit hook `hick init` installs.",
      "There is no separate test suite mirroring the docs, and no third-party recorder to keep in sync — the document is the test.",
    ],
    sample: `<hick:container name="tool" image="alpine:3.20" />

<hick:exec container="tool">
mytool --version
<hick:expect match="regex-lines">mytool 1\\.\\d+\\.\\d+
</hick:expect>
</hick:exec>`,
  },
  {
    id: "edit-the-generated-file",
    title: "The generated file is the one you actually want to edit",
    teaser: "Change the output in your own editor; the change lands in the document.",
    body: [
      "This is the sentence the whole tool exists for. A document assembles `analysis.py`; you open `analysis.py` in whatever editor you already use, change it, and the change is resolved backwards through provenance into the fragment it came from — byte-exactly, not by regenerating and hoping.",
      "`hick up` weaves a folder and keeps it woven, watching for exactly that. Every other literate-programming tool makes the generated file read-only in practice, which is why nobody uses them twice.",
      "An edit that lands on text the weaver wrote rather than text you wrote is refused and named, because there is no source behind it to change.",
    ],
  },
  {
    id: "numbers-from-data",
    title: "The number in paragraph three is from a spreadsheet you deleted",
    teaser: "Figures and statistics regenerate from the source data at render time.",
    body: [
      "Write the analysis inline and the prose quotes its result directly. Change the data, re-render, and every derived figure, table, and inline number moves with it — including the ones buried mid-sentence.",
      "Nothing is pasted, so nothing can be stale. A reviewer asking where a number came from gets an answer instead of an archaeology project.",
    ],
  },
  {
    id: "reproduce-someone-else",
    title: "Reproducing someone else's result takes a week",
    teaser: "A .hick file carries its own environment, so a stranger can re-run it.",
    body: [
      "Each step declares the image it runs in. Someone who clones the repository runs one command and gets the same pipeline, in the same environments, in the same order — not a prose appendix describing what was once installed.",
      "When the re-run disagrees with the committed output, it says so and shows both. Disagreement is the useful result; silence is what makes reproduction hard.",
    ],
  },
  {
    id: "terminal-demos",
    title: "Your terminal demo is a GIF nobody can copy a command out of",
    teaser: "Recorded runs replay as real, scrubbable, selectable text.",
    body: [
      "Every execution captures a timed transcript. The desktop app replays it as a terminal session you can scrub, pause, and select text from — commands included.",
      "It re-records itself on every run, so the demo cannot fall behind the tool the way a hand-recorded screencast does.",
    ],
  },
  {
    id: "provenance",
    title: "Nobody can say where this output byte came from",
    teaser: "Every rendered byte traces back to the source that produced it.",
    body: [
      "Outputs carry provenance back to the fragment, command, and inputs that generated them. `hick lineage` prints it for any generated file; the desktop app draws it as ribbons between the document and the output.",
      "For work that is audited, that is the difference between asserting a result was derived from the data and being able to show it.",
    ],
  },
  {
    id: "runs-on-your-machine",
    title: "You cannot send this code to somebody else's computer",
    teaser: "Execution is local by default, in Docker if you want isolation. Nothing leaves.",
    body: [
      "The local runner executes on your host and is enough for authoring and CI. The Docker executor runs the same unchanged document in the images it declares, when the work needs real separation.",
      "There is no hosted runner, no queue, and no capacity to buy — the choice is configuration on your machine, and the document does not know which one it got.",
      "The binary itself sends nothing, ever: no telemetry, no update check, no crash report, no first-run ping. A tool that runs on private repositories earns that by not talking to anyone.",
    ],
  },
];

/**
 * The one light, skippable identity anchor. This is the ONLY place an audience
 * label is allowed on this page: it is what the visitor claims for themselves,
 * which is exactly what makes it worth comparing against the interests they
 * actually opened. "Other" is deliberately absent — a free-text box here
 * collects typos, not segments; an unlisted visitor should skip instead, and
 * the skip rate is itself a signal that the options are wrong.
 */
export const DECLARED_SEGMENTS: { id: string; label: string }[] = [
  { id: "agent-builder", label: "I work with AI coding agents daily" },
  { id: "tool-maintainer", label: "I maintain a CLI or developer tool" },
  { id: "researcher", label: "I write papers, analyses, or reports" },
  { id: "platform-team", label: "I'm on a platform or infrastructure team" },
  { id: "teacher", label: "I teach or write tutorials" },
];
