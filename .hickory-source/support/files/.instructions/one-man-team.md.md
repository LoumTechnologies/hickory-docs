<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/one-man-team.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `0d81ab313ffa41029c6a119699f1af3a55508236b50bfb81a7a8236a14508348`.

<book:entry path=".instructions/one-man-team.md">
# One-man team (push to master directly)

This repository is maintained by a **single person**. Optimize for speed and
low ceremony, not multi-author review.

- **Push directly to `master`** for normal work. Do not require a pull request
  into `master` for every change.
- Feature branches and worktrees are still fine when useful (parallel spikes,
  risky experiments); when done, merge or push into `master` without waiting
  on a second reviewer.
- Keep CI and **Deploy Staging** / **Unstable Release** on updates to
  `master` so the long-lived branch still gets automatic checks and deploys.
- Production remains gated (promote / stable release) even for a solo author —
  direct push applies to **`master`**, not to production.
- When a second person starts committing regularly, **switch modules**: disable
  `one-man-team` and enable `multiple-team-members`. Do not leave both enabled.
</book:entry>

</book:doc>
