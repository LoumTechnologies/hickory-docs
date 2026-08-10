# Niche map

Under-served intersections of *interest × constraint × willingness-to-pay*.
Each carries an exploitation thesis and the **dry-up signal** being watched.
Lifecycle: `spot → enter → exploit → decay/dry → rotate`.

All entries **spotted 2026-08-09**. None has been entered — no probe has run,
`experiments.toml` is empty, and the discovery landing page has never seen
traffic. Every WTP figure below is inferred from competitor list prices, not
observed.

---

## N1 — Verified docs for devtools companies whose docs *are* the product

**State: spotted.** Strongest niche in this map on evidence quality.

- **Interest:** a broken quickstart is a lost evaluation. Devtools companies
  already staff docs and already run CI.
- **Constraint:** the entire paid docs category (Mintlify, ReadMe) hosts and
  generates docs and **verifies nothing**; the free OSS tier (doctests, Quarto)
  verifies per-language but not the published document.
- **WTP evidence:** ReadMe proves an org will pay **$250/mo** for docs
  infrastructure with no verification at all, and **$3,000+/mo** at enterprise.
  Confidence: high (primary pricing page). What is *not* evidence: that any of
  them would pay for verification specifically.
- **Incumbents & paygates:** see `paygate-teardowns.md`. Nobody gates execution.
- **Substitutes:** doctests, `mdBook test`, Sphinx doctest, Dredd/Schemathesis
  (API-only), and "do nothing."
- **Exploitation thesis:** sell the **red ❌ on a doc whose example broke** — a
  screenshotable, CI-native artifact — into a category that has just made
  hosting free and therefore has nothing left to differentiate on.
- **Dry-up signal (watch):** Mintlify or ReadMe shipping example-execution in
  CI. Both are funded to do it and Mintlify's credits meter would absorb it
  cleanly. **This is the fastest-firing signal in the map** — assume 12–18
  months, and treat any such launch as immediate rotation pressure.

---

## N2 — Provenance-for-compliance: byte-level lineage from meeting note → ticket → shipped artifact

**State: spotted.** Weakest evidence, largest ceiling, hardest reachability.
This is the niche the operator added to scope, and desk research treats it
better than any of the others in one specific respect: **its demand driver is
statute, not a vendor's claim.**

- **Interest, with dates attached:**
  - **EU Cyber Resilience Act** — in force 2024-12-10; reporting obligations
    bite **2026-09-11**; full application **2027-12-11**. Requires provenance
    tracking of artifacts entering the build and **audit trails backing the
    technical documentation**.
  - **EU AI Act Art. 12** — automatic event logging across the lifetime of
    high-risk AI systems; key high-risk obligations **2026-08-02**.
  - Reported industry framing: "an agent that can't provide an audit trail
    doesn't get deployed in regulated industries, regardless of performance."
- **Constraint:** the incumbents trace *records humans type in*, not
  derivations. ALM/traceability tools (Jama Connect, Polarion, codebeamer, DOORS
  Next) maintain requirement→test matrices as **manually maintained links**.
  GRC tools (Vanta, Drata) attest to *controls and policies*, not to how a byte
  in the product came to exist. Nothing in either category **executes** anything
  or derives lineage mechanically.
- **WTP evidence:** ALM pricing is entirely behind sales — **no public per-user
  price found for Jama, Polarion, or codebeamer.** That opacity is itself the
  signal: this is a quoted-deal, procurement-driven market, which means high
  ACV *and* a sales motion Hickory explicitly does not have (`pricing-strategy.md`:
  "self-serve, bottom-up, no sales team"). Confidence: high on the opacity,
  **none** on any specific figure.
- **Investor heat:** Norm AI raised a **$120M Series C (July 2026)** for
  regulatory-compliance agents whose pitch is a clear audit trail. Code Ocean
  ($16.5M Series B) shipped v4 as "agentic workflows that generate reproducible,
  **compliant** results" — an adjacent player moving onto this exact
  intersection.
- **Exploitation thesis:** Hickory's provenance map is *mechanically derived*
  rather than asserted — the auditable claim is "this byte came from that source,
  and re-running proves it," which no traceability matrix can make. Against a
  September 2026 statutory deadline, that is a differentiated answer to a
  question buyers are now legally required to answer.
