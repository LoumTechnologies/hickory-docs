<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/audits/2026-10-07-agent-misleading-output.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `7c2411b309bedc02ffdc035c6da225bbc385726d7779391e023739bf22139033`.

<book:entry path="docs/guarantees/audits/2026-10-07-agent-misleading-output.md">
# Can a person discover misleading agent output before relying on it?

Audience: Hickory's product designer and engineers deciding how the built-in
agent, ACP integration, and guarantee-maintainer workflow should expose model
failures. This is an evaluation of current behavior and proposed improvements;
it adds no new product guarantees.

## Assessment

Hickory is directionally correct. It makes model work inspectable through
ordinary documents, real execution observations, reviewable edits, and distinct
connections for input context, byte lineage, and asserted citations. These are
substantial improvements over relying on an agent's account of its own work.

The strongest implemented safeguards address wrong edits, stale inputs, lost
records, and fabricated accounts of execution. The weakest area is a plausible
but false interpretation of genuine evidence. A faithful transcript, fresh
summary, executable calculation, and passing test can all coexist with a wrong
conclusion. Hickory's mechanisms make that conclusion easier to investigate;
they do not yet establish that an ordinary reader will notice it before acting.

The next priority should be reliable evidence and explicit assumptions at the
point of reliance. More ribbons alone will not resolve this. Some current
provenance paths themselves overstate what the record establishes.

## What the implementation gets right

- **Inputs and outputs have a record independent of the answer.** The built-in
  loop executes scripts and records observations. ACP preserves exposed
  activity, permissions, interrupted text, and turn status. Arbitrary ACP shell
  output is not converted into invented file-read receipts or executable cells.
- **Evidence has different meanings.** `session_receipts::citations` marks
  paragraph citations as declared assertions. Completed reads name the snapshot
  hash and returned lines. Editor snapshots include unsaved and untitled text.
  The default provenance layers include lineage, context, and declared links.
- **The record is accessible while reading.** The conversation is a literate
  editor, with source chips and links to open files or editor buffers. Evidence
  can be inspected beside the answer instead of reconstructed from a separate
  transcript. Random window identities prevent an old untitled receipt from
  binding to a new unrelated tab after reload.
- **ACP document edits normally pause before application.** New conversations
  use Review. Hickory-handled buffer edits, output edits, document creation, and
  ACP text-file writes use previews and acceptance. A changed review surface is
  refused. Live-buffer edits use ordinary undoable transactions. Generated
  bytes are routed through reverse editing rather than generic file writes.
- **Source changes can invalidate generated prose.** Transform fingerprints
  cover selected input bytes and instruction, including upstream fragments.
  Passage fingerprints detect subsequent text changes. Tests are offline and
  do not ask another model whether the fingerprint matches.
- **Current output lineage is checked independently.** `serve/sessions.rs`
  weaves without execution, compares output bytes to disk, and supplies
  current-output evidence only on a match. A citation triggers this check but
  is not itself its evidence.

Evidence: `crates/hickory-agent/src/{react_loop,session}.rs`,
`crates/hickory-cli/src/serve/acp/{client,edits,mcp,record,read_receipt}.rs`,
`crates/hick-literate/src/{session_elements,session_receipts}.rs`,
`crates/hickory-cli/src/serve/sessions.rs`,
`apps/web/src/lib/{provenanceLayers,editorBufferTargets,agentEdit}.ts`, and
`docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md`.

## Failure scenarios and whether a person can catch them

| Scenario | Current help | Remaining opportunity for misleading output |
|---|---|---|
| A transcript says “Friday if legal approves”; the summary says “Friday” | Preserved source and inspectable citations | Losing the condition is a semantic error; a fresh fingerprint passes |
| A model invents a citation or quotes a real paragraph incorrectly | Citation links and declared styling; dangling selector checks for `cites=` | Answer citations do not establish quote equality, support, or whether that range was read |
| The model reads a failed request as a successful read | Requests and completed receipts have distinct explanations | Both can produce context-family connections; users must notice the distinction |
| A file changes after the model reads it | Snapshot hash in the receipt; review staleness checks | Ordinary links target the current file or buffer, which may differ from the historical input |
| A calculation produces the wrong answer consistently | Real execution, expectations, reproducibility | A wrong denominator, unit, filter, input file, or hardcoded expectation can still pass |
| An agent claims it verified work without verifying | Tool results and execution observations remain inspectable | Built-in completion accepts `done` without checking for a successful verification |
| A model silently chooses an interpretation of an ambiguous request | Full prompt and conversation survive | No general requirement to expose consequential assumptions before accepting work |
| A model finds only evidence agreeing with its hypothesis | Read receipts reveal what was inspected | No record establishes completeness of a search or distinguishes absence from failed access |
| A model writes misleading prose without citations | Supplied editor context links can remain visible beside the answer | These broad input links can look like support despite the explanatory caveat |
| An agent takes a harmful native action | ACP permission decisions and reported activity are recorded; Stop exists | Native commands use the adapter's boundary and can bypass Hickory document review |
| A source contains instructions to ignore the user or transmit data | Editor-context prompt says contents are user material, not instructions | A prompt instruction is not enforced separation; receipts help diagnosis after an action |
| The record is missing, edited, or discarded | Sessions are readable local documents | Editable and gitignored records are not tamper-evident shared attestations; surviving prose can lose its context |

