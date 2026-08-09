// The documents the landing-page demos drive, and the step machine for the
// knowledge-work walkthrough.
//
// Everything here is DATA and pure functions: no React, no DOM, no network.
// That is deliberate — the demos run entirely in a stranger's browser with no
// account and no server, so the only way to know they still tell the truth is
// to weave these sources in a test (see scripts.test.ts) and assert the
// lineage they claim to show actually exists.
//
// These sources use real `hick:` tags rather than the `h:` prefix that this
// repo's *documentation* uses. The rule exists so prose about hick can quote
// `hick:` examples literally; this is an application feeding a real weaver, so
// the tags have to be the real ones.
//
// PROSE PARAGRAPHS ARE ONE LINE EACH. The editor soft-wraps, so a source
// hard-wrapped at 78 columns wraps twice and comes out visibly ragged — and
// nobody typing into a WYSIWYG editor puts newlines mid-paragraph. Verbatim
// bodies (hick:copy, hick:file) keep their real line breaks: those are the
// artifact's own bytes.

/** A step of the knowledge-work walkthrough. */
export interface DemoStep {
  /** Permanent analytics id — renaming one splits a funnel in two. */
  id: string;
  /**
   * The step, in the visitor's words. Short: these are chips in a row, and a
   * sentence-length one wraps every chip onto its own line, so on a phone the
   * navigation fills the screen before the demo gets a chance to. The detail
   * lives in `hint`, which has a whole line to itself.
   */
  label: string;
  /** What to look at once the step has run. */
  hint: string;
  /** The document as it stands at the END of this step. */
  source: string;
  /** Lines the agent strip shows while this step is current. */
  activity?: string[];
  /** True when this step's whole point is direct manipulation. */
  invites: "hover" | "edit" | null;
  /**
   * Literal text to scroll the document to, for a step that changes nothing.
   * "Follow the lineage" adds no bytes, so there is no diff to reveal — but it
   * is exactly the step where the notes need to be on screen, because a
   * fragment scrolled out of view draws a stub instead of a ribbon.
   */
  focus?: string;
}

/** Char range of `needle` in `source`, or null. */
export function findRange(source: string, needle: string): [number, number] | null {
  const at = source.indexOf(needle);
  return at < 0 ? null : [at, at + needle.length];
}

const NOTES_DOC_HEAD = `# Sprint planning — 12 August

Attendees: Nate, Priya, Sam. Notes taken live, in this document.
`;

const NOTES = `
## What came up

<hick:copy id="note-auth">
Login fails on Safari 17 whenever the session cookie is SameSite=Lax.
Priya can reproduce it; it started after last Thursday's cookie change.
</hick:copy>

<hick:copy id="note-billing">
Annual plans print the monthly price on the receipt.
Two customers wrote in about it this week. Sam owns it.
</hick:copy>

<hick:copy id="note-search">
Search keeps returning deleted documents for about a minute after deletion.
Nobody has complained yet, so it can wait until after the release.
</hick:copy>
`;

const PROMPT = `
## Handing it to Hickory

<hick:session model="claude-opus-5">
<hick:user>
Turn the notes above into Jira issues in the HD project — one issue per note, with the note itself as the description.
</hick:user>
</hick:session>
`;

// The same session, now carrying the tool calls the agent actually made. A
// tool call is an element in the document, not a line in a chat log that
// scrolls away: this is the record you review in a diff.
const SESSION = `
## Handing it to Hickory

<hick:session model="claude-opus-5">
<hick:user>
Turn the notes above into Jira issues in the HD project — one issue per note, with the note itself as the description.
</hick:user>
<hick:tool name="jira.create_issue" project="HD" key="HD-412">
{"summary": "Safari 17 login fails when the session cookie is SameSite=Lax",
 "type": "Bug", "priority": "High", "assignee": "priya"}
</hick:tool>
<hick:tool name="jira.create_issue" project="HD" key="HD-413">
{"summary": "Annual receipts show the monthly price",
 "type": "Bug", "priority": "Medium", "assignee": "sam"}
</hick:tool>
<hick:tool name="jira.create_issue" project="HD" key="HD-414">
{"summary": "Deleted documents stay in search for about a minute",
 "type": "Task", "priority": "Low", "assignee": "nate"}
</hick:tool>
</hick:session>

<hick:file path="jira/HD-412.md" language="markdown">
# HD-412 — Safari 17 login fails when the session cookie is SameSite=Lax

Type: Bug | Priority: High | Assignee: priya

## Description
<hick:paste select="#note-auth"/>
</hick:file>

<hick:file path="jira/HD-413.md" language="markdown">
# HD-413 — Annual receipts show the monthly price

Type: Bug | Priority: Medium | Assignee: sam

## Description
<hick:paste select="#note-billing"/>
</hick:file>

<hick:file path="jira/HD-414.md" language="markdown">
# HD-414 — Deleted documents stay in search for about a minute

Type: Task | Priority: Low | Assignee: nate

## Description
<hick:paste select="#note-search"/>
</hick:file>
`;

