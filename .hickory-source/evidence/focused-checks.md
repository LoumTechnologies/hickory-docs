<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Inspect the checks that actually ran

- [Engineer-first dogfooding readiness](engineer-first-readiness.md).
<book:include file="engineer-first-readiness.md" />

For: reviewers distinguishing executed witnesses from retained requirements and historical verification claims.

Seventeen focused witnesses passed against the integrated generated source on 2026-10-25: eight Rust checks and nine Vitest assertions. Each invocation was required to execute exactly one named passing case. Other cases in the same frontend file were skipped, not reported as covered.

| Witness | Requirement | Source |
|---|---|---|
| `closure_constructs_atomic_endpoints_and_can_be_reorganized` | `literate-projects/markdown-constructs-the-complete-project.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `refuses_unordered_transforms_hidden_inputs_and_processes` | `literate-projects/markdown-constructs-the-complete-project.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `inventory_checks_binary_modes_symlinks_and_portable_paths` | `literate-projects/complete-inventory-is-byte-exact.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `patch_preserves_bom_crlf_unicode_and_untouched_origins` | `literate-projects/replay-is-exact-and-atomic.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `newline_markers_add_delete_rename_and_failure_are_atomic` | `literate-projects/replay-is-exact-and-atomic.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `maps_edit_within_one_span` | `execution/output-lineage-round-trips-byte-for-byte.md` | [lib.rs](../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3) |
| `rejects_synthetic_overlap_with_offending_range` | `execution/output-lineage-round-trips-byte-for-byte.md` | [lib.rs](../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3) |
| `parse_text_content_with_special_chars` | `language/prefix-only-parsing-no-escaping.md` | [tests.rs](../mechanisms/crates/hick-lang/src/lib/tests/tests.rs.md#output-e91f067d75eb) |
| `waits for an unfinished recovery write and never rewrites explicitly discarded work` | `authoring/unsaved-work-survives-closing-the-app.md` | [drafts.lifecycle.test.tsx](../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md#output-06cfd4483cf0) |
| `keeps an inactive view open when its attempted Save fails` | `authoring/unsaved-work-survives-closing-the-app.md` | [useUnsavedLifecycle.test.tsx](../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-18ff98d49ede) |
| `serializes overlapping saves so each diff sees the previous result` | `authoring/an-output-edit-lands-in-its-document.md` | [outputSave.test.ts](../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43) |
| `does not discard generated edits typed while an earlier save is settling` | `authoring/an-output-edit-lands-in-its-document.md` | [outputSave.test.ts](../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43) |
| `installs a missing debugger and resumes without another click` | `debugging/a-missing-debugger-is-a-button.md` | [DebugStrip.test.tsx](../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210) |
| `does not start a session after setup is dismissed` | `debugging/a-missing-debugger-is-a-button.md` | [DebugStrip.test.tsx](../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210) |
| `preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds` | `agent/expanded-reading-projection.md` | [contextProjection.test.ts](../guarantees/agent/expanded-reading-projection/contract.md#output-cd54a8e364bd) |
| `retains fold intent across editor unmounts and refuses stale source coordinates` | `agent/expanded-reading-projection.md` | [contextProjection.test.ts](../guarantees/agent/expanded-reading-projection/contract.md#output-cd54a8e364bd) |
| `disables both buttons while the document's run is in flight` | `editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md` | [workspaceTabs.test.tsx](../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d) |

Exact commands, observed output and cited-source hashes are retained in the outer focused-results.json record. Cited-file hashes are not a complete transitive source/environment closure. Frontend checks used the integrated source with already installed local dependencies; temporary links and cache files were cleaned up.

## Interpret the results within their scope

These checks exercise core reconstruction and byte mapping; failed saves and discard ordering; serialized generated saves; selected-context coordinates and stale folds; debugger prerequisite setup and dismissal; and disabled actions during a run. They do not certify every clause, native window destruction, real adapter downloads, paid-provider behavior or released-package acceptance.

Three separate paragraph-ledger controls rejected a removed interval, a changed hash and an incorrect destination. This checks accounting of original text, not semantic equivalence of a replacement. LLM implementation review, executed checks and human acceptance remain separate.


## Agent pane refinement — 2026-10-31