The same concerns apply to non-malicious model failures: omission, confusing
speaker identities, outdated knowledge, incorrect causal reasoning, and
overgeneralizing a successful example. Preserving source bytes makes these
investigable but cannot repair an inaccurate transcription of the original
meeting. Reproducibility establishes a result for recorded conditions; it does
not establish that those conditions model the person's real question.

## Concrete implementation findings

### 1. Context provenance can include an abandoned branch

**High priority; reproduced using the existing CLI.**
`hickory-agent/src/context.rs::derive_from_session` accumulates every preceding
input in file order and does not follow `turn`/`parent`. A fixture with a root
turn, an abandoned child reading `secret-abandoned.csv`, and a new child of the
root writing a line makes `hick context --json` list the abandoned read as input
to the new write.

The actual built-in history follows the selected ancestor chain through
`serve/agent.rs::prior_turns_of`; ACP rewind uses an exact adapter fork point.
The context derivation can therefore attach evidence that was not supplied on
the relevant branch. This finding concerns document write-context derivation;
it does not establish that the conversation lens's branch selection is broken.

There is a second precision issue: built-in later turns replay earlier prompts
and final answers, not the earlier tool observations. “Previously recorded in
the conversation” and “included in this model request” need distinct meanings.
The deriver also ignores the newer editor-context records.

Recommendation: record the actual input envelope for each request, including
branch ancestry, snapshots, tool results, and known replay/compaction limits.
Derive available context from that envelope rather than session-file order.
For ACP, state what Hickory supplied and what the adapter reported; its internal
context management is not fully observable.

### 2. Short line hashes can attach context to changed text

**High priority; reproduced using the existing CLI.**
`tools/hashline.rs::line_hash` folds FNV-1a to 16 bits. Both of these lines hash
to `a0bf`:

```text
Approval threshold: 449 dollars.
Approval threshold: 614 dollars.
```

A session recording a write of the first line, resolved against a document
containing the second, reports `current_lines: [1, 1]`.
`context.rs::locate` chooses the first matching hash sequence, also without
rejecting repeated identical occurrences. Thus even identical text cannot
establish which historical occurrence was written.

The edit resolver checks endpoints of a run, not a digest of its complete
interior. Endpoint hashes alone cannot establish that the whole reviewed or
previously read region is unchanged. The ACP review preview comparisons provide
an additional defense, so this is not a claim that every ACP edit is misapplied.

Recommendation: retain short hashes as convenient display anchors if useful,
but establish identity with exact bytes or strong range/snapshot digests and
explicit occurrence. Ambiguous attribution should remain unresolved.

### 3. Built-in “verify before done” is unenforced

**High priority; confirmed by implementation inspection.**
`protocol.rs::TOOLS_SYSTEM_PROMPT` asks the model to run `verify` before done.
`react_loop.rs` accepts `Turn::Done` and returns a successful agent outcome
without checking that verification ran, passed, or followed the last edit.
The built-in tool path also applies document edits directly; the new ACP Review
policy is not a general policy over this path.

Recommendation: derive a visible verification state from tool events, linked to
the document version checked. “Finished answering,” “edited,” and “checks passed
for these bytes” should be separate facts. Permit legitimate prose-only work
without forcing execution, but never let the final answer manufacture a passing
check. Where review is expected, apply the same review policy across backends.

### 4. Session logging can fail without surfacing the loss

**High priority; inspected, not fault-injected.**
`session.rs::HickSessionLog::record` deliberately discards write/flush errors.
It also silently returns on a poisoned mutex. ACP's `Record::event` uses that
method and returns `Ok(())` after opening the log, so those event-write failures
are not propagated by this wrapper either. Some ACP context writes separately
use a fallible atomic writer, which is a different path.

