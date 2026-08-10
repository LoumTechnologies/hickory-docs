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
// silently splits a funnel in two.

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
    id: "ci-drift",
    title: "Your README's examples stopped working and nobody noticed",
    teaser: "Every command in the docs runs on every commit. Drift is a red build.",
    body: [
      "A code block in a README is a claim nobody checks. Hickory runs it. Each example is executed, its real output captured, and compared against what the document says it produces.",
      "`hickory test` exits non-zero when a command's output drifts from the committed one, so the same failure that catches a broken test catches a lying paragraph. It runs in CI and as a pre-commit hook.",
      "There is no separate test suite mirroring the docs, and no third-party recorder to keep in sync — the document is the test.",
    ],
    sample: `<hick:exec cmd="mytool --version">
  <hick:expect match="regex-lines">mytool 1\\.\\d+\\.\\d+</hick:expect>
</hick:exec>`,
    links: [{ id: "ci-drift-guide", label: "How verification works", href: "#/pricing" }],
  },
  {
    id: "numbers-from-data",
    title: "The number in paragraph three is from a spreadsheet you deleted",
    teaser: "Figures and statistics regenerate from the source data at render time.",
    body: [
      "Write the analysis inline and the prose quotes its result directly. Change the data, re-render, and every derived figure, table, and inline number moves with it — including the ones buried mid-sentence.",
      "Nothing is pasted, so nothing can be stale. A reviewer asking where a number came from gets an answer instead of an archaeology project.",
    ],
    links: [{ id: "numbers-example", label: "See a worked document", href: "#/pricing" }],
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
    id: "agent-output-review",
    title: "You cannot review what an AI agent actually did",
    teaser: "Agent sessions are literate-programming files in git, not a chat log.",
    body: [
      "The agent writes its work as a document: prose, the commands it ran, their real output, and the reasoning between them. It lands in the repository as a file you can read in a diff.",
      "A messy exploratory session can be promoted into a clean pipeline that reproduces the same result without the dead ends — so the record of how it was found and the artifact you maintain are both kept, separately.",
    ],
    links: [{ id: "agent-pricing", label: "What the agent costs", href: "#/pricing" }],
  },
  {
    id: "terminal-demos",
    title: "Your terminal demo is a GIF nobody can copy a command out of",
    teaser: "Recorded runs replay as real, scrubbable, selectable text.",
    body: [
      "Every execution captures a timed transcript. The web app replays it as a terminal session you can scrub, pause, and select text from — commands included.",
      "It re-records itself on every run, so the demo cannot fall behind the tool the way a hand-recorded screencast does.",
    ],
  },
  {
    id: "somewhere-not-my-laptop",
    title: "It works on the machine where it was written",
    teaser: "Run the same document locally or on isolated cloud microVMs.",
    body: [
      "The local runner executes on the host and is enough for development and CI. The same unchanged document runs on isolated microVM sandboxes when the work needs real separation, more capacity, or an environment nobody has to install.",
      "Which one is used is deployment configuration, not something the document knows about.",
    ],
    links: [{ id: "exec-pricing", label: "Execution minutes by plan", href: "#/pricing" }],
  },
  {
    id: "provenance",
    title: "Nobody can say where this output byte came from",
    teaser: "Every rendered byte traces back to the source that produced it.",
    body: [
      "Outputs carry provenance back to the fragment, command, and inputs that generated them. Open a generated file and see which part of the document is responsible for which region of it.",
      "For work that is audited, that is the difference between asserting a result was derived from the data and being able to show it.",
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
  { id: "tool-maintainer", label: "I maintain a CLI or developer tool" },
  { id: "researcher", label: "I write papers, analyses, or reports" },
  { id: "platform-team", label: "I'm on a platform or infrastructure team" },
  { id: "agent-builder", label: "I build with AI agents" },
  { id: "teacher", label: "I teach or write tutorials" },
];