/**
 * Everything the human typed: the heading and the three notes. No later step
 * may rewrite a byte of it — the agent appends its session and the files it
 * weaves, and the notes stay exactly as they were written. Asserted in
 * scripts.test.ts, because a demo that quietly rewrote what you typed would
 * be a slideshow wearing an editor's clothes.
 */
export const HAND_WRITTEN = NOTES_DOC_HEAD + NOTES;

/** Path of the walkthrough's document, as it appears in provenance. */
export const KNOWLEDGE_DOC_PATH = "meetings/2026-08-12-sprint-planning.hick";

export const KNOWLEDGE_STEPS: DemoStep[] = [
  {
    id: "create-document",
    label: "Create a document",
    hint: "An ordinary document. Nothing is generated yet, so nothing flows out of it.",
    source: NOTES_DOC_HEAD,
    invites: null,
  },
  {
    id: "write-notes",
    label: "Write the notes",
    hint: "Each note is a named fragment. Naming it is what makes it addressable later — it is still just the text you typed.",
    source: NOTES_DOC_HEAD + NOTES,
    invites: null,
  },
  {
    id: "ask-the-agent",
    label: "Ask the AI for tickets",
    hint: "The request lands in the document as an element, not in a chat window.",
    source: NOTES_DOC_HEAD + NOTES + PROMPT,
    activity: ["reading meetings/2026-08-12-sprint-planning.hick", "3 notes found"],
    invites: null,
  },
  {
    id: "create-tickets",
    label: "Create the tickets",
    hint: "Three tool calls, three tickets. The calls and their arguments stay in the document.",
    source: NOTES_DOC_HEAD + NOTES + SESSION,
    activity: [
      "jira.create_issue HD-412 · Bug · priya",
      "jira.create_issue HD-413 · Bug · sam",
      "jira.create_issue HD-414 · Task · nate",
      "3 issues created in HD",
    ],
    invites: null,
  },
  {
    id: "follow-lineage",
    label: "Follow the lineage",
    hint: "Hover a ticket, or a ribbon: every ticket description traces back to the sentence in the notes that produced it. Click one to jump to its note.",
    source: NOTES_DOC_HEAD + NOTES + SESSION,
    invites: "hover",
    focus: "Login fails on Safari 17",
  },
  {
    id: "edit-in-jira",
    label: "Edit a ticket",
    hint: "Type in the ticket on the right — that is the Jira copy. The agent is prompted by the change and writes it back into the note it came from.",
    source: NOTES_DOC_HEAD + NOTES + SESSION,
    invites: "edit",
    focus: "Login fails on Safari 17",
  },
];

/**
 * The range of `next` that differs from `prev` — common prefix and suffix
 * stripped — or null when they are the same text.
 *
 * This is what the walkthrough scrolls to and flashes. Steps are not pure
 * appends (the agent's tool calls land *inside* the session element it opened
 * a step earlier), so "everything after the old length" would point at the
 * wrong place exactly when the document gets interesting.
 */
export function changedRange(prev: string, next: string): [number, number] | null {
  if (prev === next) return null;
  let start = 0;
  while (start < prev.length && start < next.length && prev[start] === next[start]) start++;
  let end = 0;
  while (
    end < prev.length - start &&
    end < next.length - start &&
    prev[prev.length - 1 - end] === next[next.length - 1 - end]
  ) {
    end++;
  }
  // Snap to whole lines. A character-exact diff starts mid-token — the common
  // prefix of "</hick:session>" and "<hick:tool …>" is the "<" — so the flash
  // would begin one character into the tag it is pointing at, and the scroll
  // target would sit just below the line the visitor is meant to read.
  const from = next.lastIndexOf("\n", start) + 1;
  const nlAfter = next.indexOf("\n", Math.max(next.length - end, from));
  return [from, nlAfter < 0 ? next.length : nlAfter];
}

