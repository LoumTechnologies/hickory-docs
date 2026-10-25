<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/reconciliation/decisions.json

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/reconciliation/decisions.json">
{
  "authoring/unsaved-work-survives-closing-the-app.md": {
    "title": "Close or recover a document without losing the wrong draft",
    "scenario": "Close a changed note, choose Discard, then reopen the workspace. The discarded draft must stay discarded. A crash instead recovers the last retained draft as unsaved work.",
    "contract": "A buffer remains dirty until its actual file write is acknowledged. Tab close offers Save, Discard and Cancel; window close offers Save all, Close without saving and Cancel. Explicit discard removes the corresponding recovery draft and suppresses late writes from that discarded buffer. Save failure, cancellation or changed source identity keeps the affected view open. Each unnamed note has independent recovery outside the project. Named-file recovery follows its setting, which defaults off. Recovery does not create a project file or acknowledge a save. A recovered draft is compared with its recorded base and current file before restoration or merge. An absent editor view must not be read as an empty buffer.",
    "decision": "Fold the explicit-discard, in-flight-write and dead-view corrections into one current lifecycle rule. Keep crash recovery distinct from an intentional close without saving.",
    "limits": "Periodic recovery can lose input after its last successful flush. Frontend tests mock persistence and do not certify native window destruction or crash durability. Keep the original merge, path-renaming and startup-note clauses as detailed requirements.",
    "implementation": [
      "apps/web/src/lib/drafts.ts",
      "apps/web/src/lib/outputSave.ts",
      "apps/web/src/views/useUnsavedLifecycle.ts",
      "apps/web/src/lib/windowClose.ts"
    ],
    "tests": [
      {
        "runner": "vitest",
        "file": "apps/web/src/lib/drafts.lifecycle.test.tsx",
        "name": "waits for an unfinished recovery write and never rewrites explicitly discarded work"
      },
      {
        "runner": "vitest",
        "file": "apps/web/src/views/useUnsavedLifecycle.test.tsx",
        "name": "keeps an inactive view open when its attempted Save fails"
      }
    ]
  },
  "authoring/an-output-edit-lands-in-its-document.md": {
    "title": "Save a generated edit once and retain it when saving fails",
    "scenario": "Type a second change while the first generated-file save is pending. The next save must use the acknowledged baseline and retain the newer text.",
    "contract": "Supported generated-text edits map to their document source. The frontend computes each diff against the last content acknowledged by the backend; successful saves advance that baseline, failed saves leave it unchanged. Saves are serialized. Incoming output does not replace pending local text, and a discard request rechecks the expected buffer before and after awaiting earlier saves. The filesystem watcher also rechecks contributing source before publication and refuses stale or synthetic edits instead of overwriting unrelated work.",
    "decision": "Unify the watcher and HTTP entry points around acknowledged bytes while retaining their distinct concurrency boundaries. Remove historical duplication anecdotes from the current promise.",
    "limits": "The focused saver witnesses exercise queued callbacks and byte diffs. They do not establish end-to-end watcher convergence or crash-atomic multi-document publication.",
    "implementation": [
      "apps/web/src/lib/outputSave.ts",
      "crates/hickory-cli/src/up/reverse.rs"
    ],
    "tests": [
      {
        "runner": "vitest",
        "file": "apps/web/src/lib/outputSave.test.ts",
        "name": "serializes overlapping saves so each diff sees the previous result"
      },
      {
        "runner": "vitest",
        "file": "apps/web/src/lib/outputSave.test.ts",
        "name": "does not discard generated edits typed while an earlier save is settling"
      }
    ]
  },
  "agent/the-agent-sees-the-open-editors.md": {
    "title": "Select model context without saving open buffers",
    "scenario": "Keep unsaved text open but leave it out of Next context projection. Its edit identity may be captured, but opening the tab alone must not send its full text to the model.",
    "contract": "Open buffers can supply exact edit snapshots without becoming model-visible text. The current workspace projection explicitly selects expanded or pinned text for the next capture. Unselected open files contribute identity and edit snapshots, not their complete contents. Captured unsaved bytes remain unsaved. Named buffers retain file identity; identifiable untitled buffers use editor-only addresses rather than invented paths. Historical receipts describe what was actually delivered and do not certify today's source or claim support.",
    "decision": "The October 18 selected-reading requirement supersedes the earlier send-all-open-editors contract. Keep legacy delivery behavior as historical evidence and make expanded-reading-projection the current delivery authority.",
    "limits": "Opening an editor is not evidence of provider delivery. Opaque ACP filesystem tools retain their independent access. The historical HTTP witnesses do not establish the current Python projection.",
    "implementation": [],
    "tests": []
  },
  "agent/completion-does-not-certify-verification.md": {
    "title": "Distinguish an accepted change from a verified runtime result",
    "scenario": "An agent finishes a changeset and says the tests passed. Accepting source equality must not manufacture an executed-test receipt.",
    "contract": "Hosted Python completion requires a current independently accepted changeset checked against actual captured source bytes. A model summary, snippet print or declared check is insufficient. Iteration exhaustion and rejected completion remain incomplete. Equality establishes the captured source transition; runtime execution, semantic correctness and authorship require separate evidence. Direct generic shell/write/verify/debug primitives are withheld from the snippet surface, while registered attached-window operations may exercise existing host execution authority under their own policy and receipts.",
    "decision": "Reconcile the old blanket withholding wording with the later explicitly granted UI capabilities. Withholding a direct tool is not a claim that the agent cannot cause registered host execution.",
    "limits": "Source equality cannot certify semantic correctness. External adapter reports remain declarations unless independently observed. Registered terminal/program operations can modify workspace files beyond reviewed document edits.",
    "implementation": [],
    "tests": []
  },
  "agent/python-execution-has-two-boundaries.md": {
    "title": "Separate model snippets from trusted host operations",
    "scenario": "A model snippet cannot import a shell as a fallback. A separately registered terminal action can still run a host process under the existing host policy.",
    "contract": "Model snippets execute in Monty without workspace mounts or a CPython/shell fallback. The trusted Python orchestrator runs under supported Seatbelt or Bubblewrap confinement; unavailable confinement refuses hosted execution. Python owns its conversation graph, prompts, retries, budget and completion attempts. Hickory supplies bounded transport, reviewed document edits, host operations and retained receipts. Registered window tools can expose execution, debugger, setup and filesystem effects under local policy. Stop cancels inference/review and terminates the confined process group; it does not approve a proposal or guarantee termination of an independent job already launched by a completed UI operation.",
    "decision": "Retain both confinement layers and explicitly incorporate the later host-capability grant. Remove the implication that every possible workspace effect goes through document-edit approval.",
    "limits": "Trusted runtime installations and host policies remain assumptions. This is not an adversarial same-user security boundary. Standalone development has a different confinement boundary.",
    "implementation": [],
    "tests": []
  },
  "verification/a-recording-is-keyed-by-the-cells-inputs.md": {
    "title": "Reuse a recording only for the inputs that produced it",
    "scenario": "Change an authored script without changing the shell command. The old recording must become stale; writing the transcript itself must not invalidate its own key.",
    "contract": "A recording key includes the cell's command, relevant environment and features, declared seeded input bytes, and upstream cell identities. Literal document-produced inputs remain part of the digest. The document's own unstable outputs, weave targets and retained recording payloads do not key a cell on its own results. Seeded volume state is hashed before earlier cells mutate it; upstream outputs arrive through the upstream term. Unseeded output-only volumes contribute no input bytes. Cache directories are excluded at every depth. A broad output-path exclusion must not erase real inputs.",
    "decision": "Combine the September 2 and 3 input-digest amendments, including the explicitly withdrawn output-volume-prefix shortcut, into one rule.",
    "limits": "Correct identity coverage depends on declared inputs. A matching cache key does not establish correctness of an undeclared ambient dependency.",
    "implementation": [],
    "tests": []
  },
  "verification/a-weave-without-a-recording-keeps-the-artifact.md": {
    "title": "Keep an existing artifact when no recording can construct it",
    "scenario": "Open a captured project whose transcript is absent locally. Weaving must not replace a retained output with a never-run marker.",
    "contract": "When a required cell has no recording, weaving preserves existing outputs it cannot derive, including the woven report, and reports that preservation. A stale recording is different: weaving may display its last recorded bytes with stale status and write that rendering. Only an absent baseline is unrecorded. Explicit execution can refresh the source outputs; an unrecorded weave must not silently destroy retained evidence.",
    "decision": "Apply the September 3 stale-versus-unrecorded amendment to the September 2 preservation rule. Keep missing-baseline and changed-input outcomes separate.",
    "limits": "Retaining an artifact does not verify its current inputs. A newly absent output cannot be recovered from a recording that does not exist.",
    "implementation": [],
    "tests": []
  },
  "verification/test-separates-unverifiable-from-drifted.md": {
    "title": "Report whether verification found drift, no baseline or a false expectation",
    "scenario": "CI must distinguish a stale recording that needs regeneration from a cell with no baseline and from an expectation that actually failed.",
    "contract": "Verification reports verified, drifted, unverifiable or expectation-failed with their distinct public exit codes. A stale recording counts as drift; a wholly missing baseline counts as unverifiable. A frozen valid recording is a baseline. Unverifiable cells do not evaluate expectations or supply trustworthy drift comparisons. When several outcomes occur, expectation-failed outranks unverifiable, which outranks drifted, which outranks verified; all findings remain visible.",
    "decision": "Fold the stale-recording amendment into the four-outcome contract. Retain the public code mapping and severity ordering from the original detailed requirements.",
    "limits": "Exit status summarizes evidence, not application correctness. A failed expectation needs a requirement/code decision rather than automatic baseline regeneration.",
    "implementation": [],
    "tests": []
  },
  "execution/a-volume-carries-what-the-repository-carries.md": {
    "title": "Apply repository exclusions when seeding execution inputs",
    "scenario": "Mount the repository as a volume. Ignored build outputs must not become accidental inputs, and a cell's own transcript must not make its next recording unreachable.",
    "contract": "Repository volume seeding applies the repository's intended ignore filtering. Recording input digests additionally exclude the document's unstable generated products and weave targets. Literal authored inputs remain inputs. Removing the old self-mount warning does not remove ignore filtering or authorize a blanket exclusion of everything under an output directory.",
    "decision": "Keep the original ignore-filter promise and fold in the later removal of the obsolete self-mount warning. Delegate exact digest policy to the recording-key contract.",
    "limits": "Volume input filtering and executor confinement are separate boundaries. This does not police arbitrary filesystem writes by an unconfined process.",
    "implementation": [],
    "tests": []
  },
  "debugging/a-missing-debugger-is-a-button.md": {
    "title": "Install a supported missing debugger and resume the requested launch",
    "scenario": "Choose Debug on a supported program with no adapter installed. Setup should continue that launch without requiring another Debug click, and dismissal must block a late launch.",
    "contract": "A typed catalogue-supported missing-adapter result starts the existing confined installer and then retries the originating program. Setup exposes actual progress and diagnostics. Failure offers an explicit retry; repeated missing-adapter results do not cause an automatic installation loop. Dismissed or replaced setup cannot start a late session. Requests for the same adapter share installation. Unsupported adapters retain their ecosystem-specific guidance. Installer paths are shell-quoted and redirected home applies to the complete compound install.",
    "decision": "The current Debug action authorizes prerequisite setup and supersedes the former separate-install-button decision. Keep retry, cancellation and typed-error boundaries in the current statement.",
    "limits": "The UI witness uses a controlled installer response. It does not download an adapter, establish platform confinement or certify a packaged debugger.",
    "implementation": [
      "apps/web/src/debug/DebugStrip.tsx"
    ],
    "tests": [
      {
        "runner": "vitest",
        "file": "apps/web/src/debug/DebugStrip.test.tsx",
        "name": "installs a missing debugger and resumes without another click"
      },
      {
        "runner": "vitest",
        "file": "apps/web/src/debug/DebugStrip.test.tsx",
        "name": "does not start a session after setup is dismissed"
      }
    ]
  },
  "debugging/debug-uses-the-latest-project-containing-the-displayed-code.md": {
    "title": "Debug the latest atomic project that still contains the displayed bytes",
    "scenario": "A caller and library change belong to one atomic scope. Debugging must not combine the new caller with the old library simply because that intermediate state existed during editing.",
    "contract": "A launch selects the latest supported complete project endpoint containing the exact displayed code. Candidate states honor atomic scopes; dependency-local editing states do not automatically become launch candidates. Changed source or unavailable exact mapping refuses selection. The launch profile chooses the executable within that selected tree, while breakpoint routing preserves the viewed source identity.",
    "decision": "Use the atomic-prefix correction and exact-source launch-profile rule, replacing the earlier permissive cumulative-state interpretation.",
    "limits": "Source selection is separate from language-server endpoint choice and from a successful native launch. Exact-source checks must be repeated at the host boundary.",
    "implementation": [],
    "tests": []
  },
  "literate-projects/tools-select-complete-isolated-endpoints.md": {
    "title": "Keep language tools and debug launches tied to their captured project",
    "scenario": "Read one side of a comparison while another file is renamed. Its tools must use the captured side and filename; choosing a debugger must not silently change ordinary language-service scope.",
    "contract": "Language and debug tools receive explicit captured project and source identities. Ordinary document language services use the completed endpoint. File-block and generated-output debug requests identify the displayed code; the host rejects changed source and chooses the latest supported matching whole tree. Comparisons retain the immutable capture used by their tools, breakpoints and launch. Opening a comparison can register memory-only language context but does not execute the program. Removed or ambiguous mappings remain unavailable.",
    "decision": "Remove the historical global-selector and auto-open-different-endpoint expectations. Retain the later exact-source rule and revision-local filenames.",
    "limits": "A tool context is not an execution receipt. Additional native platforms and adapters require their own witnesses.",
    "implementation": [],
    "tests": []
  },
  "literate-projects/patch-edits-revalidate-downstream-construction.md": {
    "title": "Keep a patch edit as a draft until the whole construction still works",
    "scenario": "Change an early patch so a later patch no longer applies. The edit must remain an invalid draft rather than silently publishing a half-valid project.",
    "contract": "Editing authored patch code regenerates the patch against its captured predecessor and reevaluates the complete supplied closure. A downstream-breaking edit remains unchecked and cannot publish. Current source identity is checked before local publication; cross-document candidates use the native publication guard. Executable patch identities remain required for this editing surface. Old-side editing follows its declared permission rather than inheriting New-side authority.",
    "decision": "Fold the implemented editor/WASM/native-guard continuation into the contract and retire the old pending-wiring exclusion.",
    "limits": "Passing patch evaluation does not certify explanatory prose or runtime behavior. Actual native visual acceptance is separate from editor-controller tests.",
    "implementation": [],
    "tests": []
  },
  "literate-projects/finalization-and-incorporation-check-distinct-equalities.md": {
    "title": "Check the agent's transition separately from maintained-source reconstruction",
    "scenario": "An agent finishes B from captured A. Its explanation must reproduce that transition, while the maintained source must reproduce B from an empty tree.",
    "contract": "Finalization checks apply(explanation, A) equals B. Incorporation separately checks generate(maintained source, empty tree) equals B. Both proposed and reviewed candidates are rechecked before publication, including destination source identity and completed code. Already-present B is not applied a second time. Removing source retires its live rooms so late persistence cannot recreate it. Captured editor and dirty-source identities remain part of the boundary.",
    "decision": "Use the implemented built-in/Python/ACP capture and review-queue continuation. Remove obsolete pending-integration language while retaining the distinct equalities.",
    "limits": "Observed filesystem snapshots are not global transactions. Equal source trees do not prove that the explanation is accurate or that B behaves correctly.",
    "implementation": [],
    "tests": []
  },
  "lineage/a-ribbon-crosses-documents.md": {
    "title": "Trace a ribbon to the visible part of its source",
    "scenario": "Narrow or horizontally scroll the Files pane while a ribbon points into it. The endpoint must remain on the visible row or fall back to the divider.",
    "contract": "Cross-document lineage, context and declared links retain their recorded source identity and use the appropriate visible editor, Files row or divider endpoint. Files-row endpoints are clipped on both axes by every scrolling or clipping ancestor, excluding borders and scrollbar space. Fully clipped rows use the existing divider fallback. Tree scrolling schedules overlay updates. This presentation rule changes neither provenance semantics nor citation support.",
    "decision": "Fold the October 8 visible-row correction into the current endpoint contract. Retire the rejected Input-marker presentation proposal as history.",
    "limits": "The recorded geometry review assumes the shell's ordinary untransformed layout. Rotated/scaled custom hosts and semantic truth of citations are outside that witness.",
    "implementation": [],
    "tests": []
  },
  "integrations/the-desktop-finds-tools-from-the-shell-path.md": {
    "title": "Find installed tools when the desktop starts outside a terminal",
    "scenario": "Launch from Finder after configuring a language tool in your login shell. Tool discovery should use the resolved search path without waiting for shell logout hooks.",
    "contract": "Desktop launch resolves an absolute executable search path through the supported login/interactive shell setup within its bound. Discovery replaces the shell with the path-reporting command after startup profiles, avoiding logout hooks. A new window reuses a parent path only when its marked resolved value is unchanged; altered or unmarked values are rediscovered. Discovery failure retains explicit fallback behavior rather than blocking editor startup indefinitely.",
    "decision": "Include the October 17 shell replacement and parent-path reuse amendment; keep startup responsiveness separate from later workspace preparation.",
    "limits": "Shell profile side effects remain those of the user's configured shell. Native startup and each platform need their own acceptance evidence.",
    "implementation": [],
    "tests": []
  },
  "release/only-a-green-commit-becomes-downloadable.md": {
    "title": "Publish the committed Unstable build chosen by the maintainer",
    "scenario": "Publish Unstable from the configured Linux-to-Mac workflow. The payloads must come from the committed revision; the tool must not invent a green-CI gate that this workflow explicitly removed.",
    "contract": "Manual Unstable publication builds the exact committed HEAD present on GitHub through the configured Apple Silicon SSH builder. It uploads the DMG and signed app archive, then switches the updater manifest only after payload uploads succeed. Existing assets stay downloadable. Publishers using that builder share its lock. The command neither dispatches GitHub Actions nor gates or monitors regression results; the maintainer chooses publication timing. Stable release behavior is separate.",
    "decision": "The explicit October 25 manual-publication requirement supersedes the older automatic green-CI-only Unstable contract. Keep old stable-channel rules historical until separately reviewed.",
    "limits": "No publication is authorized or performed by this rewrite. Builder/signing configuration and real installability remain unverified. Manifest replacement is not atomic across GitHub failures, and separate builders can race.",
    "implementation": [
      "scripts/release-unstable.py",
      "scripts/build-unstable-macos.sh"
    ],
    "tests": []
  },
  "agent/a-session-is-the-conversation.md": {
    "title": "Read the retained branch of an agent conversation",
    "scenario": "Reopen a conversation after changing branches. The recorded selected branch must remain available; it must not be reconstructed from a model summary.",
    "contract": "The session retains prompts, responses, actions, observations and branch relationships as document evidence. The selected branch determines the conversation reading. Python owns current conversation/context construction; Rust provides supervision, tool boundaries, transport and recording. Abandoned work remains historical rather than silently becoming current context.",
    "decision": "Apply the Python-runtime amendment. Retain the existing session and branch vocabulary; retire references to the removed Rust ReAct loop.",
    "limits": "Opaque provider delivery and hidden model state are not reconstructed by a retained session.",
    "implementation": [],
    "tests": []
  },
  "agent/byo-agent-tool-surface.md": {
    "title": "Use the same document tools from an external coding agent",
    "scenario": "An external agent reads and edits a document through its anchored tool interface, then inspects the resulting source and evidence.",
    "contract": "The CLI document commands and MCP surface expose the shared anchored document and lineage tools. Edits recheck their supplied source identities and return their actual results. Optional session recording retains requests and observations. The built-in Python loop is a client of the shared host boundaries, not a second document-edit implementation. External adapters retain their own execution authority.",
    "decision": "Keep the common document-tool contract and apply the current Python ownership amendment. Separate the external agent interface from hosted snippet restrictions.",
    "limits": "A shared tool schema does not authenticate external agent claims or confine that agent's independent filesystem tools.",
    "implementation": [],
    "tests": []
  },
  "agent/changeset-completion-is-negotiated-and-checked.md": {
    "title": "Finish an agent turn against its captured project boundary",
    "scenario": "An accepted final response must explain the actual A-to-B change, including dirty source and editor identities captured at the start.",
    "contract": "Completion capability and project boundary are negotiated explicitly. Accepted artifacts are checked against the captured starting state and completed bytes. The current hosted Python path requires a current independently accepted changeset; model declarations do not bypass the host check. Changed editor identities, save/rename transitions or destination state require a new valid boundary. Source transition and maintained-source incorporation remain separate checks.",
    "decision": "Apply the current Python completion requirement and retire the older statement that built-in/Python mandatory finalization is not wired.",
    "limits": "Filesystem snapshots are observed reads rather than a global transaction; equality does not establish runtime behavior.",
    "implementation": [],
    "tests": []
  },
  "agent/reasoning-is-shown-apart-from-the-answer.md": {
    "title": "Keep model reasoning distinct from the answer",
    "scenario": "Inspect a recorded turn without mistaking a reasoning block for the final response or an executed check.",
    "contract": "The conversation distinguishes reasoning, final response and tool observations using their recorded roles. The current Python runtime owns prompt/history construction; the shared session and UI retain the separate recorded content. Reasoning text does not execute tools, certify completion or establish a check result.",
    "decision": "Retain role separation while applying the Python-runtime amendment. Earlier Rust-loop test claims remain historical.",
    "limits": "Provider-specific availability of reasoning is not a promise to expose hidden model state.",
    "implementation": [],
    "tests": []
  },
  "agent/the-agent-is-told-the-language.md": {
    "title": "Teach the agent the document syntax its tools will accept",
    "scenario": "Ask the agent to produce an executable document. Its syntax guidance should match the supported elements instead of inventing attributes.",
    "contract": "The current built-in Python runtime supplies document-language guidance and shared tool descriptions. That guidance identifies supported elements and their semantics, including the declared boundaries for diagrams and generated source. Host parsing and validation remain authoritative; a prompt is not an alternative grammar or permission boundary.",
    "decision": "Move ownership of the prompt to the current Python runtime while preserving the requirement that the advertised document vocabulary be accurate.",
    "limits": "A documented syntax list does not guarantee that a model follows it or that every explanation is accurate.",
    "implementation": [],
    "tests": []
  },
  "agent/an-answer-in-the-agent-pane-has-ribbons.md": {
    "title": "Trace conversation passages without confusing context with support",
    "scenario": "Follow a link from an answer to the unsaved editor snapshot supplied to that turn. The link must describe historical input, not claim that it proves the answer.",
    "contract": "The live Agent pane uses the protected conversation editor over the ordinary document editor. Recorded and streaming content remains protected; the response region remains editable. Context links use the actual recorded branch and source snapshots, including identifiable untitled editor addresses. They can attach to answer paragraphs in the same user turn and remain visible through folded bookkeeping. A new user turn does not inherit the previous turn's input links. The pane follows normal workspace stacking so connections remain visible.",
    "decision": "Fold the live-document, explicit-buffer-identity, per-turn context and pane-stacking amendments into the current reading contract. Keep the standalone read-only session-card view separate.",
    "limits": "Context records what was supplied. It does not establish derivation, entailment or current source freshness.",
    "implementation": [],
    "tests": []
  },
  "authoring/adoption-is-byte-exact-or-refused.md": {
    "title": "Adopt a plain file without changing its bytes",
    "scenario": "Make a plain source file literate. The new document must regenerate the exact original file before adoption is accepted.",
    "contract": "Supported plain-text adoption verifies the constructed output against the original before keeping the change. Failure restores the affected source/output state. Appending refuses an already-owned path and preserves the document's other generated products; its own woven reading may change as the authored document changes. The current adoption path refuses unsupported binary/document inputs and source syntax it cannot represent exactly.",
    "decision": "Fold the bare-document amendment into the preservation rule. Do not promise that the woven reading stays unchanged when the maintained document itself changes.",
    "limits": "Declared-project raw/hex entries can construct inputs that the plain-file adoption command does not support; those are separate interfaces.",
    "implementation": [],
    "tests": []
  },
  "authoring/an-output-that-cannot-be-carried-back-is-held.md": {
    "title": "Keep divergent output until an explicit decision names the write",
    "scenario": "A generated file differs from the document and cannot be mapped back. Leaving the warning must not silently resolve the conflict.",
    "contract": "Unreproducible generated output is retained as divergent rather than overwritten. The surface names the write consequence: generated files offer Leave disk unchanged or Regenerate from document; plain buffers offer Overwrite disk with this buffer or Reload from disk. Merge uses the recorded base and current sides under their source guards. Dismissing a notice is not resolution. Kept unrecorded output and an external change under an unsaved buffer remain distinct reasons for divergence.",
    "decision": "Retain the common divergence mechanism and replace ambiguous keep-mine/take-theirs labels with the later explicit-consequence wording.",
    "limits": "Retaining a divergent file is not verification. Per-target guards do not form a filesystem transaction across all products.",
    "implementation": [],
    "tests": []
  },
  "debugging/a-compiled-language-launches-what-a-build-produced.md": {
    "title": "Launch the artifact built from the selected compiled project",
    "scenario": "Debug a Rust or C# program described by the current project. The debugger must receive the selected build artifact rather than a guessed source-file stem.",
    "contract": "Compiled-language launch uses the language's declared build step, artifact location and executable selection. Rust builds in its manifest directory, uses the package manifest to identify the program, and excludes dependency-note files from executable selection. Adapter discovery prefers a supported installed adapter; catalogue installation can supply supported archive-based C# and Rust adapters. Build failure retains actual diagnostics rather than starting a different program.",
    "decision": "Fold Rust/codelldb and the available C# archive installer into the earlier compiled-language contract. Retire the obsolete claim that C# has no installer.",
    "limits": "A successful build is not behavioral verification. Platform adapter availability and live stop/source mapping need independent witnesses.",
    "implementation": [],
    "tests": []
  },
  "debugging/an-adapter-that-ships-as-an-archive-installs-like-any-other.md": {
    "title": "Install a supported debugger archive into its declared prefix",
    "scenario": "Install codelldb from its VSIX or another catalogue archive. The executable must be found in the declared extraction layout.",
    "contract": "Catalogue archive installation selects a supported platform asset, extracts into its declared prefix and locates the declared executable. Archive layouts can include an explicit destination directory, including codelldb's extension tree. Executable metadata and bundled runtime layout are retained where supported. Existing user-installed adapters remain discoverable.",
    "decision": "Fold the codelldb archive shape into the original archive-install rule instead of keeping a second historical exception.",
    "limits": "A catalogue entry is not a passed native-platform installation witness. Unsupported platforms fail explicitly.",
    "implementation": [],
    "tests": []
  },
  "debugging/debug-controls-preserve-editor-space.md": {
    "title": "Choose a launch without replacing the editor being read",
    "scenario": "Open the launch picker while reading generated code. Choosing a target should not replace the editor or silently switch the code being debugged.",
    "contract": "Debug controls preserve the mounted editor and its available reading space. Launch profiles use the current floating picker and exact displayed-source selection. Source identity is rechecked by the host; a stale or unavailable mapping remains unavailable. Ordinary language-service scope is independent of the selected debugger launch.",
    "decision": "Use the launch-profile continuation instead of the earlier global version-menu and replacement-editor wording.",
    "limits": "A controller test does not establish native visual layout across every viewport.",
    "implementation": [],
    "tests": []
  },
  "editor-intelligence/actions-belong-to-their-subject.md": {
    "title": "Keep editor actions attached to the view they affect",
    "scenario": "Move or focus a spatial pane. Its action controls must still target that pane without moving its editor buffer.",
    "contract": "Actions retain their owning document/file/view identity and current availability. Current metadata and document/file controls use each spatial pane's existing header. View changes preserve the mounted editor and source identity. A source revision or subject change invalidates stale action context.",
    "decision": "Apply the minimalist-editor amendment while retaining the explicitly preserved visible ruler and action-rail collision behavior.",
    "limits": "Subject identity and visual placement do not establish that every native action has a registered agent equivalent.",
    "implementation": [],
    "tests": []
  },
  "lineage/a-ribbon-is-drawn-where-it-is-asked-for.md": {
    "title": "Show the requested connections without changing their meaning",
    "scenario": "Focus a source block, then inspect its connections in a folded or grouped view. Grouping must not invent new provenance.",
    "contract": "Connections are drawn according to the current requested source/caret/fold visibility policy. Grouping joins eligible neighboring presentations without changing source identity, destination, attribution family or byte evidence. Intervening prose and distinct destinations or provenance families remain separate.",
    "decision": "Retain the request-driven visibility rule and include the later coalescing requirement. Keep browser-appearance acceptance separate from grouping tests.",
    "limits": "A grouped drawing is presentation, not a new claim of semantic support.",
    "implementation": [],
    "tests": []
  },
  "debugging/an-adapter-that-listens-is-connected-to.md": {
    "title": "Use the adapter\u2019s transport and the current measured language support",
    "scenario": "Start an adapter that listens on TCP rather than reading stdin. Connection transport and language support must come from the current catalogue, not a dated installation note.",
    "contract": "A server-style adapter starts with its declared port placeholder substituted and uses DAP over the selected TCP connection; pipe-style adapters retain their declared transport. Discovery carries required launch metadata. Current JavaScript/TypeScript discovery and live-suite declarations include multi-session support; the old claim that they cannot be driven is historical. React component files are not standalone Node launch programs, and Ruby remains unavailable in the current discovery guidance. Finding an executable or installing an archive alone does not establish a live debugging witness.",
    "decision": "Replace the obsolete JavaScript/TypeScript unavailability paragraph with the current shared plugin discovery and live coverage declarations. Keep TCP transport and measured support as distinct conditions.",
    "limits": "The source inspection does not run a real adapter. Catalogue/live-suite agreement is distinct from a current machine having that adapter installed; no installer or network operation was executed.",
    "implementation": [
      "crates/hick-plugins/src/dap_discovery.rs",
      "crates/hick-plugins/src/languages/typescript.rs",
      "crates/hick-dap/src/adapter.rs"
    ],
    "tests": []
  },
  "editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md": {
    "title": "Run and test from the pane menu without adding a toolbar row",
    "scenario": "Open the document menu while a run is active. Run and Test must refer to that document and stay disabled until the run permits another action.",
    "contract": "Run executes the document; Test matches the hick test operation. These document-wide actions use the pane-header menu, while per-cell actions remain on the editor rail. Run and Test are disabled while a document run is in flight. A baseline action is named for the output baseline it establishes. The problem count opens the cross-document problem list; F8 retains next-problem navigation.",
    "decision": "Fold the dense-editor amendment into the action-placement contract. Keep earlier dedicated-toolbar rationale as history rather than a competing layout requirement.",
    "limits": "The focused pane test uses injected session callbacks. It does not establish native menu placement, the runtime test outcome or every problem-list interaction.",
    "implementation": [
      "apps/web/src/views/workspaceTabs.tsx"
    ],
    "tests": [
      {
        "runner": "vitest",
        "file": "apps/web/src/views/workspaceTabs.test.tsx",
        "name": "disables both buttons while the document's run is in flight"
      }
    ]
  }
}
</maint:entry>
</maint:doc>
