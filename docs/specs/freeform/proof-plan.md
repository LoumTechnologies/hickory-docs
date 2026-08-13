# Morning proof plan (2026-08-06)

How to verify, in under 30 minutes, that last night's output meets the vision
on all three axes: money, portfolio, itch. Every claim below is backed by a
command you can run or a test that already gates CI.

## Already proven mechanically (re-runnable)

| Claim | Proof |
|---|---|
| The language + abilities work without the wasm runtime | `cargo test --workspace -- --test-threads=1` — 55 suites, 0 failures; no wasmtime in the tree |
| Docs really execute and self-verify | `just run examples/text-tools-tour.hick && just verify examples/` (also a CI step) |
| Drift is a build failure with a precise message | `cargo run -p hickory-cli -- test crates/hickory-cli/tests/fixtures/drifted-tour.hick` → exit 1, line + diff |
| A statistical paper reproduces its own figure | `just run examples/bootstrap-ci.hick` → regenerates `bootstrap-histogram.svg` from seeded data |
| The flagship demo: weaving, SQL, Polars, feature gates | `cargo run -p hickory-cli -- test examples/grand-tour.hick` — assembles + runs `analysis.py` from prose sections, verifies DuckDB tables exactly, fits a Polars regression and regenerates `regression-explorer.html`; add `--features with-r` for the ggplot section (CI does; locally needs `sudo apt install r-base r-cran-ggplot2`) |
| Agent output is literate programming in git | `cargo test -p hickory-cli --test agent_promote_e2e` — scripted LLM → `hick:session` file → parses → promotes to a `hick:doc` → passes check, zero network |
| Canopy protocol correctness | `cargo test -p hickory-executor-canopy` — 9-test mock-gRPC contract suite (pty framing, forks, tar volumes, tokens); live dev-agent smoke reached authz + spawn, blocked only on a real guest image |
| Canopy API stays in one crate | `tests/isolation.rs` scans the whole workspace (guarantee doc: verified) |
| Server: auth→docs→git→run→stream→check | `cargo test -p hickory-server --test integration` — 6 tests incl. WS streaming and the run-commits-baseline regression |
| Stripe webhooks can't double-fulfill | integration test races 6 concurrent deliveries → exactly one fulfillment (verified to fail against a broken impl) |

## Hands-on demo (the stack is still running)

1. Open http://localhost:8080 — log in `smoke@test.dev` / `hunter2hunter2`
   (dev-only account). Project "smoke" → `tour.hick`.
2. Click **Verify** → green "Verification passed" banner. Edit the source
   (Source toggle), change an expected number, save, Verify → red banner with
   the exact mismatch; run detail includes the line number.
3. Click **Run** on a cell → live transcript streams in over the WebSocket;
   the player scrubs through timed output.
4. Pricing page renders `plans.json` through the API (no hard-coded prices).
5. CLI: `hick agent "…"` works once `ANTHROPIC_API_KEY` is set — sessions
   land in `sessions/*.hick`, `hick promote` compacts them.
6. The editor, live: the "smoke" project now has `weave-demo.hick` — open it
   to see the Typora-style Document view (syntax visible, styled like the
   render), then switch to **Output** → `pleasantries.py`: hover to see each
   character's source lineage, click "Edit output", change a word, save —
   the source `hick:copy` block is rewritten through provenance and a re-run
   reproduces your edit byte-for-byte (verified live against this server).
7. The grand tour: `cargo run -p hickory-cli -- run examples/grand-tour.hick`
   (needs `duckdb` and `uv` on PATH) — read `examples/grand-tour.md`, open
   `examples/regression-explorer.html` in a browser and drag the slider. Add
   `--features with-r` for the ggplot chapter; R locally requires
   `sudo apt install r-base r-cran-ggplot2` (CI runs it with the feature on
   every push).
8. Stop everything with `just dev-stop` when done.

## The three axes

**Money.** Pricing strategy (`pricing-strategy.md`) names the buyer (devtools
teams whose docs are product surface), the anchor (one broken quickstart),
and a $0/$29/$149/$449 grid served live from `plans.json` with entitlements
enforced server-side (private projects, editors, metered exec minutes).
Billing chassis is Stripe-sandbox-ready; the stated readiness gate: no live
mode until hosted execution is stable and a stranger's repo verifies clean.
Validation plan: HN launch + fake-door on Team/Business, decision rule
written down (≥5 fake-door Team selections or 1 concierge commitment in 6
weeks → enable live billing).

**Portfolio.** The repo demonstrates: language implementation (recursive-
descent parser with an unusual no-escaping invariant), dataflow DAG
execution, CRDT collaboration (Yrs), a gRPC protocol client against your own
infra layer, capability tokens, an AI agent harness with a novel state model,
billing done right (idempotent webhooks, grandfathering by construction),
and guarantees-as-docs with tests named to them. README is HN-ready and
honest about boundaries (local executor is not sandboxed; hosted beta in
progress).

**Itch.** The loop that has been in your head for years now runs end to end:
prompt → agent session as a literate `.hick` file → promote → clean pipeline
→ `check` gates drift forever → provenance from every output byte to its
source span. That's literate programming + executable notebooks + scaffolding
+ an agent harness in one artifact, with the semantic-web thread (XML
namespaces as capability vocabulary) intact.

## Honest gaps (next moves, in priority order)

1. **Real canopy execution**: build a sandbox image (`just sandbox-image` on a
   NixOS builder / colo-1 after its ledger re-init), declare it in the ledger,
   set `CANOPY_*` env → the live smoke test (`HICKORY_CANOPY_LIVE=1`) closes
   the last gap. The adapter is ready.
2. ~~**Deploy to Railway**~~ — done, on Fly.io instead: `hickory-docs-production`
   serves hickorydocs.com, and every green CI run on `master` deploys to it
   automatically (`.github/workflows/deploy-production.yml`). Still open: a
   portzero tunnel (or canopy's nginx/ACME domain) so the deployed app can
   reach the canopy node.
3. **Agent live run**: set `ANTHROPIC_API_KEY` and try `hick agent` for
   real (only the scripted-LLM path ran tonight — no key in env).
4. Mobile: Tauri v2 scaffolds are configured (`apps/mobile/README-mobile.md`);
   iOS needs your Mac, Android needs an SDK.
5. BYO-key storage endpoint (BYO-key plans currently fall back to the server
   key), partial cell runs (`{cells}` accepted but whole-doc executes),
   PostHog projects + SendGrid list not yet created (portfolio.toml has
   empty domain/DO fields).
