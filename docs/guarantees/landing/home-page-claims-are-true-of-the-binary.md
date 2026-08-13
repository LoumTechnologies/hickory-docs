# Every Claim On The Home Page Is True Of The Downloaded Binary

Given a visitor reading hickorydocs.com, when they act on anything the page
tells them — a command, a flag, a file the tool writes, a hick fragment they
paste — then it behaves as the page said, because the page describes only what
the shipped binary does.

This is a stricter promise than "the marketing is honest", and it is worth
stating as a guarantee because the failure mode is silent: a page for a product
that used to have a hosted workspace keeps describing that workspace long after
it is deleted, and nobody notices until a stranger types the command.

Specifically, the home page must not:

- **Name a command or flag the CLI does not have.** Every one it shows comes
  from `crates/hickory-cli/src/main.rs`.
- **Show a hick fragment that is not valid hick.** A visitor pastes it. An
  `<hick:exec>` must carry its commands in its body and name a container the
  fragment declares — there is no `cmd` attribute.
- **Imply a plan, a price, a quota, a seat, an account, a signup, or capacity
  we operate.** There is none of it (`docs/specs/freeform/local-only.md`). No
  link may point at a pricing page, because there is no pricing page.
- **Advertise a capability the product does not have** — collaboration, a
  hosted issue-tracker integration, a cloud runner we operate. The Canopy
  executor is a node the *user* runs and must never be described as ours.
- **Describe execution as happening anywhere but the visitor's machine.** The
  executors are `LocalExecutor` and Docker, chosen by the user's own
  configuration.

Given the downloaded binary, then the page's claim that it never phones home is
literally true: no telemetry, no update check, no licence check, no crash
reporting, no first-run ping. The marketing site's own PostHog is a property of
the web page and shares no key, build, or code path with the product.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified, after fixing four violations introduced while the product
  was hosted
- Evidence: The command surface shown by
  `apps/web/src/components/AgentToolSurface.tsx` (`hick init`, `hick mcp`, the
  five `hick doc` subcommands, `hick promote`, `HICKORY_SESSION`) matches
  `crates/hickory-cli/src/main.rs` and the managed AGENTS.md body in
  `crates/hickory-cli/src/init.rs`, which is what `hick init` actually writes
  (alongside the `.mcp.json` entry and the pre-commit hook).
  `apps/web/src/landing/interests.ts` carries no `href` to a pricing route and
  no plan language.
- Violations found and fixed in this change (all in the pre-existing page):
  1. Three interest sections linked to `#/pricing`, a route that does not
     exist, with labels "What the agent costs" and "Execution minutes by plan".
  2. The `ci-drift` sample used `<hick:exec cmd="…">`. No `cmd` attribute
     exists — commands go in the element body, and the exec must name a
     declared container.
  3. `somewhere-not-my-laptop` advertised "isolated cloud microVMs" as capacity
     the reader could use. Canopy is optional, off the default path, and points
     at a node the user runs.
  4. A whole demo section ("Two people, one document") demonstrated
     collaboration, which `local-only.md` records as a feature this product
     does not have; its document was a plan-limits table for plans that do not
     exist.
- Test coverage: `apps/web/src/landing/demos/demos.test.tsx` asserts the tool
  surface lists only real subcommands and contains none of "sign up",
  "pricing", "per seat", "free trial", "upgrade";
  `apps/web/src/landing/demos/scripts.test.ts` asserts every `<hick:exec>` in
  the demo document names a container the document declares and that the
  transcript agrees with the pinned expectation;
  `apps/web/src/views/StaticSite.test.tsx` asserts the page issues no `fetch`
  and offers installation rather than signup.
- Caveat requiring human review: no test can prove a *newly added* sentence is
  true of the binary — the tests catch the specific words that have gone wrong
  before, not novel claims. Any change to `interests.ts`,
  `AgentToolSurface.tsx`, or `LandingView.tsx` needs the claims re-read against
  the CLI, and the hick fragments in `interests.ts` are not executed by
  anything. Wiring the page's samples into `hick test` would close that gap and
  has not been done.
