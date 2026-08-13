# The Home Page Leads With The Claim, Never With The Edit Format

Given a visitor on the home page, when they read it top to bottom, then they
meet the argument before the mechanism and the mechanism before the proof — in
that order, as three sections:

1. **The claim.** An agent's session, the reasoning behind a change, and the
   change itself are three separate artifacts today, and two of them are
   discarded. A `.hick` document is where they converge.
2. **The mechanism.** An MCP server and five tools, so the agent the visitor
   already runs produces that document as a side effect of working.
3. **The proof.** One demo of the document itself, driven in the visitor's own
   browser.

Given any heading on the page (`h1`, `h2`, `h3`), then it does **not** name the
edit format — no "hash", "anchor", "edit format" or "diff format". Content-hash
anchoring is load-bearing and is described in body text, but it may never be
the promise.

This is a guarantee rather than a preference because of what the failure costs.
Applying an edit to the right span is a problem every serious agent harness
solved years ago. A page that presents it as the innovation reads as years late
to precisely the reader who would install this — the one who already knows the
field — and that reader is the whole market. The mechanism is table stakes; the
convergence is the claim. Confusing the two is a marketing failure that looks
like a technical one.

The corollary, stated so it is not lost: the anchors still have to work. A
record whose edits landed somewhere other than where they claim is worth
nothing, which is why the mechanism appears at all rather than being cut.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/views/LandingView.tsx` renders `Convergence` →
  `AgentToolSurface` → `ProgramDemo` under headings "Three things that should
  have been one", "How it reaches your agent", and "What the document actually
  is". `apps/web/src/components/Convergence.tsx` holds the claim as a shape —
  three muted cards (session, reasoning, edit) collapsing into one accented
  card — so a reader who only skims the headings still receives the argument.
  `apps/web/src/components/AgentToolSurface.tsx` describes content hashes only
  in its closing `.agent-surface-aside` paragraph, which is set quieter and
  rule-indented so the typography agrees with the words. In
  `apps/web/src/landing/interests.ts` the `agent-edits-land-wrong` section is
  titled by the durable-record problem, and names hashing only in its third
  paragraph, explicitly as table stakes.
- Test coverage: `apps/web/src/views/LandingView.test.tsx`
  (`makes the claim before the mechanism, and the mechanism before the proof`
  asserts the `h2` order; `never promotes the edit format to a heading` scans
  every `h1`/`h2`/`h3` for the forbidden vocabulary).
- Caveat requiring human review: the heading scan catches the specific words
  that have gone wrong before, not the general failure. A future heading could
  lead with some other solved mechanism — a diffing strategy, a context-window
  trick — and pass every test. Whether a heading states a claim or a feature is
  a judgement no assertion makes for us.