For maintainers reviewing contextual approval and conversation space, the following
focused checks passed locally with the installed web dependencies and confined
Python runtime. These are execution observations, not acceptance of every agent
contract or a broad regression result.

- `npm --prefix apps/web test -- src/components/AgentChanges.test.tsx src/components/ChatDock.test.tsx src/components/ContextUpdates.test.tsx src/components/AcpControls.test.tsx src/views/WorkspaceChat.test.tsx`: 64 passing tests. These cover empty review/context UI, proposal decisions, agent self-authorization refusal, mandatory private transport, context revision/race handling, provider/model identity, approval reset and hidden context selection.
- `npm --prefix apps/web run typecheck`: passed.
- With `HICKORY_AGENT_TEST_PYTHON` set to `apps/agent/.venv/bin/python`, `cargo test -p hickory-cli --test serve_python_agent python_turn_requires_review_streams_receipts_and_rehydrates_its_branch -- --exact`: passed. The real confined runtime waits for the current decision, retains future approval in its session, applies a later proposal, restores the saved choice and starts a new conversation in Review.
- `cargo test -p hickory-cli --test serve_acp acp_untitled_edits_use_tools_and_wait_for_client_review -- --exact`: passed. Authorizing future edits at a pending review leaves the current proposal pending; Accept/Reject and consumed-decision refusal still work.
- `cargo test -p hickory-cli --lib private_acp_managed_tools_require_review_and_refuse_unscoped_delivery`: one passing safety case.
- `bash scripts/test-python-agent-ui.sh`: one passing Chromium flow using the scripted provider and real confined worker, through runtime configuration, edit review, reload/recovery and cancellation. The empty-pane screenshot was visually inspected. The stale assertion for unrun-check wording was updated to the supervisor's existing receipt text. Later ACP model compatibility and approval-reset wiring were checked in the focused component tests above.
- Source reconstruction, `behavior.py check`, `behavior.py audit` and `git diff --check` passed during this change. Reconstruction establishes source/output agreement; the audit retains the existing explicit explanation gaps.

Paid providers, other native hosts and broad regressions were not exercised. The
changed context gutter's decisions and lifecycle were exercised in component tests;
its changed-reference popup was not driven against a paid-provider browser session.


- [Local agent measurements and retained raw evidence](local-agent-2026-10-31.md).
<book:include file="local-agent-2026-10-31.md" />


- [Qwen3.8 local CPU measurements](local-agent-qwen38-2026-11-01.md).
<book:include file="local-agent-qwen38-2026-11-01.md" />


- [Context placement measurements](local-agent-placement-2026-11-01.md).
<book:include file="local-agent-placement-2026-11-01.md" />


- [Historical human-code benchmark](historical-code-2026-11-01.md)
<book:include file="historical-code-2026-11-01.md" />

- [Engineer-first refactor and exact output comparison](engineer-first-refactor.md).
<book:include file="engineer-first-refactor.md" />

## Public maintained-source browser

The static site now builds a committed Markdown source mirror rooted at
`project.md` and serves it through the shared read-only Hickory document surface
at `/source.html`. Maintained-source changes trigger that deployment. The source
export tests prove that dirty files, untracked notes, sessions and environment
files cannot enter the committed publication; four focused component tests cover
navigation, read-only embedding, missing-page recovery, legitimate asset filenames,
path traversal refusal and mixed-revision/hash refusal. TypeScript checking and
the production site build passed during implementation.

The focused Chromium walkthrough opens the entry, follows the workspace chapter,
checks read-only state and default scaffold folding, reloads the deep link and
checks the mobile viewport. Local screenshots and the receipt are under
`target/dogfood/public-source-*`. These checks concern the static source browser,
not broad application regression coverage.

The previously failed site deployment refused a stale reviewed `Cargo.lock`
identity. Inspection against its reviewed revision found only the existing `fs2`
and `similar` dependencies newly listed for `hick-project-env`; example and
Rust-toolchain bytes were unchanged. The selection was updated after that review,
and all seven existing focused publication tests plus the real selected-example
publisher passed. The publisher's changed-input refusal remains active.

## Build-tutorial narrative and document-only reader

