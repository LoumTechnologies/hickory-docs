---
skills:
  - implement-dev-environment
---

# API Client Codegen

- **Every REST API client is generated from the server's OpenAPI spec —
  never hand-written or hand-edited.** If a generated client is missing a
  capability, fix the server's route/DTO annotations and regenerate; don't
  patch the generated file.
- **Generating the client must never require any running infrastructure**
  (no live database, no started server, no cloud dependency). The OpenAPI
  spec is produced by a static generation entrypoint the framework supports
  for this purpose, so `just codegen` works offline on a clean checkout.
- **Mark every generated client path as generated in `.gitattributes`**
  (`linguist-generated=true`) so review tools and diffs treat it as
  generated output, not hand-authored code.
- **CI fails a pull request into `main` if a generated client doesn't match
  what regenerating it right now would produce**, and the failure message
  states the exact fix command (`just codegen`, then commit).
- **Never hand-resolve a merge conflict inside a generated client file —
  always regenerate instead.** This applies equally to humans and LLMs:
  spend zero effort reading or reconciling conflict markers in these paths.
  The fix is always the same, short sequence regardless of what the
  conflict looks like: resolve to either side (content doesn't matter, it's
  about to be overwritten), run `just codegen`, commit the result. Document
  this exact sequence somewhere a contributor without a dev environment set
  up yet can find and follow it (e.g. the CI failure message itself, and
  `docs/developers/developer-environment.md`).