/**
 * Advance the walkthrough.
 *
 * `source` is carried, not recomputed, once the visitor has started editing:
 * a step that rewrote the document under someone mid-sentence would be a demo
 * that punishes you for touching it.
 */
export function advance(
  state: { step: number; source: string; edited: boolean },
  direction: 1 | -1,
): { step: number; source: string; edited: boolean } {
  const step = Math.min(Math.max(state.step + direction, 0), KNOWLEDGE_STEPS.length - 1);
  if (step === state.step) return state;
  const next = KNOWLEDGE_STEPS[step];
  // Moving between two steps that share a source (lineage → edit) must never
  // discard what the visitor typed.
  const sameSource = next.source === KNOWLEDGE_STEPS[state.step].source;
  return {
    step,
    source: sameSource ? state.source : next.source,
    edited: sameSource ? state.edited : false,
  };
}

// ---------------------------------------------------------------------------
// Demo 2 — a document that is also the program it describes.
// ---------------------------------------------------------------------------

export const PROGRAM_DOC_PATH = "notes/bisect.hick";

export const PROGRAM_SOURCE = `# Finding an insertion point

We want the leftmost index at which \`target\` could be inserted into a sorted list without breaking the ordering. Two indices bound the answer, and every comparison halves the distance between them.

<hick:copy id="invariant">
# lo <= answer <= hi, and every index below lo is known to be too small.
</hick:copy>

The loop keeps that true. When \`lo\` and \`hi\` meet there is exactly one index left, and the invariant says it is the answer.

<hick:copy id="search">
def bisect_left(xs, target):
    lo, hi = 0, len(xs)
    while lo < hi:
        mid = (lo + hi) // 2
        if xs[mid] < target:
            lo = mid + 1
        else:
            hi = mid
    return lo
</hick:copy>

The prose above is the paper. The two files below are the program, and they are woven from the very same fragments — the explanation cannot drift from the code, because there is only one copy of the code.

<hick:file path="search/bisect.py" language="python">
<hick:paste select="#invariant"/>

<hick:paste select="#search"/>


if __name__ == "__main__":
    print(bisect_left([1, 3, 5, 7, 9], 6))
</hick:file>

<hick:file path="search/test_bisect.py" language="python">
from bisect import bisect_left as reference

<hick:paste select="#search"/>


def test_agrees_with_the_standard_library():
    xs = [1, 3, 5, 7, 9]
    for target in range(0, 11):
        assert bisect_left(xs, target) == reference(xs, target)
</hick:file>
`;

/** The captured run of the tangled program, replayed under the demo. */
export const PROGRAM_TRANSCRIPT: { cmd: string; out: string[] }[] = [
  { cmd: "python search/bisect.py", out: ["3"] },
  {
    cmd: "pytest -q search/test_bisect.py",
    out: [".                                        [100%]", "1 passed in 0.03s"],
  },
];

// ---------------------------------------------------------------------------
// Demo 3 — two people in one document, and git underneath it.
// ---------------------------------------------------------------------------

export const COLLAB_DOC_PATH = "docs/limits.hick";

export const COLLAB_SOURCE = `# Plan limits

The numbers live here once. Everything that quotes them is woven from this fragment, so there is no second copy to forget.

<hick:copy id="limits">
free    60 requests / minute
team   600 requests / minute
scale 6000 requests / minute
</hick:copy>

Support hands the table below to customers.

<hick:file path="docs/limits.md" language="markdown">
# Rate limits by plan

<hick:paste select="#limits"/>
</hick:file>
`;

/** The edit "octocat" pushed while you were reading — applied by Pull. */
export const REMOTE_PULL_EDIT = {
  find: "scale 6000 requests / minute",
  replace: "scale 6000 requests / minute\nenterprise  negotiated, see the contract",
  message: "Add the enterprise row",
  author: "octocat",
};
