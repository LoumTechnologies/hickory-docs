# Persona — Owner of agent output (agent sessions become artifacts in git)

> **Unvalidated hypothesis.** Identity has not been through the brain's gate and
> this cohort has never been observed. Only the recon-owned sections below are
> populated. See `personas/README.md`.
>
> This is the **least-evidenced** of the five cohorts. External research found no
> product charging for this job, which means willingness to pay here is
> **unknown, not low** — the two are easy to confuse and expensive to confuse.

## Alternatives set — what they use today

*Recon 2026-08-09.*

| Alternative | Kind | Note |
|---|---|---|
| The chat transcript / session scrollback | Do-nothing | The overwhelming default. Unstructured, unversioned, not re-runnable, gone when the window closes |
| `git log` + PR description | Generic substitute | Records the *diff* and a human's summary of intent. Not the derivation |
| Agent vendors' own session history (Claude Code, Cursor, Factory) | Direct rival, latent | They **own the session** and can ship durable artifacts whenever they choose. Reported competitive framing: these platforms already compete on "governance, controls, context, model routing, auditability" |
| Pasting the interesting parts into a Notion/markdown doc by hand | Generic substitute | What conscientious people actually do. Expensive, lossy, done once and never updated |
| Committing `.md` session notes alongside the code | Generic substitute | Cheap, and the closest thing to the product's thesis that costs $0 |
| **Do nothing** | Do-nothing | The reasoning evaporates and nobody notices until someone asks "why is this like this?" six months later |

## Watering holes

*Recon 2026-08-09. Medium-low confidence — this cohort is new and its channels
are unstable.*

| Channel | Norms |
|---|---|
| Hacker News | Fastest-moving audience for agent tooling; also the most saturated with it as of 2026 |
| r/ClaudeAI, r/LocalLLaMA, r/ChatGPTCoding | Workflow-sharing culture; a genuinely novel *workflow* post outperforms a product post by a wide margin |
| Latent Space / AI Engineer community (Discord + newsletter) | Practitioner-dense, high signal, receptive to tooling — but treats hype allergically |
| X/Twitter agent-tooling circle | Where this cohort's norms are actually set; demo-video driven |
| GitHub — the repos themselves | The product's own artifact (`.hick` sessions committed to a public repo) is the distribution: a visible, browsable example repo is a better ad than a post |

## Mistaken-identity set

*Recon 2026-08-09. This cohort's mistakes are the most damaging, because the
adjacent category (agent observability) is loud, well-funded, and describes a
different job with the same vocabulary.*

| They'll think it's… | Why they'd reach for it | Why it falls short |
|---|---|---|
| Agent observability / LLM tracing (LangSmith, Braintrust, PostHog LLM analytics) | The word "provenance" and "trace" collide exactly | Those exist to help you debug and evaluate *the agent*. This exists so the agent's **output** is a document that still runs next year. Different artifact, different reader, different lifetime |
| "Just my chat history, saved" | Superficially the same content | A transcript is not re-runnable and has no lineage from an output byte to its origin. `promote` turns a session into a clean pipeline; a saved transcript stays a transcript |
| A prompt/context manager | "Managing what the agent produced" | Prompt tooling manages input. This manages output as a durable, verifiable artifact |
| An AI documentation writer (Mintlify's agent, ReadMe's GitHub AI Writer) | "AI + docs" | Those generate prose *about* code. Here the agent's session **is** the document, and the document executes |

**Reachability warning (inference):** this cohort's alternatives are owned by
platforms that can close the seam unilaterally, since they control the session.
`docs/market/niche-map.md` N3 records this as the shortest defensible window in
the map.