- **Honest counter-thesis (inference):** this niche contradicts the product's
  own go-to-market. It is enterprise, procurement-led, sales-cycle-measured-in-
  quarters, and its buyer is not on Hacker News. Entering it means either a
  sales motion or a land-and-expand path through engineers who then drag it into
  a compliance review — and only the second is compatible with the current plan.
- **Dry-up signal (watch):** an ALM incumbent (Siemens/Polarion, PTC/codebeamer)
  or a GRC incumbent (Vanta, Drata) shipping mechanically-derived code lineage;
  or the CRA guidance settling on SBOM-only evidence, which would let the
  existing supply-chain tools satisfy it and close the seam.

---

## N3 — Agent output as an auditable, re-runnable artifact

**State: spotted.** Newest, least-evidenced, most contested.

- **Interest:** agents now write a large share of code, and the reasoning behind
  it evaporates into chat scrollback. Reported (not observed) industry framing:
  "as agents write more code, context and provenance become independent software
  layers."
- **Constraint:** what exists today is chat transcripts (unstructured, not
  re-runnable) and git history (records the diff, not the derivation).
- **WTP evidence — CORRECTED 2026-08-09 (addendum).** The first pass recorded
  "none found." That was wrong within hours: **AWS Kiro sells this at
  $20/$40/$100/$200 per month** (metered on AI credits, $0.04/credit overage,
  Team tiers with SSO). Spec-driven development is a **funded, priced category**,
  not an unpriced hypothesis. The correction cuts both ways — WTP is proven, and
  the seat at the table is taken.
- **Investor heat: heavy and rising** — this is where capital is flowing, which
  cuts both ways: it front-runs demand *and* guarantees crowding. Named
  platforms competing on governance/auditability of agent work: Factory,
  Cursor, and others.
- **Exploitation thesis:** `hick:session` → `promote` → `hick:doc` turns an
  agent session into a clean, re-runnable pipeline in git. The pitch is not
  "observability of the agent" (crowded) but "the agent's output is a document
  that still runs next year" (not crowded).
- **Dry-up signal (watch):** an agent vendor shipping durable, re-runnable
  session artifacts natively. Because the vendors own the session, they can do
  this whenever they choose — **this niche has the shortest defensible window in
  the map.**

---

## N4 — Reproducible analytical documents for data/research teams

**State: spotted, and already contested. Enter last, if at all.**

- **Interest:** real and long-standing — "works on my machine" is the field's
  running joke.
- **Constraint:** it is *not* under-served. Hex ($36–$75/editor/mo), Deepnote
  ($39–$49/editor/mo), Code Ocean (funded, compliance-flavored), marimo
  ($5M seed; reactive, pure-Python, git-versioned, backed by Jeff Dean and
  Clem Delangue), Quarto and Jupyter Book (free), and Jupyter itself (40M+
  monthly downloads) all occupy it.
- **WTP evidence:** strong and public — a settled **$36–$75/editor/month** band.
  Confidence: medium (third-party sources).
- **Assessment:** this is the best-*proven* WTP in the map and the worst
  *whitespace*. marimo in particular has already taken the "git-friendly,
  reproducible, reactive, file-is-the-truth" position with funding and
  celebrity angels. Hickory's differentiator here narrows to provenance and
  expectation-assertions — genuinely distinct, but a feature-level distinction
  in a category with a funded incumbent, which is the weakest kind.
- **Dry-up signal:** already partially fired at spot time (marimo's seed round,
  Nov 2024). Watch for marimo adding assertions/provenance.

---

## N5 — OSS maintainers (distribution, not revenue)

**State: spotted. Not a revenue niche — do not price it as one.**

- WTP is **$0 by design** (`pricing-strategy.md` says so plainly). The value is
  that this cohort decides whether an HN launch lands, and free public verified
  docs are a marketing surface with backlinks.
- **Constraint worth noting:** this cohort's substitutes are excellent and free,
  and their tolerance for a hosted dependency is low. The self-hostable OSS core
  is not generosity here, it is the price of admission.
- **Dry-up signal:** the OSS core being perceived as a funnel rather than a
  tool — the moment a load-bearing feature moves behind the hosted product, this
  niche stops producing distribution.

---