Recommendation: make evidence-recording health visible and propagate failures
where the record is a prerequisite for a mutation. If an edit already landed,
preserve it and explicitly state that its record is incomplete. Fault-injection
coverage is needed before claiming this boundary is reliable.

### 5. Historical reads commonly open current contents

**Important usability gap; inspected.** Read links carry a path and line range;
the snapshot hash is in their explanation. Editor-buffer targets activate the
current editor. A reader can click a convincing old receipt and inspect newer
text at the same line numbers. The caveat is present, but resolving the mismatch
still requires the reader to do additional work.

Recommendation: open the exact recorded input first, with a nearby comparison
to current contents and an explicit changed-source state. Preserve exact
returned bytes where currently only a digest/range survives. A hash detects a
mismatch; it cannot reconstruct missing uncommitted input. Commit readings and
immutable document lenses provide existing building blocks.

### 6. Reported evidence and checked evidence share families

**Important interpretation risk; implemented distinction is mainly textual.**
`session_elements.rs::ContextElement::links` draws ACP-reported read/search
locations as context and reported edits/diffs as lineage. Titles correctly say
“Reported by the adapter; no byte hash was supplied.” That is weaker evidence
than a harness-observed read or independently checked byte lineage, but the
family does not itself distinguish the strength.

Recommendation: keep the existing three meanings, and add a visible observation
status within each: requested, adapter-reported, harness-recorded, or
independently checked. This does not require inventing a fifth provenance family.

### 7. Several labels make stronger claims than the mechanism supports

**Important wording issue; inspected.** `provenanceLayers.ts` describes lineage
as something “nobody can forge.” Context specs historically assume recorded tools
are the only read path, which is too broad for native ACP actions. The notes-IDE
spec says a summary can be proven to still describe its source, although
`stale_transforms` compares inputs and instruction, not semantic truth.

`TransformHandler` also labels a changed passage “edited by hand since.” A
fingerprint mismatch establishes changed bytes, not whether a human or another
agent changed them. The guarantee title “A Diagram Names What Proves It” exceeds
an arbitrary assertion-cell link: its own caveat acknowledges a cell that always
prints `0` could be named.

Recommendation: consistently name the actual check: source unchanged, passage
changed since generation, bytes match, recorded input, or tests passed at a
particular revision. Avoid a general truth or authorship conclusion.

## How the guarantee-maintainer skill should work here

The current skill already provides good discipline: specific behavioral
contracts, concrete code/test evidence, explicit verification caveats, and a
rule against changing a guarantee to conceal an implementation bug. It is an
instruction workflow today. Merely writing a guarantee Markdown file does not
make Hickory automatically execute its tests or invalidate its review status.

Its natural role is to make consequential assumptions and intended behavior
inspectable before a person accepts an implementation. For example:

> Given a proposed document edit in Review mode, when the agent submits it,
> the target bytes remain unchanged until acceptance. Native adapter commands
> are outside this guarantee's scope.

A useful Hickory workflow would be:

1. **Record the origin of the requirement.** Link to the user's request or
   existing contract. A model-inferred guarantee starts as a proposal. Code
   inspection reveals current behavior but does not establish intended behavior.
2. **Make assumptions and exclusions local to the guarantee.** Which backend,
   edit route, platform, configuration, input size, and concurrency cases apply?
   Show consequential exclusions beside the claim instead of only in a long
   verification footer.
3. **Attach executable witnesses.** Use document cells for checks where practical,
   or invoke existing repo test recipes through those cells. Derive pass/fail
   from the real execution. Keep expectations reviewable and linked to the
   requirement, rather than treating observed current output as the intended
   answer automatically.
4. **Record the version and scope tested.** Include relevant source/test bytes,
   command, environment, result, and whether code was inspected, fixtures ran,
   a real provider ran, or a person observed the UI. Missing dependencies or
   failed reads are evidence gaps, not support.
5. **Challenge the evidence.** Would the witness fail if the claimed behavior
   broke? Add meaningful negative cases, counterexamples, or mutation checks
   where warranted. In the example, verify bytes remain unchanged before Accept,
   Reject changes nothing, stale acceptance fails, and cancellation releases the
   pending request. Native-shell bypass must remain visibly outside the scope.
6. **Review the contract together with the implementation.** A person should see
   what behavior is promised, why that promise matches their request, what was
   tested, and what was not. Rewriting the contract to match a regression should
   be presented as a behavior change.
