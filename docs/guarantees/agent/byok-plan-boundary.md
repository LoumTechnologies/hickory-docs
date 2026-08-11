# A Bring-Your-Own-Key Plan Never Spends The Deployment's Credential

Given an account whose plan entitlement is `agent: byo_key` (Open and Pro in
`plans.json`), when it starts an agent session, then the run uses a provider
key **that account stored**, or it does not run at all — even on a deployment
that holds its own key and even when that key would work.

An account with no key of its own is answered `402 Payment Required` with a
message naming its plan, what the plan expects, where to add a key, and the
alternative (a plan with an included allowance). It is never silently served
from the operator's credential.

Symmetrically: an account whose entitlement is `metered_allowance` (Team,
Business) runs on the deployment's key — unless it has stored a key of its
own, in which case its own key is used. Storing a key is an explicit act, and
the reading of it that respects the user is "spend mine, not my allowance."

## Why this is a guarantee and not a preference

`plans.json` has advertised `"agent": "byo_key"` on the free tier since the
catalog was written, and the pricing page renders it as *"AI agent (bring your
own API key)"*. Before this boundary existed there was no place for a user's
key to live, so the entitlement resolved to one of two wrong behaviors: a 503
(the feature was sold and absent), or — once any key was configured on the
server — every free account's agent traffic billed to the operator.

The second failure is the dangerous one, because nothing about it looks broken
from inside the product. Documents run, sessions complete, users are happy;
the only symptom is an invoice. A pricing model that depends on someone
noticing a bill is not a pricing model, so the boundary is enforced in code
and tested from the outside, by watching which key the vendor actually
receives.

## Boundary

This guarantee is about **whose credential is spent**, not about how much. Per
account LLM spend caps against `agent_llm_budget_cents_month` are not
implemented; a `metered_allowance` plan currently bounds agent usage only
through the executor-minutes quota. That gap is real and is recorded here
rather than in a comment, because the entitlement in `plans.json` names a
cents figure that nothing yet enforces.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/server/src/byok.rs` — `resolve_agent_credential` is the only
  function that answers "whose key?"; it reads the entitlement through
  `plans::resolve`, prefers a stored key when one is selected, and for
  `byo_key` with none stored returns `402` rather than falling back to
  `config.agent_llm`. `apps/server/src/routes/agent.rs::start_agent` no longer
  reads `config.agent_llm` at all — it calls this function and passes the
  resolved credential into `run_agent_session`, so a second entry point cannot
  grow different rules.
- Test coverage: `apps/server/tests/integration.rs` —
  `an_open_plan_account_runs_the_agent_on_the_key_it_brought` runs a real
  session on a deployment that **also** holds a key, and asserts on the
  `x-api-key` the far side received: the account's own key is present and the
  deployment's is absent. Deleting the key returns the account to 402 in the
  same test. `health_and_agent_stub` pins the 402 body for a fresh Open
  account. `agent_replays_branch_history_and_writes_its_edits_back` covers the
  allowance side by putting the account on `team`.
- Not covered by tests: the per-month cents budget named in the Boundary
  section above — there is nothing to test yet.