## N6 — Derived (not asserted) traceability for requirements-bound delivery

**State: spotted 2026-08-09 (addendum).** Added after the operator challenged
the "verified docs" framing as underselling the product. Research supports the
challenge, with one important correction and one hard boundary.

- **Interest:** "did we build what we said, and can you prove it" is a question
  with a named, recurring cost. Reported: fixed-price contracts routinely
  dispute whether a deliverable met the spec, and requirements traceability
  matrices are explicitly the documentation used to demonstrate it. DO-178C at
  DAL A requires **full bidirectional traceability across four levels** —
  system requirements → high-level → low-level → source → tests, forward *and*
  backward — described in the literature as "labor-intensive and susceptible to
  human error without appropriate tools."
- **Constraint — the seam, and it is sharp:** *every* incumbent records
  traceability as a **link someone or something asserted**.
  - GSA's official M3 Playbook RTM template is an **`.xlsx` spreadsheet**.
  - Jama / Polarion / codebeamer / DOORS maintain human-typed link matrices.
  - **AWS Kiro** traces requirements → tasks → tests as agent-written markdown.
  - **GitHub Spec Kit** keeps requirement-to-task links in `tasks.md`.
  - ADRs record a decision and never check it again.

  **Nothing in that list re-executes anything to prove the link still holds.**
  A trace that cannot be re-derived rots silently the first time someone forgets
  to update it — which is the exact failure the whole practice exists to prevent.
- **WTP evidence: the strongest in the map, from two directions.**
  - *Published, self-serve, per-developer:* Kiro at **$20–$200/mo**. This is the
    first proven price found anywhere for "requirements traced through to code."
  - *Opaque, quoted, high-ACV:* Jama/Polarion/codebeamer publish nothing, which
    signals procurement-scale deals in regulated delivery.
  - **No public figure was found for what traceability costs a project today**
    as a percentage of budget — searched for, not found. Declared unknowable.
- **Exploitation thesis:** *the trace is derived and re-provable, not asserted.*
  One sentence, and it holds against Kiro, Spec Kit, Jama, and a spreadsheet
  simultaneously. Verification is not a smaller sibling of provenance here — it
  is the mechanism that makes the provenance claim load-bearing, and it is the
  only part no competitor has.
- **The hard boundary on the pitch (inference, stated so it is not oversold):**
  Hickory's provenance is mechanical from **executed inputs** to outputs. The
  link from a **meeting note, a client decision, or a Jira ticket to a
  requirement** is still an *assertion*, exactly like everyone else's. The
  defensible claim is "requirement → code → test → output is derived;
  intent → requirement is asserted, but versioned and diffable in the same
  document." The full "meeting notes all the way through to the final product"
  story is a **product roadmap claim, not a current capability claim**, and the
  first competent technical evaluator will find the join.
- **Design constraint inherited from the graveyard:** reported best practice is
  to "capture evidence as a byproduct of normal work rather than asking the team
  to do extra work." That is the same lesson as literate programming's death by
  parallel maintenance burden (`failure-ledger.md` §1). Any trace that requires
  deliberate upkeep loses to the spreadsheet it replaced.
- **Dry-up signal (watch):** AWS or GitHub adding *execution-backed* verification
  to their spec traceability — i.e. re-running to confirm the requirement is
  still satisfied rather than asserting the link. Both have the distribution and
  the incentive. Also watch for Jama or Polarion shipping mechanically-derived
  code lineage.
- **Relationship to N2:** N6 is the **reachable, self-serve front door** to the
  same value N2 sells through procurement. N6's buyer (`personas/delivery-owner.md`)
  is on Hacker News and in Jira; N2's buyer is not. If land-and-expand into
  compliance is ever going to work, **N6 is the land.**

---

## N7 — Permission-prompt elimination via document-declared capabilities

**State: spotted 2026-08-09 (fourth addendum).** Added when the operator argued
that sandboxing and policy are not compliance features but **UX features** —
"everyone is TIRED of constantly having security prompts." The evidence supports
the pain strongly and the *position* narrowly. Both halves matter.

### The pain is real and quantified (high confidence)

- Claude Code asks **roughly 100 permissions per hour**; prompts arrive every
  2–3 minutes, and the reported behavioural result is that users **rubber-stamp
  approvals without reading them** — the prompt stops being a control at all.
