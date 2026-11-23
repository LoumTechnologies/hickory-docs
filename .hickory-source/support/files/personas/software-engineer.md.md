<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Retain the Simple UI walkthrough for a software engineer

For maintainers evaluating first run and guarded code review. The dated log
records the rendered surfaces and the local checkout used in the walkthrough.

<book:entry path="personas/software-engineer.md">
# Software engineer

**Status:** walkthrough hypothesis supplied by the maintainer.

- **Defining interest:** Read an unfamiliar codebase, ask an agent a specific
  question, and review the exact changes before keeping them.
- **Declared anchor:** A working test and a small reviewable change in a Git
  workspace. Familiar tools are an editor, Git, a terminal and subscription
  coding agents.
- **Adoption stance:** Try the shipped sample first. Use source, a diff and
  actual test output to decide whether the workflow belongs in an existing repo.
- **Monetization trigger:** Reuse an existing subscription for the sample and
  own-project work. Willingness to buy a separate plan is unknown.
- **Products:** Hickory Docs Simple.

## 2026-11-22 · Hickory Docs Simple · workbook to reviewed patch

### Goal and feasibility

“I want to run the checks, understand the result and inspect the agent's change
before it reaches Git.” Start from the local development launcher and use a
disposable copy of the sample. Success means a test result, a change shown as
an exact draft, and a saved Git diff after explicit review. The run can locate
confusion and show that a route works with the tested host and configuration.

### Walked path

1. **Learn and start.** “I have the checkout and want one command.” The sample
   README names `just simple-dev browser`. Browser startup reached Files.
2. **First run.** “Which file will give me a first win?” The browser started
   with just the Files tab. Clicking the workbook opened readable text and
   exposed inline examples. A macOS launch with the workbook path showed
   Files, the workbook and its two native editable grids. Opening `orders.py`
   kept the workbook tab.
3. **Inspect.** “Where's the file this block generates?” The original sample
   omitted `generated/coupon.py` in Files. The prepared output existed on disk.
4. **Ask.** “I already have Codex. Can I use it here?” On macOS, View showed
   standard window commands and Panes held Agent and Repository changes.
   The Agent pane exposed ChatGPT subscription as the default. In the browser,
   selecting Agent and submitting a sample prompt returned HTTP 503 before
   inference. It directed the engineer to Settings → Agents.
5. **Transfer to a real workflow.** “Can I compare this proposal to the last
   saved revision?” The original isolated sample had no Git baseline. The
   existing browser witness separately took recorded patches through guarded
   review, draft Apply and Undo.

This path reached the original browser refusal in six direct actions
(launch, open workbook, open orders, open Agent, enter a prompt, Send), with
two discovery decisions (which file and which pane). Completion time and
own-project adoption were unmeasured.

### Confusion report

| Where | Expected | Got | Did instead | Gave up | Severity | Quote |
| --- | --- | --- | --- | --- | --- | --- |
| Browser Agent Send | Host uses the installed worker and logged-in Codex | HTTP 503 and a settings path absent from Simple | Traced engine startup and connected the existing worker | Original attempt stalled | P0 | “I signed in already. Where is Settings → Agents?” |
| Browser Agent after worker setup | The request reaches Codex from an open source file | The turn refused an editor snapshot without live refresh capability | Connected the existing completion capture protocol | Original attempt stalled | P0 | “The request started. Which editor do I need to update?” |
| Files after launch | Workbook with an obvious first action | Empty editor and a folder of files | Opened workbook by name | No | P1 | “Am I supposed to start in README, the workbook, or orders.py?” |
| Files and repository review | Inspect generated output and compare a saved change | Coupon omitted from the tree and no sample Git baseline | Inspected the disk and Git state | Original review stalled | P1 | “Where is the coupon file, and what am I comparing against?” |
| Recorded patch review | Inspect the live answer's diff and apply it as a draft | The workbook had no canonical workspace entry, so review refused safely | Added `project.md` to bind the workbook's outputs and reset the demo | Original attempt stalled | P1 | “I can see the patch. What entry does it want me to choose?” |
| Repository changes | Select the staged side after pressing Stage | Selecting an old row during the request left the panel needing a refresh | Kept the diff controls disabled until Git reported the new state | No | P2 | “It staged. Why is Unstage still disabled?” |
| macOS menu | Find Agent where the sample says to look | System View menu, then separate Panes menu | Opened Panes | No | P2 | “I can see the editor. Where do I ask about this file?” |

### Follow-through

- Open the demo at `workbook.md` on every development launcher. Lead with
  the four test results, then a small agent patch that changes one thing.
- Attach the existing worker from the development checkout and give a setup
  command on refusal. Send a fresh editor snapshot when the same running
  turn requests one, even while the code tab is active.
- Keep the coupon visible and seed a local Git baseline in freshly prepared
  demos. Guide the reader from the draft to the working-tree diff, with the
  macOS menu name alongside View.

The ideal path is launch → run the four tests → ask for a boundary test →
review and apply a draft → save and rerun → see the resulting Git diff.

### Follow-up evidence

The rebuilt browser opened the workbook, exposed the read-only generated
coupon, and ran the four baseline tests in Terminal. The live ChatGPT
subscription turn finished while the code tab was active. Its exact diff
entered recorded-fence review and applied to a draft without changing the
saved test. An explicit Save made the new test visible in Git, and Stage
published exactly that path. A separate browser run exercised saved diff,
stage and unstage. The focused browser witnesses for early source, immutable
agent history, recorded patch Undo and the addressed Git request passed.
The macOS and Android clients built with the updated capture and Git review
paths. The native observation covered workbook startup, grids, opening code
and reaching the subscription picker.

### Feature candidates for later review

1. Show worker readiness and provider sign-in state before the first Send.
2. Expose a direct review action on completed answers with an applicable
   patch, using the same guarded review path as the gutter.
3. Explore Gemini subscription support if an official subscription path can
   use the same review and isolation boundaries.
4. Evaluate a confined Python worker profile for a local Windows host.

### Declared unknowables

Live provider acceptance covers ChatGPT only. This Mac lacks a Claude Code
CLI on PATH. Windows local Python confinement is currently unavailable.
The walkthrough scope begins with a source checkout. Referral, the public
landing page, downloads, installed-app onboarding, and paid upgrade were
outside it. Live observation covers browser and macOS. iOS, Android, KDE and
Windows use the shared launch or host routing but need runtime acceptance on
their own platforms. A local sample cannot establish time to first value in
a real repo, conversion, persona prevalence, cross-device continuity or which
provider entitlements a different account has.
</book:entry>
</book:doc>