7. **Invalidate evidence when its dependencies change.** Re-run relevant checks;
   treat changed review dependencies as needing inspection. A stale fingerprint
   can establish changed inputs, but unchanged inputs do not make an LLM review
   correct. Dependency completeness itself needs inspection.

Use separate statuses such as **contract proposed**, **checks passed for this
revision**, **LLM reviewed**, and **human review recorded**. Formal proof should
refer to actual proof artifacts and their assumptions. An AI `BACKED` verdict
from `hick:check` is another model judgment with receipts, useful for finding
suspect passages but capable of the same error as the original answer.

The major failure is shared assumptions: one agent can interpret a request
incorrectly, write the corresponding implementation, write a matching test, and
mark a matching guarantee verified. All four artifacts can agree. A second
model can share the same assumption too. Preserving the original requirement,
displaying interpretation choices, and challenging the witness matter more
than accumulating approving reviews.

## Priorities for making failures discoverable before reliance

1. Repair evidence correctness: branch/request-aware context, strong range
   identity, ambiguous-write refusal, and visible recording failures.
2. Derive completion and verification facts outside the model. Display the exact
   checked revision and outstanding failures. Extend review consistently where
   the user expects it, and show native-action boundaries beside the controls.
3. Make evidence inspection immediate: historical passage, claim-local quote,
   reported-versus-observed status, and changed-source comparison. Validate quote
   equality, range existence, and recorded access deterministically where possible;
   these checks still do not establish semantic entailment.
4. Make assumptions visible for consequential work: selected inputs, units,
   denominators, omitted data, conditional decisions, and intended side effects.
   Preserve every identified consequential assumption across the entire session,
   including abandoned branches and retrospective corrections, for later debugging.
   Review points should make the relevant entries easy to inspect.
5. Evaluate people using the product. Seed misleading outputs and measure whether
   readers detect them before Accept, copy, export, or another consequential step;
   measure time, false alarms, and whether readers misunderstand a receipt as
   proof. Passing component tests does not answer this question.

Suggested evaluation fixtures: a fabricated quote with a real citation; a dropped
negation; a conditional deadline made unconditional; conflicting sources; a stale
buffer; an abandoned branch; an incorrect denominator with passing expectations;
“tests passed” after a skipped/failed test; a guarantee narrowed to conceal a
regression; native-shell edits during Review; and hostile instructions inside a
source document. Use disposable workspaces and deterministic peers for unsafe
actions, then separately evaluate live model behavior and human comprehension.

## Initial audit verification and limits

- Inspected product specs, agent guarantees, native loop and recorder, context
  derivation, edit anchors, ACP file/review/recording paths, session receipts,
  current-output checks, and frontend provenance defaults and buffer targets.
- `just test-conversation-lineage` passed: 11 Rust vocabulary/receipt tests, two
  session HTTP tests, one uncited ACP HTTP test, TypeScript checking, and 74 web
  tests. Two native context tests also passed. Those context tests cover linear
  records and ordinary relocation; they do not cover the reproduced branch or
  collision cases.
- A separate run of `transform_tests` passed eight tests and failed
  `weaving_emits_the_pinned_passage_and_calls_no_model` reading missing `out.md`.
  The fixture uses legacy `d.md`; this audit did not repair it or establish the
  intended output path for that fixture. The two session HTTP tests passed.
- Reproduced branch contamination and false current-line attribution using
  synthetic session documents in disposable directories and the existing local
  `target/debug/hick` executable. These are targeted reproductions, not a live
  model trial. Source inspection corroborates both algorithms.
- No paid provider calls, new desktop/browser observation, human study, recording
  fault injection, full security audit, or full repository test run was performed.
  No production guarantees are marked verified by this report. Existing spec
  verification notes provide historical evidence, not checks rerun in this audit.
- Code and guarantee behavior remain unchanged. Recommendations concerning skill
  integration, snapshots, assumption review, and human evaluation are proposed.


## Implementation and retrospective audit (2026-10-08)

The user clarified that assumption disclosure must support debugging long after
work, rather than merely an acceptance checkpoint. The implemented unit is the
retained session: all branches, original declarations and later superseding
entries remain inspectable. Structured tools record assumptions, decisions and
guarantee reviews. The app offers search, category filters, user annotations and
full-session export. Work requests/results and review decisions give the reader
places to investigate when an explanation was omitted. Invalid audit payloads
leave a visible gap rather than disappearing silently.

