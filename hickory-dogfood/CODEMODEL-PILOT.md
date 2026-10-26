# Adopted code-model API wrappers

For maintainers deciding whether code-model generation reduces the source they
must understand and change. Evaluated 2026-10-26 against `c87bcaeed070f814deb3ae4ac501028ca2f14851`.

The TypeScript code model now generates 99 simple browser API wrappers. This
approach is adopted to concentrate review complexity: reviewers check forwarding
rules once in the generator, then review each route declaration for its intended
verb, path, parameters and response. The generated adapter text is derived output.

Start with the [route declarations](../.hickory-source/mechanisms/apps/web/src/api/routes.ts.md)
when adding a simple endpoint. Change the
[generator](../.hickory-source/mechanisms/scripts/generate-api.py.md) when changing
forwarding policy. The [developer workflow](../docs/developers/literate-workflow.md)
explains how to refresh the recording and check for drift. Conditional payloads,
defaults and special transport decisions remain explicit in the manual client.

For example, `file(path)` must URL-encode its path, while `doc(id)` interpolates
its identifier literally. `gitPush()` sends an empty JSON object and
`closeWindow()` sends no body. The declarations select these rules, and the
generator implements them in one place. Focused tests protect those distinctions.

The generator queries the existing TypeScript model through the existing Python
client and keeps its host output in a Hickory exec recording. Offline
reconstruction replays it. A separate check reruns the model and rejects stale
output. The recording makes no container execution claim.

The pilot was initially set aside based on a small line-count reduction. That
assessment used the wrong acceptance criterion. The adoption decision prioritizes
concentrating reasoning and making reviews easier; it does not claim a measured
reduction in review time. The footprint measurements remain below as context.

## What was measured

Counts include comments and exclude blank lines. Derived adapter payloads,
Markdown prose, inventory maps and the unchanged model runtime are excluded.
The delta covers all changed code and workflow files, including new tests;
one-line declarations contribute to the apparent reduction.

| Maintained source | Before | Pilot | Change |
| --- | ---: | ---: | ---: |
| API client | 766 | 390 | −376 |
| Existing client tests | 164 | 187 | +23 |
| Typed route declarations | 0 | 214 | +214 |
| Generator and isolated model config | 0 | 107 | +107 |
| Local check plan, selector, CI and task runner | 1,484 | 1,495 | +11 |
| Total affected scope | 2,414 | 2,393 | −21 |

The independent import control reduced the original client from 766 to 695
lines. Its emitted runtime JavaScript was identical after normalizing the
scratch file's import path, and focused TypeScript checking passed. Removing
imports from the pilot saved 43 lines, but that pruning depended on moving the
wrappers; it was not an independent import-only comparison.

## What was verified

- All 99 wrappers matched the previous implementation's request arguments and
  function arity in 204 cases, including omitted optional arguments and encoded
  spaces, slashes and Unicode.
- Three exact Vitest witnesses passed: generated forwarding, concurrent render
  deduplication, and server JSON refusal messages. Focused TypeScript checking
  covered the client, declarations, generated adapters and client tests.
- Nine temporary probes rejected unknown path parameters, unsupported body modes,
  default/rest parameters, GET bodies, unresolved types, stale working output,
  stale recordings and incorrect input digests.
- Source/output checking, ownership audit and fresh Markdown-only reconstruction
  passed for all 2,881 pilot outputs. Reconstruction safety probes rejected byte
  drift, extra output and an outside symlink. Baseline-byte preservation was not
  claimed because this experiment intentionally changed generated source.

At adoption, the 204 equivalence cases, three exact Vitest witnesses, focused
TypeScript check and nine refusal probes passed again. Fresh reconstruction
passed for all 2,882 outputs, including this adoption record.

These are verification results, not new product guarantees. The temporary probes
are saved with the experiment and are not an installed regression suite. No
broad regressions or post-commit/CI status monitoring were run. Local Node was
24.14.0; the CI configuration uses Node 22, which was not executed here.

## Apply the review criterion to future candidates

Prefer generation when reviewers can learn a small set of rules once and then
check simple declarations for intent. Keep domain-specific decisions visible at
their owner. Count generator, declarations, tests and drift checks as maintained
source, and exclude derived output from that count. Line savings are secondary
to how much reasoning reviewers must repeat. Missing explanations alone do not
establish a generation opportunity.

The independent namespace-import control was not adopted. The ignored local
folder `target/dogfood/codemodel-api-pilot/` retains the original patch, comparison
data and temporary probes. That archive describes the initial pilot; the adopted
literate source and the developer workflow are the maintained implementation.
