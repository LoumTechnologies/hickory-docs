<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Agent

For: Authors using an agent to change documents while retaining review control.

Follow a proposed change from context selection through review, application and retained evidence.

- [A Claude Code Transcript Imports As A Session](a-claude-code-transcript-imports-as-a-session/contract.md) — retained-requirement.
- [A Missing Key Degrades to a Note, Never a Crash](a-missing-key-degrades-to-a-note/contract.md) — retained-requirement.
- [A Provider Key Entered in Settings Stays on This Machine](keys-are-stored-locally-and-never-leave/contract.md) — retained-requirement.
- [Read the retained branch of an agent conversation](a-session-is-the-conversation/contract.md) — reconciled-requirement.
- [A Session Names the Provider It Actually Ran On](a-session-names-the-provider-it-actually-ran-on/contract.md) — retained-requirement.
- [A desktop app bundles its own Python agent](a-desktop-app-bundles-its-python-agent/contract.md) — retained-requirement.
- [A final changeset revision applies B to C](final-changeset-revisions-apply-b-to-c/contract.md) — retained-requirement.
- [A protected workspace save is validated against the revision it read](the-mounted-workspace-keeps-source-and-output-together/contract.md) — retained-requirement.
- [A running agent can be stopped](a-running-agent-can-be-stopped/contract.md) — retained-requirement.
- [ACP Agents Work In The Agent Pane And Leave A Hickory Record](acp-agents-are-first-class/contract.md) — retained-requirement.
- [An Agent Cell Is A DAG Vertex, And Its Edges Are A Barrier](an-agent-cell-is-a-dag-barrier/contract.md) — retained-requirement.
- [An Agent Edit Session Never Applies A Stale Edit](edit-session-staleness-impossible/contract.md) — retained-requirement.
- [An Agent Recording Is Keyed By The Prompt And The Model](an-agent-recording-is-keyed-by-prompt-and-model/contract.md) — retained-requirement.
- [Trace conversation passages without confusing context with support](an-answer-in-the-agent-pane-has-ribbons/contract.md) — reconciled-requirement.
- [An Empty Response Never Kills a Session](an-empty-response-never-kills-a-session/contract.md) — retained-requirement.
- [Use the same document tools from an external coding agent](byo-agent-tool-surface/contract.md) — reconciled-requirement.
- [An Upstream Refusal Names The Declaring Document](an-upstream-refusal-names-the-declaring-document/contract.md) — retained-requirement.
- [An exhausted Python iteration budget retains incomplete work](an-exhausted-turn-budget-hands-off/contract.md) — retained-requirement.
- [Assumptions remain auditable after the work](assumptions-remain-auditable-after-the-work/contract.md) — retained-requirement.
- [Captured reading surfaces share owned native actions](scoped-reading-surfaces-share-native-actions/contract.md) — retained-requirement.
- [Finish an agent turn against its captured project boundary](changeset-completion-is-negotiated-and-checked/contract.md) — reconciled-requirement.
- [Distinguish an accepted change from a verified runtime result](completion-does-not-certify-verification/contract.md) — reconciled-requirement.
- [Context Provenance Is Derived From The Session, Never From The Model](context-provenance-is-derived-from-the-session/contract.md) — retained-requirement.
- [Context protocol validates declared byte lineage before integration](context-protocol-validates-lineage/contract.md) — retained-requirement.
- [Conversation edits use the client's review policy](conversation-edits-use-the-client-review-policy/contract.md) — retained-requirement.
- [Explicit UI action dispatch checks scope, state, policy and retries](ui-action-dispatch/contract.md) — retained-requirement.
- [Explicit context updates preserve confirmed history](explicit-context-updates/contract.md) — retained-requirement.
- [Historical account-key selection](byok-key-selection/contract.md) — historical.
- [Historical evidence stays historical](historical-evidence-stays-historical/contract.md) — retained-requirement.
- [Historical hosted plan billing](byok-plan-boundary/contract.md) — historical.
- [Historical hosted-account key encryption](byok-key-at-rest/contract.md) — historical.
- [Host context observations are immutable and changes are source-backed](immutable-context-lineage/contract.md) — retained-requirement.
- [Local refresh experiments preserve historical request evidence](refresh-experiments-preserve-evidence/contract.md) — retained-requirement.
- [Model snippets cannot hang or flood an agent session indefinitely](a-script-cannot-hang-or-flood-a-session/contract.md) — retained-requirement.
- [Separate model snippets from trusted host operations](python-execution-has-two-boundaries/contract.md) — reconciled-requirement.
- [Python model requests carry validated context manifests](python-requests-have-validated-context/contract.md) — retained-requirement.
- [Re-Preparation After An Agent Edit Terminates, Under A Declared Bound](re-preparation-terminates/contract.md) — retained-requirement.
- [Recording failure is visible](recording-failure-is-visible/contract.md) — retained-requirement.
- [Rewind And Re-Run Are Named Apart, And The Difference Is What Is Kept](rewind-and-re-run-are-two-different-acts/contract.md) — retained-requirement.
- [Selected reading scope remains separate from complete edit state](expanded-reading-projection/contract.md) — retained-requirement.
- [The Agent Can Author A Diagram](the-agent-can-author-a-diagram/contract.md) — retained-requirement.
- [The Agent pane is a live document](the-agent-pane-is-a-live-document/contract.md) — retained-requirement.
- [Teach the agent the document syntax its tools will accept](the-agent-is-told-the-language/contract.md) — reconciled-requirement.
- [The Dock Reports Spend and Runs the Chosen Model](the-dock-reports-spend-and-runs-the-chosen-model/contract.md) — retained-requirement.
- [The Editable Set Is the Pipeline Closure](the-editable-set-is-the-pipeline-closure/contract.md) — retained-requirement.
- [Keep model reasoning distinct from the answer](reasoning-is-shown-apart-from-the-answer/contract.md) — reconciled-requirement.
- [Select model context without saving open buffers](the-agent-sees-the-open-editors/contract.md) — reconciled-requirement.
- [Unsent responses survive restart](unsent-responses-survive-restart/contract.md) — retained-requirement.
- [Verify Covers the Whole Editable Set](verify-covers-the-whole-editable-set/contract.md) — retained-requirement.
- [`hick test` Never Spends The Reader's Tokens](test-never-spends-tokens/contract.md) — retained-requirement.

<book:include file="a-claude-code-transcript-imports-as-a-session/contract.md" />
<book:include file="a-desktop-app-bundles-its-python-agent/contract.md" />
<book:include file="a-missing-key-degrades-to-a-note/contract.md" />
<book:include file="a-running-agent-can-be-stopped/contract.md" />
<book:include file="a-script-cannot-hang-or-flood-a-session/contract.md" />
<book:include file="a-session-is-the-conversation/contract.md" />
<book:include file="a-session-names-the-provider-it-actually-ran-on/contract.md" />
<book:include file="acp-agents-are-first-class/contract.md" />
<book:include file="an-agent-cell-is-a-dag-barrier/contract.md" />
<book:include file="an-agent-recording-is-keyed-by-prompt-and-model/contract.md" />
<book:include file="an-answer-in-the-agent-pane-has-ribbons/contract.md" />
<book:include file="an-empty-response-never-kills-a-session/contract.md" />
<book:include file="an-exhausted-turn-budget-hands-off/contract.md" />
<book:include file="an-upstream-refusal-names-the-declaring-document/contract.md" />
<book:include file="assumptions-remain-auditable-after-the-work/contract.md" />
<book:include file="byo-agent-tool-surface/contract.md" />
<book:include file="byok-key-at-rest/contract.md" />
<book:include file="byok-key-selection/contract.md" />
<book:include file="byok-plan-boundary/contract.md" />
<book:include file="changeset-completion-is-negotiated-and-checked/contract.md" />
<book:include file="completion-does-not-certify-verification/contract.md" />
<book:include file="context-protocol-validates-lineage/contract.md" />
<book:include file="context-provenance-is-derived-from-the-session/contract.md" />
<book:include file="conversation-edits-use-the-client-review-policy/contract.md" />
<book:include file="edit-session-staleness-impossible/contract.md" />
<book:include file="expanded-reading-projection/contract.md" />
<book:include file="explicit-context-updates/contract.md" />
<book:include file="final-changeset-revisions-apply-b-to-c/contract.md" />
<book:include file="historical-evidence-stays-historical/contract.md" />
<book:include file="immutable-context-lineage/contract.md" />
<book:include file="keys-are-stored-locally-and-never-leave/contract.md" />
<book:include file="python-execution-has-two-boundaries/contract.md" />
<book:include file="python-requests-have-validated-context/contract.md" />
<book:include file="re-preparation-terminates/contract.md" />
<book:include file="reasoning-is-shown-apart-from-the-answer/contract.md" />
<book:include file="recording-failure-is-visible/contract.md" />
<book:include file="refresh-experiments-preserve-evidence/contract.md" />
<book:include file="rewind-and-re-run-are-two-different-acts/contract.md" />
<book:include file="scoped-reading-surfaces-share-native-actions/contract.md" />
<book:include file="test-never-spends-tokens/contract.md" />
<book:include file="the-agent-can-author-a-diagram/contract.md" />
<book:include file="the-agent-is-told-the-language/contract.md" />
<book:include file="the-agent-pane-is-a-live-document/contract.md" />
<book:include file="the-agent-sees-the-open-editors/contract.md" />
<book:include file="the-dock-reports-spend-and-runs-the-chosen-model/contract.md" />
<book:include file="the-editable-set-is-the-pipeline-closure/contract.md" />
<book:include file="the-mounted-workspace-keeps-source-and-output-together/contract.md" />
<book:include file="ui-action-dispatch/contract.md" />
<book:include file="unsent-responses-survive-restart/contract.md" />
<book:include file="verify-covers-the-whole-editable-set/contract.md" />

</book:doc>
