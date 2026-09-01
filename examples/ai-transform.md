
# An AI-Written Passage That Stays Honest

This document shows `hick:transform`: a passage WRITTEN by a model but
VERIFIED like everything else. The transform names the fragments it read
(`select=`), the instruction it followed (`instruct=`), and a fingerprint of
exactly those inputs (`from=`). `hick refresh` — the only command that calls
a model — rewrites the passage and stamps the fingerprint; `hick test` then
checks the fingerprint offline, free, and deterministically. The document
never claims the prose reproduces; it claims the prose was written from
exactly these bytes under exactly this instruction, and that claim is
checkable in CI without a key.

Edit any fact below and `hick test` fails with "stale transform" until
someone runs `hick refresh` — an LLM-written summary can never silently
drift from the facts it summarizes.

## The facts (the transform's only inputs)

## The summary (written by a model, pinned to the facts)

The release pipeline failed due to three separate issues: the macOS packaging script broke on bash 3.2 with an unbound array, the Windows bundle was missing icon.ico, and the release smoke test couldn't run without bubblewrap. As a result, no release was published for two days, hickorydocs.com's apex served HTTP 522 throughout, and the advertised curl installer failed for every visitor. All three faults were fixed on 2026-08-16, the apex domain was re-attached, and the installer now falls back to the unstable channel.


## The extraction (same inputs, different instruction)

One set of facts can feed several transforms. This one extracts rather than
summarizes:

dist-desktop.sh fails on macOS (bash 3.2, unbound array), Windows bundle missing icon.ico, release smoke test lacks bubblewrap


## Why this is different from pasting model output into a doc

A pasted summary is a claim with no provenance: nothing records what it
summarized, and nothing notices when the facts move. A transform records
both. `hick test` on this file answers one question — *was this prose
derived from these exact bytes under this exact instruction?* — and answers
it without spending a token.
