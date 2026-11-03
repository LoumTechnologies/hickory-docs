<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/pre-launch.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This literal entry preserves the committed repository guidance, including the skill refresh in `efbe277f`. Edit this owner when the guidance changes; regeneration must retain the exact payload.


Current payload SHA-256: `589178dfdb1b512a92fc18f36f5716c0b108aa9055754687edf8dc9a93d138f5`.

<book:entry path=".instructions/pre-launch.md">
# Pre-launch (production may be torn down)

This project has **not launched** yet. Production infrastructure may already
exist (see `start-with-production`), but there are **no real active users**
depending on it.

- It is **okay to tear down production** when that helps: rebuild a broken
  stack, fix a botched first promote, cut cost while iterating, or wipe and
  re-apply Terraform from a clean state.
- Prefer managed destroy paths (`terraform destroy` in the production
  workspace, project-owned Teardown/destroy workflows if present) over ad-hoc
  cloud console deletes so state stays accurate.
- Staging may also be torn down freely (e.g. **Teardown Staging**) for cost
  control; bring it back before the next serious test.
- Still avoid careless data loss when the repo has seed data, validation
  waitlists, or credentials you care about — back up or re-seed deliberately.
- When real users or live purchases arrive, **switch modules**: disable
  `pre-launch` and enable `post-launch`. Do not leave both enabled.

Run `just launch-state` (when the repo has it) if you are unsure whether you
are still pre-launch; treat live accounts, real customer data, or live Stripe
activity as a signal to move to `post-launch` immediately.
</book:entry>

</book:doc>
