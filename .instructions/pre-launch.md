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
