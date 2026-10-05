> Retired 2026-10-05. The homepage now contains one short document with a
> real browser debugger, paused on first load, and a desktop download section.
> Interest disclosures, the identity question, and the recorded-run demo are
> no longer on the homepage. The historical rationale below remains a record,
> not a guarantee of the current page. See `landing/homepage-opens-paused.md`.

# The Discovery Landing Page Is Organized By Interest, Never By Audience Label

Given a signed-out visitor at the site root (`/`, hash route `#/`), when the
landing page renders, then they receive the discovery page — not a login form
— and every disclosure section on it is titled by the job or pain it
addresses ("Your README's examples stopped working and nobody noticed"),
never by a guessed audience label ("For Platform Engineers"). A signed-in
visitor at the same route receives their projects instead, so a returning
customer is never shown a pitch for software they already have.

The single exception is the optional identity anchor at the foot of the page,
which asks the visitor what *they* claim to be. That is a declared-identity
probe whose entire value comes from being comparable against the interests
they actually opened; it is styled quietly, answerable in one click, fully
skippable, and gates nothing on the page.

---

Last LLM verification:
- Date: 2026-08-08
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/landing/interests.ts` holds all page copy as data;
  its header comment states the interest-vs-audience rule and `INTERESTS`
  titles are job/pain phrasings. `DECLARED_SEGMENTS` in the same file is the
  documented single exception. `apps/web/src/views/LandingView.tsx` renders
  the hero, the interest list, and the anchor; the anchor collapses to a
  thank-you after one answer and no CTA is conditioned on it.
  `apps/web/src/router.ts` maps path `/` to `{ name: "landing" }`;
  `apps/web/src/App.tsx` renders `LandingView` for that route only when
  signed out and `ProjectsView` when signed in, and lists `landing` in
  `publicRoute` so the auth gate does not intercept it.
- Test coverage: `apps/web/src/views/LandingView.test.tsx`
  (`titles every section by a job or pain, never by an audience label`,
  `asks for identity only once, and does not block anything on the answer`)
  and `apps/web/src/router.test.ts`
  (`routes the bare domain to the landing page, not straight to a login form`).
- Caveat requiring human review: the test asserts no title begins with "For",
  which catches the obvious regression but cannot judge whether a *new*
  title is genuinely a job rather than a re-worded audience label. Copy
  changes to `INTERESTS` still need a human read against this guarantee.