- The escape hatch is named after its own risk: `--dangerously-skip-permissions`
  ("YOLO mode"), with community consensus reported as "containers or don't
  bother."
- **Anthropic has quantified the operator's exact thesis:** `/sandbox`, powered
  by the open-sourced `sandbox-runtime`, **cut permission prompts by 84%** in
  internal testing. Sandboxing → fewer prompts is a vendor-measured effect, not
  a hypothesis.
- Governance is thin in practice: a 2026 survey reports **41–44% of
  organizations lack human-in-the-loop controls** and **55–63% lack purpose
  binding, kill switches, or network isolation** for agents.

**Correction to the first pass:** the recon addendum treated the security stack
as serving the enterprise cohort (N2/N6). That was wrong. Prompt fatigue is felt
by the *individual developer* — the OSS/distribution half — which means the
capability work serves **both** paths, not just the sellable one.

### The counterweight: the generic sandbox is already commoditized

- **Anthropic open-sourced its isolation layer** (`sandbox-runtime`). OpenAI
  Codex ships a default **Landlock + seccomp** sandbox. Docker launched
  experimental Docker Sandboxes for AI isolation. Cloudflare, Vercel, Ramp, and
  Modal all shipped sandbox features by early 2026.
- Dedicated platforms are funded: **E2B ($35M)**, **Daytona ($24M Series A,
  FirstMark)**, plus Northflank, Firecrawl, Fly Machines.
- Reported: "the sandbox has become the most contested piece of real estate in
  the AI agent stack," with the forecast that **major clouds commoditize the
  baseline in 2026–2027**, leaving specialists to compete on compliance depth
  and developer experience.

**So "we have a sandbox" is not a position.** It competes with a free,
open-source one from the model vendor. The dry-up signal for a generic-sandbox
niche fired *before* entry.

### What is genuinely unoccupied (inference)

Every sandbox found wraps **the agent** — process-level, ambient, session-scoped
— and expresses policy as a **command allowlist**. Two consequences:

1. **The command-allowlist abstraction is the wrong one, and there is a CVE
   proving it.** **CVE-2026-22708** (Cursor): attackers poisoned environment
   variables through shell built-ins so that allowlisted commands such as
   `git branch` delivered arbitrary payloads — *the auto-approval of "safe"
   commands is what made the attack quiet*. A capability model over **resources**
   (network, filesystem, secrets) has no equivalent failure mode, because
   nothing is authorized by *name*. `hick-token`'s attenuate-only macaroons are
   the right shape; the incumbents' allowlists are not.
2. **Nobody's permission is a property of the artifact.** Theirs is a property
   of your session, so it is not reviewable, not versioned, and not reproducible.
   Hickory's would be declared in the document (`hick:allow`), enforced by the
   executor (`--network none` by default in `hickory-executor-docker`), and
   re-provable by `check` — with `image=` making two machines that agree on the
   image agree on the result.

**Exploitation thesis:** *the permission is a property of the document, not of
your session.* The decision "may this run reach the network" is made once, by a
human, in a reviewable file — instead of a hundred times an hour by a tired one.
That is the same derived-not-asserted wedge (N6) applied to authority, and it is
the one framing under which the security work is an individual-developer feature
and an enterprise claim simultaneously.

**Dry-up signal (watch):** Anthropic or OpenAI moving from session-scoped
sandboxes to **per-artifact declared capabilities** — i.e. permissions committed
to the repo rather than configured in a client. Also watch for the major clouds'
managed agent sandboxes shipping policy-as-code, which would close the seam from
below.

**Honest risk:** this niche's competitors include the vendor of the model this
product calls. Distribution, defaults, and bundling all favour them.

---

## Cross-cutting: the category-level repricing event

Not a niche, but it constrains all of them. **Mintlify made documentation
hosting free in June 2026** (Starter *and* Pro at $0 base, unlimited seats on
Pro) and moved its meter to AI credits, on $45M at a $500M valuation. The
implication for `docs/specs/freeform/pricing-strategy.md` is direct: **Pro at
$29/mo gated on "10 private projects, 3 editors" charges for two things the
market now gives away.** The meter that survives this event is execution or
verification, not projects and seats.