The six core chapters now teach the construction sequence rather than present a
component tour. Sixteen existing implementation and test fragments moved into the
steps that explain their decisions, retaining fragment IDs and output assembly.
The ownership ledger records the new locations. The literate-source skill now
requires this tutorial approach and the restrained hosted reading surface.

The hosted reader keeps only a left files pane and breadcrumbs around the shared
read-only document renderer. Its own prose links navigate the tutorial; Markdown
link syntax and standalone fragment anchors render without changing source text.
Code and literal examples remain verbatim. Six focused component and link tests,
TypeScript checking, skill validation and the production site build passed during
implementation. Chromium exercised in-document links, files, breadcrumbs, actual
owned tutorial code, folded scaffolds, reload, fragment anchors and mobile layout;
the local receipt is `target/dogfood/tutorial-reader-browser.json`.

Output comparison against the starting commit preserves every generated Rust,
native-host and code-model byte. Intentional output changes are the skill, hosted
reader and tests, operator documentation and ownership metadata; the new link
extension and its tests are added outputs. Complete reconstruction and comparison
receipts are retained locally under `target/dogfood/tutorial-*`. Existing fragment
explanation gaps remain explicit; the core tutorial does not claim that every
historical asset or inherited module has been rewritten.

The native inventory check found site build output and Python import caches.
`project.md` now declares only those specific additional artifact exclusions;
unrelated unconstructed files still refuse verification. Broad regressions and
post-commit monitoring remain the maintainer's responsibility.


## 2026-11-03: subscription Codex model transport

The built-in Python loop now accepts `codex-subscription` model inference. This
is separate from the external Codex ACP harness. Authentication uses supported
Codex account RPC and CLI status; no credential files are read into Hickory.
Native execution and integration tools are disabled, unexpected native items
are refused, and the macOS process boundary denies workspace content
and child commands, and only explicit API-provider selection can change billing.
Retained subscription turns restore the same provider across engine restart.

Executed focused witnesses:

- `cargo test --offline --locked -p hickory-cli --lib serve::codex_subscription::tests -- --nocapture`: ten fixtures passed, covering account modes, startup warnings, token accounting, process exit, cancellation, tool rejection, actual sandbox denials and provider selection with an API key present.
- `HICKORY_AGENT_TEST_PYTHON=…/apps/agent/.venv/bin/python cargo test --offline --locked -p hickory-cli --test python_agent -- --nocapture`: all nine real confined-worker safety checks passed. A pre-existing test selected the context JSON sidecar by directory order; it now selects the Markdown record explicitly and retains the same refusal assertions.
- `cargo test --offline --locked -p hickory-cli --test prediction_http native_run_route_uses_private_gate_and_alternate_http_surfaces_refuse_without_answers -- --exact`: the existing prediction-gate HTTP witness passed.
- `npm exec vitest run src/components/ConversationModelChoice.test.tsx src/components/ChatDock.test.tsx`: 37 UI fixtures passed. `npm run typecheck` also passed.
- `python3 scripts/test-subscription-codex.py --engine target/debug/examples/python_agent_ui_fixture --python-runtime apps/agent/.venv/bin/python`: native HTTP selection, signed-out status, supported sign-in request, original authentication failure, one attempted authentication check, unknown charges and selection retained across restart passed. Its fake Codex never starts inference or calls a paid provider.

The HTTP witness exposed an error-masking problem: duplicate worker delivery
requests replaced the original sign-in error with a generic uncertain-delivery
message. The delivery cache now replays the original success or failure without
making another inference request.

The working tree's normal `behavior.py generate` is blocked by existing pending
captures, and its `check` first reports the pre-existing desktop `Cargo.lock`
difference. Those unrelated products and captures were preserved. Generation and
checking of this feature's construction ran in an isolated source closure; full
source-only snapshot reconstruction and ownership audit were also checked. This
does not reconcile the unrelated working-tree drift.

No live account login, inference, exhausted allowance or credit consumption was
exercised. A packaged desktop browser callback, an existing real login, a reviewed
live edit, Stop, expired credentials and actual network/limit failures remain live
verification. The initial transport refuses non-macOS inference, captured images
and private predictor inference. Unknown counters and credit balances stay
unknown. Broad regressions and post-commit status monitoring remain with the
maintainer (`just checks-status`, or `gh run list` / `gh run view RUN_ID`).

</book:doc>
