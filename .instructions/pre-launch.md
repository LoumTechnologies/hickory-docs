# Pre-launch (nobody has downloaded this yet)

This project has **not launched**. There are **no real users** — nobody has
installed a release and started depending on it.

For a downloadable product this changes what "safe to break" means. There is no
infrastructure to tear down, no database to migrate, and no customer data
anywhere: everything the product touches lives on the user's own disk. What is
irreversible instead is **anything already published**.

- **Break whatever you like in `master`.** Rename commands, change the hick
  grammar, restructure the CLI surface, delete a crate. No installed copy is
  waiting on compatibility.
- **A published release is permanent.** Someone may already have it. Never
  delete or move a published tag or release asset; roll forward with a new
  version instead.
- **Document formats deserve more care than code.** A `.hick` document is a
  file in someone's git repository. Even pre-launch, a parser change that makes
  previously-valid documents fail is worth a deliberate decision rather than a
  side effect — that is the one artifact that outlives any version of this
  tool.
- **Do not add compatibility shims, deprecation periods, or aliases for renamed
  commands.** They are pure cost until someone is actually depending on the old
  spelling, and they make the surface permanently larger.

When real users arrive — meaningful download numbers, issues filed by strangers,
anyone reporting that an upgrade broke their workflow — **switch modules**:
disable `pre-launch` and enable `post-launch`. Do not leave both enabled.

There is no `just launch-state` to consult here and nothing for it to check:
with no server, no accounts, and no payments, the only signal is whether
strangers are using the releases.