### Decisions, assumptions and challenges

- **Record discovered assumptions, never certify exhaustive discovery.** A model
  cannot report assumptions it does not recognize; opaque adapters expose less
  context than the native loop. Audit instructions, explicit completeness limits
  and action evidence support investigation. A missing declaration must not be
  read as absence of assumptions. An independent human study remains necessary.
- **Keep the historical passage, separately check the current file.** Exact returned
  bytes, ranges and whole-source SHA-256 live in the session. Dependency snapshots
  and verification source manifests are checked against current confined paths.
  A literal quote occurrence establishes neither attribution nor entailment;
  dropped negations, unsupported denominators and conflicting sources remain
  semantic challenges. Abandoned-branch receipts cannot prove current-turn access.
- **Separate execution evidence from prose and adapter reports.** The native loop
  records its supplied message envelope, not unknowable provider internals. ACP
  records visible prompts and callbacks, not its hidden memory. Adapter locations
  and diffs render as declarations. Harness verification status is independent of
  a completion sentence; executable edits cannot finish without a current passing
  check. Script work invalidates an earlier pass. Checked expectations can themselves
  be wrong, so a pass does not certify a document's conclusions.
- **Review native document tools through the existing edit queue.** The app defaults
  to Review and records accept/reject decisions; CLI runs retain their existing
  policy. Review does not mediate arbitrary shell commands. Preview freshness is
  checked again before applying, without claiming an atomic filesystem transaction
  against unrelated concurrent writers.
- **Replace short edit/attribution identities with SHA-256.** The reproduced legacy
  collision is refused; ambiguous identical-line attribution remains unresolved.
  A whole-surface snapshot is additionally available for callers selecting ranges.
  Existing implicit-output fixtures were updated to declare `hick:file` outputs,
  following the current core contract rather than restoring retired implicit weave.
- **Treat recording loss as failure.** Intent recording gates Hickory-handled work,
  failures remain sticky and missing terminal status is interrupted. Result-recording
  failure after effects requires inspecting actual files; this is not rollback.
  Tests cover failed and removed native records and recovered ACP storage failures.
  Session deletion, malicious rewriting and power-loss durability are outside scope.
- **Keep current guarantees independent of private history.** The updated
  guarantee-maintainer skill preserves original requirements, interpretations,
  exclusions, independent witnesses, counterexamples and dependency revisions.
  It distinguishes LLM review, executed checks and human declarations. Durable
  contract references belong in the repository because private sessions are
  ordinarily gitignored. A failure is not authorization to narrow a requirement.

The ACP Unicode receipt witness also exposed a UTF-8 line-lookup panic in the
session view. Line counting now uses bytes safely, with an explicit emoji-offset
regression witness.

The implementation creates a stronger discovery and investigation framework.
It cannot establish that people detect misleading conclusions before acting.
`examples/agent-audit-evaluation` supplies a blinded observer protocol and seeded
cases for that missing evidence; no observer results are claimed.

### Validation

- `just test-agent-audit` passed: 140 Rust cases across evidence, native agent,
  adversarial outputs, stop/budget, edit freshness and HTTP session/review flows;
  TypeScript checking and 43 focused web cases passed.
- `just test-agent-implementation` passed: agent/CLI libraries, upstream-chain
  verification, agent-cell replay, tool end-to-end, session and transform flows.
- The Rust portions of `just test-acp` passed: six ACP units, five MCP/CLI cases,
  14 deterministic ACP cases (two live-provider cases ignored), ten native HTTP
  cases, eight workspace-filesystem cases and the desktop proxy flow. The final
  web run was repeated separately after test stabilization.
- `just test-conversation-lineage` passed: 12 Rust vocabulary/receipt cases,
  session HTTP and uncited ACP coverage, TypeScript and 74 web cases.
- Final `just test-web` passed: TypeScript and all 1,955 cases across 194 files.
  Earlier runs exposed tests observing the editor before folder hydration and
  the lens registry before its registration effect. Those witnesses now wait
  for the relevant state; assertions remain intact. A run made while editing an
  audit component also saw mismatched old component/new test code; the final
  run used a settled tree.
- Workspace clippy with all targets, code-generation drift, source-file length,
  skill metadata validation and whitespace checks on implementation paths passed.
  The pre-existing startup-content whitespace was left untouched.
- No paid provider calls or human efficacy study were run. Retrospective checks
  establish retained records and their presentation, not exhaustive assumption
  discovery or human harm-prevention efficacy.
</book:entry>

</book:doc>
