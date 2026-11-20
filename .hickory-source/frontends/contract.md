<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build equivalent IDE workflows across the simple frontends

For maintainers implementing the complete IDE contract across browser, macOS,
iOS and Android while preserving the compact editor. Earlier dated slice receipts
retain their original limits; they do not narrow the required outcomes below.

## IDE parity required by the maintainer

The requested parity is the set discussed in the IDE comparison, across browser, macOS, iOS and Android. Keep operations discoverable through existing menus, left/right gutters, minimap and status bar. Floating controls must not resize code. Typed workspace state and replaceable capability adapters own behavior independently of the renderer; no plugin marketplace or permanent feature toolbar is required.

The complete outcome includes:

- Whole-program navigation: files, types, members, usages, declarations, implementations, callers/callees, file outlines, text search, action search and back/forward history.
- Safe changes: previewed multi-file rename, provider refactorings (including extract/move when supported), code actions, quick fixes, formatting/cleanup, code generation/templates, completion snippets/import edits and a coordinated undo path. Authored/generated ownership and source revisions remain authoritative.
- Diagnosis: workspace Problems with next/previous navigation and source-bound contextual/bulk fixes.
- Testing: discovery, source-adjacent run/debug, retained results and sessions, failure-to-source navigation, coverage and truthful source-revision evidence.
- Review: source/generated diffs, staging, history, blame and conflict resolution, plus local history, crash recovery and session/layout restoration.
- Debugging: verified supported-language adapters, profiles/attach, breakpoints/conditions, frames/threads, watches/evaluation and authored-source navigation.
- Hickory behavior readings: contracts, explanations, implementation fragments, generated consequences, related tests and execution evidence are connected through declared or observed relationships whose status remains explicit.
- Change impact: affected constructions/mechanisms and relevant checks explain their relationship, with unknowns retained. Evidence beside a claim distinguishes current, stale, manual and missing verification.
- Agent review: coherent proposals expose explanation, implementation, generated consequences and checks before guarded application; inference is distinguished from observed evidence.

These are required outcomes. Existing verification paragraphs establish only the explicitly tested slices; unsupported providers, missing relationships and unexecuted native acceptance must remain visible. JetBrains-specific language engines are supplied through existing language/provider interfaces; the UI must never claim an unavailable refactoring or an unmeasured test result.

## Current integration ledger, 2026-11-18

This ledger distinguishes exercised paths from mechanisms awaiting integration.
Dated receipts below retain their original test scope; a later receipt supersedes
an earlier open item only for the behavior it actually exercises.

| Required outcome | Current usable evidence and remaining acceptance |
| --- | --- |
| Navigation and code intelligence | Search Everywhere, file and saved-text search, outlines, semantic locations and back/forward have focused receipts. Genuine basedpyright covers callers and callees in Simple Browser, mounted AppKit, the iPhone simulator and stock Android. Browser and Android reached exact cross-file results while retaining the unsaved origin. Configured csharp-ls adds browser definitions, usages and history. Genuine clangd supplied both type relationships in Simple Browser, mounted AppKit, the iPhone simulator and stock Android. The focused replays retained unsaved source and reached exact names. Installed basedpyright and TypeScript Language Server read ordinary Markdown fences on browser, mounted macOS, iPhone simulator and rendered Android. Mounted iOS passed real basedpyright cross-file Definition and native Back and Forward. An Android controller navigated into a literal source tab while Agent was open. Index breadth depends on the installed provider. Unavailable capabilities and unsupported fence actions leave authored text readable. |
| Refactoring, cleanup, generation, templates and history | Genuine basedpyright covers grouped cross-file Rename through preview, draft Apply, coordinated Undo and Redo in Simple Browser, mounted AppKit, the iPhone simulator and stock Android. Configured csharp-ls also covers browser rename. Genuine clangd formatting through review, Apply and workspace Undo/Redo covers CRLF Markdown-owned C++ in Simple Browser, mounted AppKit and the iPhone simulator, plus LF Markdown-owned C++ in stock Android. TypeScript Language Server 6.1.0 with TypeScript 6.0.3 covers extraction, accessor generation, cleanup and Move to a new file on browser, mounted macOS and the Android controller. Guarded native action and formatting receipts include controlled providers. Preview holds the complete edit. Missing preimages, commands without supported effects and changed owner revisions refuse the group. Later template receipts close the earlier embedded and multi-caret catalog gaps. Expanded regex and source-bound variable programs retain their syntax and work bounds. Mounted iOS and rendered Android covered reviewed Cleanup. Mounted iOS also replayed native Paste, coordinated Cleanup Undo/Redo and the genuine CRLF and LF TypeScript extraction. |
| Diagnostics and contextual fixes | Genuine basedpyright browser and rendered Android Problems, configured csharp-ls browser Problems, and clangd mounted iOS Problems exercised exact source navigation. Real clangd engine coverage includes the missing-semicolon quick fix. Controlled-child receipts cover complete-file `source.fixAll` acquisition, review, draft Apply and Undo/Redo on browser, mounted macOS and the Android controller, including a prose-origin action that acquires its copy contributor. Stale and unversioned diagnostics stay readable. Precise navigation and application require matching authority. Mounted iOS and rendered Android exercised the genuine clangd missing-semicolon Quick Fix through guarded review, draft Apply and coordinated Undo/Redo. Provider workspace completeness and additional source mappings remain open. |
| Testing and coverage | Discovery, failed results, coverage and failed-only reruns have provider receipts for pytest, Jest, Vitest, Cargo with cargo-llvm-cov, and .NET with Coverlet. Connected pytest runs on browser, macOS and the Android controller. Mounted iOS executed selected pytest and rendered a collected failure and coverage. Rendered Android ran the saved Rust case and retained exit 0. Rendered Android discovered both pytest cases, retained a coverage run and reran only the deliberate failure. Source-adjacent Run and Debug distinguish process exit from a provider verdict. Unsupported runners and stale, missing or ambiguous test identities refuse selection. Coverage navigation requires matching source hashes. |
| Review, staging, attribution, conflicts and recovery | Browser, macOS and Android controller receipts cover Git status/diffs, history, attribution, shared conflict composition including deletion/rename, staging, local source history and guarded Save As. Rendered Android compared all three conflict stages and kept reviewed Save separate from Stage. Later complete 32-session recovery management and three exact revisions supersede the earlier capacity gap. Browser reload, native restart and mounted iOS compact recovery cover retained drafts, layout and view state. Shared publisher Move and interruption have rendered browser, mounted macOS and Android controller receipts. Mounted iOS completed cold recovery with the original path present and deleted, plus the six-publication older Move chain. Rendered Android completed cold recovery with the original path present and already deleted. Rendered Android completed both dirty Moves and both pairs of workspace Undo and Redo, preserving exact saved bytes, draft text, entry identity and editor frame. A separate API 36 replay entered both destinations with Gboard and confirmed the full reviewed Move, Undo and Redo chain. |
| Debugging | Genuine debugpy launch/loopback attach and LLDB canonical C launch/PID attach pass on browser, macOS and the Android controller. Scoped receipts cover profiles, conditional breakpoints, threads, deep frames, evaluation, source navigation, Continue and retained outcomes. Browser, mounted macOS and mounted iOS application launches also passed with netcoredbg for .NET and js-debug for Node.js. Saved source-test Debug has verified pytest/debugpy and Cargo/LLDB adapters. Rust binds the selected declaration to the compiled inline or external-module function before exact harness selection, with engine and host acceptance for plain and compiler-authored source. Mounted iOS reached the real selected Rust breakpoint after a build exceeded 30 seconds. Rendered Android paused the selected pytest test and a netcoredbg .NET application at their saved lines, continued and retained exit 0. Rendered Android reached a genuine saved Node.js breakpoint, evaluated a scoped Watch expression and continued to retained Finished. |
| Connected behavior readings | Browser, mounted macOS and Android controller have guarded contract, explanation, implementation, generated, test and evidence readings through the captured construction graph, including dirty contributors. Mounted iOS opened the connected reading and exercised its source-staleness boundary. Default entry reads carry their original proof through execution admission. Missing mappings and stale captures retain their own status. |
| Change impact and relevant verification | Browser, mounted macOS and Android controller have scoped compiler consequences, bounded declared relationships, selected checks/profiles and retained process-only results. Mounted iOS reached the selected process-only receipt from its connected reading. Undeclared dependencies remain unknown. A command exit supplies process evidence. Independent verification requires a source-bound receipt. |
| Agent proposal review and guarded application | Owned managed-provider flows on browser, macOS and the Android controller cover a sealed complete review, original-context application, coordinated draft history and separate resource publication. Rendered Android and mounted iOS read sealed historical material without restoring application authority. Mounted iOS also completed a guarded resource rename and its reviewed inverse. AppKit, iPhone simulator and rendered Android completed native Send through coordinated draft history using a local ACP fixture. External provider breadth remains open. |

Stock Android opened the connected reading from the native View menu, followed
the captured source proof to the full Answer contract and ran the selected
related `test_selected` case. The retained process-only result matched the
original request and exited 0. The unrelated case stayed outside the run.
Both explicit Close and Android Back preserved the exact editor frame, buffer,
selection, viewport, focus and saved file metadata without opening the keyboard.
The rendered receipt is in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-qq140t6k/connected-behavior/receipt.json`.

Across all rows, the evidence distinguishes engine protocols, native controllers,
mounted AppKit/UIKit, rendered browser and rendered Android. Physical input,
device IME and assistive technology need their own acceptance. The emulator
originally refused userdata allocation. An owned API 36 emulator booted after
inactive compiler caches were reclaimed, and the checkpoint Android APK built.

The complete-file `source.fixAll` flow still needs a rendered Android replay.
The iPhone has retained pytest failures, coverage and source navigation.
Its rendered failed-only rerun and configured debugger Attach still need
their own replays.

The reusable Insert template catalog has guarded Preview, Apply and history
in the main and Simple browsers, mounted AppKit and the Android controller.
Rendered UIKit and Android still need the same catalog flow. Android's
configured debugger Attach has genuine controller evidence and still needs
a rendered replay.

The signed iPhone simulator build sent exact whole-author `source.fixAll`
ranges and retained the distinct opaque provider choices across acquisition.
Native review, Apply and workspace Undo and Redo changed both drafts in
one step. The second author stayed unopened until the changes were reviewed.
Opening it afterward showed the exact applied draft. The first editor kept
its frame and history, and saved bytes and file metadata stayed intact. See
`target/ios-complete-file-fix/combined-literal-1/complete-file-fixes/attempt-2/receipt.json`.

### iPhone Python definition and navigation history

The basedpyright fixture resolved the caller in `main.py` to
`helper.py:4:5` through the real engine. The first mounted replay opened the
caller and reached the native Edit menu. The definition command was below
the visible rows, so the replay stopped before sending the request. The
fixture now scrolls the menu to reveal the commands for definition, Back
and Forward. The next mounted run scrolled that menu and got a real
basedpyright reply. The native test tap had selected the blank second row,
so the provider returned an empty definition and the run stopped before
reveal or history. A direct request at the call's first character resolved the
helper declaration through genuine basedpyright.

The first native Find replay typed the exact call text and showed one match.
The iPhone keyboard's first-use introduction blocked the Next button. The
revised arm64 replay dismissed that introduction, submitted Find with native
Return, closed search and returned to the caller. It was stopped while
revealing Go to definition as free disk fell to 116 MiB. No definition
request or Back/Forward occurred in that attempt. Starting with 3,397 MiB
free proved insufficient on this host, so another mounted run awaits more
headroom. All owned simulators were deleted. See
`target/apple-python-definition-uikit-2026-11-19/` and
`target/apple-python-definition-uikit-retry-2026-11-19/` for the earlier runs,
`target/apple-python-definition-uikit-find-d2d9f531/` for the keyboard-blocked
run, and `target/apple-python-definition-uikit-quickpath-2026-11-19/` for the
interrupted run. The updated bundle build is in
`target/ios-python-definition-quickpath-arm64.log`.

The current iPhone simulator sent `main.py:3:1` to basedpyright 1.1.414
and received `helper.py:4:5`. The enabled result initially lay below the
visible language panel. A native scroll exposed it and the hit opened
the exact declaration. Navigate back and Navigate forward restored the
original frames and full texts of the caller and helper. The saved files
kept their bytes. See
`target/apple-python-definition-uikit-v10-scroll/python-definition-receipt.json`.

### iPhone Git review with an unsaved draft

The iPhone simulator opened Repository changes from the View menu after
typing a distinct unsaved draft. Working Tree and Staged both displayed
the exact committed-to-saved diff. Stage wrote the saved working file's
bytes to the index. Unstage restored the committed blob and left an empty
staged diff. The editor showed the same draft bytes and frame afterward.
The saved working file and HEAD stayed unchanged. The disposable simulator
was deleted. See `target/apple-git-review-uikit-retry-2026-11-19/`.

A separate iPhone replay opened saved attribution from a dirty editor.
The stable saved line offered its exact commit, and the changed saved line
offered no commit. Attribution and file-scoped Saved file history both opened
the same complete Git identity and exact committed blob. The unsaved draft,
editor frame, saved file, HEAD and index stayed unchanged. See
`target/apple-git-history-uikit-2026-11-19/`.

### Git conflict comparison and resolution

The existing Main Git pane lists an unmerged path once in Changes. Opening
that row gives the shared immutable Git stage reader. The real-engine replay
selected the literal `literal[1] λ%?#.py`, displayed the exact base, local,
and incoming bytes and object IDs, then retired the observation when another
file was selected. The conflict row withheld ordinary Stage and Discard.
The replay made only Git reads and preserved the saved working file and
index byte-for-byte. The focused pane tests also exercise an add/add
conflict and the whole-index action boundary. See
`target/main-git-conflict-v10/browser.json`.

Main now uses the complete-group composer in the existing Git pane. The
rendered text replay applied the chosen source as a private draft, undid
and redid it in the original editor, then prepared a fresh guarded Save.
Draft history preserved the saved file and index. Save published only the
chosen resolution with the index still byte-identical. Separate Stage
resolved only the selected literal path. A second real conflict kept its
exact stage entries and source bytes. See
`target/main-complete-conflict-v10-isolated-final/browser.json`.

The rendered rename replay acquired the unopened member in the background
and showed the explicit three-member group and chosen stage before Save.
The reviewed Save retargeted the original live editor to
`resolved λ %?#.py` with the same text, selection, and viewport. It left
the unresolved index byte-identical. Separate Stage resolved the complete
group and preserved an unrelated file's distinct index and working bytes.
See `target/main-complete-conflict-rename-v10-isolated/browser.json`.

The shared editor hold also passed the ordinary managed Move replay through
Save, reviewed Undo, Redo, and a subsequent edit. The same editor and
selection stayed mounted. Retired room sockets closed, and the later edit
updated only the final room and saved path. See
`target/main-managed-resource-command-v10-common-extraction/browser.json`.

The text fixture's generic Git Changes list previously exposed untracked
publication recovery records when the repository had no ignore rule.
The generic Git pane reserves `.hick-cache` in Changes, Diff, Stage,
Discard and Stash Push. Commit and Stash Push refuse externally staged
private additions or modifications without mutating the index. The
focused repository test compares exact staged and stashed paths, checks
a nested recovery directory during folder discard, and verifies that a
diff request cannot read a recovery record through a symlinked folder.
See the [Git pane contract](../guarantees/collaboration/the-git-pane-does-the-daily-loop/contract.md#f-900059e12d12-stage).

The Simple Browser complete-group composer now binds approval to the row
that opened it. A live Git replay prepared a review, selected an unrelated
file, and saw neither the composition nor its Save action under that row.
Returning restored the typed resolution and required a fresh preview. The
working file and index stayed exact. A dispatched request keeps its original
status control across a selection change or repository refresh. See
`target/simple-git-conflict-selection-v10/browser.json`.

The current rendered Android replay opened the literal-path Git conflict
and selected Stage 1. The visible Git object and `base` bytes matched
the repository. A retry installed the current app and rendered Files,
then the owned emulator showed the Android system-UI ANR dialog over it.
That attempt ended before Stages 2 and 3, guarded Save, or separate Stage.
See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-xk4yntp6/git-conflict/`
and `target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-i2e_rlbo/git-conflict/last-screen.png`.

The later rendered Android replay compared the exact mode, object ID, and
bytes of all three stages. Preview and guarded Save preserved the original
index byte-for-byte and kept the conflict unresolved there. Separate Stage
resolved only the literal conflicted path to the selected incoming bytes.
The unrelated file and its index entry stayed exact. Reopening source showed
the reviewed bytes in the same editor frame. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-s6d8t3vp/git-conflict/rendered-conflict-result.json`.

The iPhone simulator opened the same literal conflict through its Staged
detail and displayed the exact mode, object ID, and text of all three Git
stages. Native review and Save preserved the index byte-for-byte. Separate
Stage resolved the selected path to the exact incoming blob. The unrelated
staged and working bytes, HEAD, and the source editor frame were preserved.
The Working tree control did not open the detail in either attempt. A
follow-up tested five points within its displayed bounds. The upper-right
tap opened the exact conflict and returned HTTP 200 from the Working tree
diff route. Four other points, including the center, opened no detail.
Source and index bytes stayed exact. The center-hit candidate built.
XCTest stopped while loading Accessibility before the test case started.
On a fresh simulator, Xcode reached its 410-second startup timeout before
the test runner registered. A standalone replay reused a settled simulator,
verified the installed candidate, and opened the real repository sheet.
The simulator acknowledged a touch pair at the measured button center.
Hickory sent no diff request in the observation window. A subsequent
replay verified delivery in the same foreground UIKit window. Refresh
issued a second Changes request. The center received both touch phases
without issuing a diff request. The upper-right tap issued the exact
Working tree diff request and returned HTTP 200. Source and index bytes
stayed exact. The style change did not repair center activation. See
`target/apple-git-working-tree-uikit-v10-window-observed-handoff/receipt.json`.

The rebuilt app gives both side actions native UIKit buttons in the original
compact row. The simulator delivered both exact center taps to the named
buttons. Working tree and Staged each displayed their matching HTTP 200
diff. The selected row and source editor kept their original frames. The
editor selection, saved source, and index stayed exact. System spacing
placed the second file row four and one-third points lower. Forcing the
old gap reproduced the missed center tap. See
`target/apple-git-working-tree-uikit-native-buttons-compact/receipt.json`
and `target/apple-git-working-tree-uikit-native-buttons-compact-both-rows/receipt.json`.

### Files gutter overflow

The Main browser Files tree now keeps each recipe rail inside the row's
reserved column. A mounted two-folder replay opened a five-action grid,
kept the next folder and its action at the same coordinates, and clicked
an action on the grid's second row. Escape focused the original trigger.
The exact terminal request addressed the first folder. The full app replay
passed in Chromium. The isolated Files tree also passed in WebKit and Firefox. See the
[Files browser witness](../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md#files-recipe-overflow-browser).

### Native editor gutter overflow

The mounted Mac app opened the two-action grid with app-local AppKit pointer
events after both adjacent equation lines reached their final layout. Both
actions on the following line kept their exact frames. Selecting Equation
from the grid opened its inspector. The main editor, viewport, nonempty
selection, draft text, Undo manager, and saved bytes stayed intact. The
replay used the owned app's event loop while the console was locked.
A follow-up verified the visible three-dot trigger in the same 22 × 18
point slot and repeated the native click sequence. See
`target/appkit-gutter-overflow.0mp10_c3/receipt.json`.

A mounted AppKit replay used real pytest-cov results for two adjacent
declarations. Both tests passed and both executable lines were covered.
App-local mouse events opened each excess action in the two-icon grid and
reached its retained coverage report or fresh source-test inspection. Both
following controls kept their frames. The editor, selection, scroll position
and Undo manager survived, and an undoable draft edit retired the coverage
marks while retaining the report and saved bytes. Coverage was requested
through the engine and selected through workspace state. See
`target/appkit-shared-test-coverage.bg_4cxwz/receipt.json`.

The iPhone simulator exercised adjacent occupied rows and a second
overflow button in a lower slot. The baseline used a centered sheet and
anchored the lower case one slot above its trigger. The rebuilt app opened
both grids as compact popovers attached to the exact trigger bounds. Both
replays kept the next action, editor frame, scroll, selection, undo history,
and saved source intact while opening the grid. Native taps on an excess
action revealed its exact equation source. See
`target/ios-gutter-overflow-native-replay-settled/{baseline,candidate}/receipt.json`.

The final iPhone build drew a visible ellipsis inside each of three
unchanged 22 × 18 point slots. Native taps opened fully visible grids and
selected excess actions from both adjacent rows and the lower slot. The
trigger anchors, following controls, editor state, and saved bytes stayed
exact. See `target/ios-gutter-overflow-native-visible-trigger/visible-trigger/receipt.json`.

A real pytest-cov run added coverage and source tests to two adjacent iPhone
rows. The first row's compact grid opened at its reserved slot and kept
the next row's two controls in place. Native taps dispatched both first-row
actions, but the grid and the requested workspace panel competed for
presentation. Both actions reached their views from the inline second row.
The iOS grid now delivers its selected reading after dismissal, while the
same source address and edit revision still match. An AppKit comparison
found that its grid can dismiss without the SwiftUI disappearance callback,
so AppKit delivers the action immediately. The final AppKit replay opened
both readings with unchanged following controls and editor state. See
`target/ios-test-coverage-native/iphone-27-selected-run/attempt-2/`,
`target/appkit-deferred-test-coverage.11idmnef/appkit.log` and
`target/appkit-platform-test-coverage.hwjv_88d/receipt.json`.

The final iPhone replay selected a real pytest-cov run through the fixture.
Native taps reopened its report with both measured rows and refreshed only
the first declaration in source tests. Both actions used the first row's
overflow. The editor, selection, scroll, Undo manager and following controls
held their positions. A native unsaved edit retired both coverage marks
while the saved bytes and file identity stayed intact. See
`target/ios-test-coverage-native/iphone-27-handoff/attempt-2/receipt.json`.

On Android 37, a native tap opened the recorded-fence Run and Debug grid
directly above an occupied breakpoint line. The following breakpoint and
source editor kept their exact positions, identities, scroll and selection.
Selecting Run dispatched the addressed user fence and displayed `42` with
exit code 0. Dismissal kept the transcript and saved source byte-exact.
The stock APK showed the same grid with identical editor pixels. A separate
build logged passive Compose coordinates beneath the open popup. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-ly84u3ir/gutter-overflow/rendered-android-gutter-overflow.json`.

A separate Android 37 run collected two passing pytest cases and measured
three authored lines through the embedded panel. Native taps at the measured
Compose centers selected the first line's coverage and test actions from
overflow. Coverage reopened the retained run and source tests showed only its
first declaration. The editor frame, identity, scroll, selection and next
control stayed fixed with the grid open. The fixture used the settled Gboard
viewport and passive measurements. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-ff6ifu4x/coverage-overflow/rendered-android-coverage-overflow.json`.

The dirty-source replay preselected a real pytest-cov run through the
fixture. Native Select All and Delete made the buffer an empty draft.
Coverage stayed absent after fresh source-test indexing, and the engine
received no source write. Saved source and project bytes matched their
initial digests. This draft check used fixture selection after the
automation client reported an opening screen. The prior receipt supplies
the rendered collector and report-navigation evidence. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-zqxs3v3w/coverage-overflow/rendered-android-coverage-overflow.json`.

In the focused cold-process probe, the coverage WebView mounted all 12
buttons and received HTTP 200 for its initial list request. The same-frame
screenshot showed those controls while UiAutomation still reported the
opening placeholder. Clearing only the UiAutomation client cache exposed
the controls on the next ordinary read, with byte-identical screenshots
before and after. Earlier runs reported as stalls also show the controls in
their captured pixels. The first observer APK omitted its generated JS and
CSS, and its assets were restored before the later probes. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-riuq1s_4/coverage-overflow/cold-process-1/outcome.json`.

### Requested files and capability arrival

The shared Simple launcher now passes the exact workspace-relative requested
file to each host. The rendered browser witness opened a nested Markdown file
beside only its containing folder, then held and released the actual parser and
saved renderer. It exercised stale and current Python test discovery, readable
protected output after parser failure, and successful and failed CSV grid
arrival. Editor identity, selection, scroll, Undo and draft bytes survived those
replays. This receipt used the tree-first path. The independent startup text
and retry receipts below cover held and failed Files responses.

Android's loopback startup witness passed one focused invocation. On the
rendered API 36 host, the requested nested Markdown and its containing-only
Files view were readable while the actual engine capabilities frame was held
after WebSocket connection. The accessibility text field returned `-1/-1` for
selection in that fixture, preventing its edit and viewport replay. A later
rendered replay verified them through native selection and Cut. Mounted AppKit
and UIKit each passed one focused startup
replay with zero skips. The provider's current diagnostic arrived after the
edit and its stale diagnostic was discarded. The replays kept the requested
file beside its containing tree and preserved editor identity, selection,
scroll, draft, Undo and Redo with unchanged disk bytes. UIKit sampled geometry
after the software keyboard settled. These replays use the tree-first path.
Receipts are `target/apple-startup-appkit-final-2026-11-18/appkit-startup-receipt.json`
and `target/apple-startup-ios-gated-2026-11-18/startup-receipt.json`.

The independent startup-source path mounts a read-only editor before Files
and guarded source authority. Mounted AppKit and UIKit each passed held-tree
confirmation and unavailable-tree replays against controlled endpoint
responses. The original editor and its selection, source anchor and frame
survived confirmation. The unavailable branch retained readable source without
edit authority. Chromium passed the same ownership transition with controlled
receipts.

An iPhone replay opened the requested Python file and enabled edits after
Files arrived, but source-test markers never appeared. The mounted editor
had already observed the unchanged source bytes before the workspace
attached its readers. Confirmation now explicitly asks the existing
test index and coverage reader to observe the retained buffer. A mounted
AppKit comparison held the real Files response while the raw Python source
was readable. The baseline sent no source-test query. The rebuilt app sent
one query after confirmation and indexed both adjacent declarations. The
same editor, selection, scroll, Undo manager and saved source survived.
The rebuilt iPhone app showed both test actions on first open, before any
edit. The maintained AppKit witness passed staged confirmation, unavailable
Files, retry and Files-first guard retry. See
`target/apple-requested-test-index-witness/{baseline,candidate}.json` and
`target/appkit-startup-index-xcode-20261119-172349/`.

Android's loopback controller passed independent raw, Files and
guarded-source gates, including mismatched bytes and exact same-editor
promotion. Rendered Android reached raw text before Files at nonzero scroll,
then its controlled host replay timed out before the guarded document read.
The freshly linked engine then passed rendered Android early source and
promotion with separate holds for Files, guarded source and the capability
message. It retained the editor frame, viewport and saved bytes. The receipt
is in `target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-fg7l56um/startup-raw/`.
A second real rendered replay inserted an exact draft before the held
capability arrived. The original frame and nonzero scroll position survived
release and a fresh semantic query. Native Cut removed only the selected
draft character. Two ordinary Undo actions restored the draft and then
the original source. The replay kept saved bytes unchanged and sent no
resource save. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-vfz5deqg/startup-raw/`.
Hickory Docs' desktop launcher attaches the containing folder and keeps the
exact requested tab. The main Chromium replay passed against the rebuilt engine
with ordinary raw text visible before explicit Open File and the guarded read.
The same editor gained write access, kept the visible Files tree and draft
through layout restoration, and undid the draft. The raw observation stayed
readable and read-only until authority arrived. A mounted CodeMirror witness
also retained a nonempty selection through guarded promotion and checked that
later writes carried the original authority. The focused Simple launcher test
passed exact path encoding and workspace confinement.
The main Chromium window also passed a real parser delay while the ordinary
file gained write access and retained the same editor, draft, selection and
Undo. Its managed startup replay displayed raw source through separate holds
for explicit Open File, guarded source, first collaboration sync and the real
parser. The same editor kept its nonempty selection and deep viewport, sent
no source update on promotion, and edited and undid after confirmation. A
focused mounted test delivered a collaborator edit as the room attached.
Managed documents with CRLF or mixed line endings stay readable and
read-only when the room's exact offsets cannot be represented by the editor.
The focused mixed-ending fixture kept the exact room bytes and refused a user
edit. The desktop dependency lock now resolves the current engine graph.
Two native Finder projection checks passed for a punctuated UTF-8 file and a
directory. The signed package also passed an actual Finder file launch. Its
native WebKit editor showed the exact bytes of `entry # %.md` beside only the
containing `nested` folder. The tree included its sibling and excluded the
outside file. The Documents-hosted fixture stalled during directory discovery.
The successful isolated replay used an owned temporary folder. See
`target/main-macos-packaged-startup.to8jhtpo/packaged-finder-receipt/`.
In a second signed-package replay, WebKit held the actual parser response
while Files and explicit Open File stayed live. The requested editor displayed
the exact bytes read-only and accepted a nonempty selection. After the parser
response returned HTTP 200, the same editor kept those bytes and selection,
became editable, and showed only the containing folder. The saved fixture
bytes were unchanged. See
`target/main-macos-packaged-startup.to8jhtpo/packaged-finder-wasm-receipt/`.
Holding all three replies together exposed the requested editor group before
the observation bridge timed out. The combined-delay viewport and Undo
receipt is the Chromium replay above.

The signed package also passed a sequential authority replay with Files live.
The relay held explicit Open File, its guarded read, the first collaboration
sync and the parser independently. The same native WebKit editor kept its
exact text and nonempty deep selection across all four releases. It emitted
no source update on promotion. An edit and Undo then emitted two ordinary
collaboration updates, and the saved fixture bytes matched after shutdown.
The native bridge confirmed only the containing folder. This replay checked
deep selection with the editor's scroll-into-view action. The viewport offset
and visible row still lack a direct packaged witness. See
`target/main-macos-packaged-startup.to8jhtpo/packaged-sequential-continuation.vTe50p/`.
The follow-up used a same-origin, read-only fixture observer with the signed
application and existing CSP intact. WebKit reported its page hidden and
unfocused. It delivered no animation frames during the sample. The editor
reported scroll offset zero, and the selected row lay outside the clipped
viewport. AppKit also declined the activation request for the exact owned
process. That attempt left the rendered four-gate viewport unverified.
See `target/main-macos-packaged-startup.to8jhtpo/packaged-viewport-layout.Gss7MS/`
and `target/main-macos-packaged-startup.to8jhtpo/packaged-viewport-activated.HIW2pd/`.
The next preflight confirmed that the macOS console session was locked. A
foreground signed-app replay requires an unlocked console session.

The rebuilt signed Main bundle passed the four-gate viewport replay on the
unlocked console. WebKit kept the same editor, scroll owner, nonempty
selection, offset of 4,765 CSS pixels, and first visible source row 108 while
Open File, the guarded read, collaboration sync, and parser loaded separately.
Files showed only the containing folder. Promotion emitted no source updates.
Edit and Undo emitted two ordinary collaboration updates and restored the
exact saved bytes. The build wrapper reached its disk cutoff after signing.
The completed bundle passed independent strict signature verification before
launch. See
`target/current-signed-main-9f764d14/package-receipt.json` and
`target/current-signed-main-9f764d14/packaged-viewport-unlocked-1795115620844905000/receipt.json`.

The current AppKit minimap passed a queued mouse drag with its owned
window active and key. Four intermediate viewport offsets from 1,355.5
through 3,979.5 points exposed source line 266. The same editor, frame,
selection, draft and undo manager stayed mounted. Native Undo and Redo
restored the exact text and left the file's bytes, inode and modification
time intact. Earlier inactive-window probes left the viewport at zero.
See `target/appkit-minimap-baseline.8w3fmrv7/receipt.json`.

AppKit and UIKit each passed the transient Files failure and same-server retry.
Both kept their provisional editor, selection and viewport through confirmation.
UIKit also performed native Paste and Undo after the guarded source arrived.
Mounted Chromium passed the same recovery through File → Refresh with
controlled responses and again with the freshly linked engine. It kept the
original editor, selection and viewport through guarded editing and Undo.
The Apple receipts are in `target/apple-startup-authority-appkit-retry-2026-11-18/`,
`target/apple-startup-authority-ios-isolated-2026-11-18/` and
`target/apple-startup-authority-ios-retry-2026-11-18/`. The Chromium receipt is in
`target/browser-startup-retry-controlled-7fqqvcuh/`. The real engine browser
receipt is in `target/recorded-engine-browser.CwVMo5/early-source.log`.

Mounted AppKit and UIKit also passed the reverse response order. Files
showed the requested folder while the guarded source read was pending.
Late raw text mounted immediately and stayed read-only when the guarded
reply lacked a save guard. Refresh Files confirmed the same native
editor, selection, and scroll position. Native edits, Undo, and Redo
preserved the expected draft bytes and left the file on disk unchanged.
See
`target/apple-startup-authority-tree-first-appkit-2026-11-19/tree-first-retry-receipt.json`
and
`target/apple-startup-authority-tree-first-uikit-retry-2026-11-19/startup-authority-tree-first-retry-receipt.json`.

Android's focused startup tests also cover Files arriving first. A guarded
503 leaves the independent raw read alive. When guarded text arrives
without a save guard, it opens read-only and the late raw response cannot
replace it. Refresh Files confirms permission in the same retained text
field with its selection and scroll position. A native-model draft then
round-trips through Undo and Redo with zero source publications. See
`target/android-tree-first-startup-focused-retry.log`.
Rendered Android also exercised the guarded 503 through a relay serving the
real Files, raw, and confirmed source replies. The requested folder appeared
while text was pending. Refresh kept the native editor frame and viewport
when late raw text became editable. A native edit, Undo, and Redo preserved
the expected drafts and left saved bytes unchanged. Android accessibility
provided no numeric selection offset. The focused model test above checks
exact nonempty selection retention. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-7_i4auql/startup-tree-first/tree-first-result.json`.

Simple Browser retains a read-only editor when the first guarded reply
already has text and lacks a save guard. A later raw reply leaves that
editor and its pending confirmation in place. Focused Chromium with
controlled API replies exercised both this order and a guarded 503
followed by late raw text. Refresh Files confirmed permission in the
same CodeMirror editor and preserved its nonempty selection, scroll,
and Undo and Redo history. Typing and native editor Undo and Redo produced
the expected text with zero source-history writes. See
`target/simple-startup-guardless-fixed.json` and
`target/simple-startup-guardless-vitest.log`.

With the current engine, Simple Browser opened the requested managed file
read-only while Files and the parser were held. Files release displayed
only the containing folder. The separate guarded read retained the same
editor, selection and first visible row. Typing after confirmation created
a draft before parser release. Parser attachment kept the draft, editor,
selection and viewport. Keyboard Undo restored the exact original text.
The replay sent no source writes. See
`target/simple-startup-authority-v10-owner-held-retry/browser.json`.

Main now also keeps its independent raw read after Open File refuses the
initial request. In rendered Chromium, a relayed 503 arrived before the
exact raw text for both a plain file and a managed document. The late text
mounted read-only. Retry source access used real engine replies and
confirmed the same editor with its nonempty selection and deep viewport
intact. The saved fixture bytes stayed unchanged. See
`target/main-startup-late-raw-refusal-v8-fixed-retry/browser.json` and
`target/main-startup-late-raw-refusal-v8-managed-retry/browser.json`.

Main presents the first matching startup text from the raw or scoped
read. Open File can succeed while the guarded reply is held. Rendered
plain and managed cases mounted the available raw text read-only and
kept the same editor, nonempty selection and first visible row through
guarded promotion. The managed case also held the real room sync.
Edit and Undo restored the exact bytes after promotion. In reverse
order, both editors accepted a draft before the raw reply completed.
The later reply preserved the draft, editor, selection, scroll, and
Undo history. See
`target/main-startup-authority-race-fixed-plain-raw-first-retry/browser.json`,
`target/main-startup-authority-race-fixed-managed-raw-first-retry/browser.json`,
`target/main-startup-authority-race-fixed-plain-guarded-first/browser.json`,
and `target/main-startup-authority-race-fixed-managed-guarded-first/browser.json`.
A guarded reply without a save guard also stayed read-only for both
kinds. Retry used fresh authority and kept the same editor, selection,
and visible row. The scoped read was reused for admission. A managed
document rechecked authority when its room became available. See
`target/main-startup-authority-race-fixed-plain-guardless-retry/browser.json`
and `target/main-startup-authority-race-fixed-managed-guardless-retry/browser.json`.

Open File now identifies the workspace for a contained selection. This
lets Main show exact text from its scoped read while Files and raw
startup source are both pending. With the real engine, plain and managed
editors rendered read-only and refused typing during those holds. Files
release preserved the same editor, nonempty selection, scroll position,
and first visible row. The managed editor then stayed read-only
through its held room sync. Promotion sent no source writes or room
updates. A later raw reply preserved an edit, and Undo restored the
exact bytes. See
`target/main-startup-authority-race-v10-plain-listing-held-retry/browser.json`
and `target/main-startup-authority-race-v10-managed-listing-held-retry/browser.json`.
An explicit selection outside the workspace retained its absolute path
and did not acquire the workspace scope. See `target/open-file-startup-scope-v10.json`.

Mounted AppKit repeated both cases against the freshly linked engine. A held
Files response let the exact startup bytes appear first. Releasing Files and
the guarded read kept the original editor, its selection and viewport, and
opened only the containing folder. Native editing and Undo left the file on
disk unchanged. A separate live replay returned a transient 503 for Files,
retained the readable source, then confirmed authority after retrying against
the same engine. See `target/apple-real-startup-confirmed.h738j71s/receipt.json`
and `target/apple-real-startup-retry.c5i2phsf/receipt.json`.

Mounted UIKit also passed both cases against the current real engine. The held
Files response left the requested text readable with its selection and viewport
intact. After immediate confirmation or a temporary 503 and retry, the same
native editor adopted the exact guard for those bytes in the same workspace.
Native Paste entered a draft, Undo restored the original text, and the saved
files stayed byte-identical. The disposable simulator was deleted. See
`target/apple-startup-uikit-live.fVw2t7/`.

### Rendered Android Node.js Watch and Continue

An owned API 36 emulator ran the genuine js-debug adapter against saved Node.js
source. The exact saved hash and requested breakpoint on line 2 were confirmed.
The renderer showed `value` as `41` after an explicit Watch in stop 1, thread 0,
frame 0. Continue produced `42` and retained Finished with exit code unavailable
from the adapter. Saved bytes were unchanged. This closes the Node.js Watch and
Continue gap for that deliberately selected scope in the Debugging row above.
The wire and renderer receipts are in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-2ecyyg_2/node-watch/`.

### Recorded Agent message fences

The recorded-fence admission reads the exact stored turn and message side,
checks its SHA-256 revision, and captures one complete literal fence. Engine
preparation is memory-only. Explicit Debug uses a scratch program and maps
stops back to the message. The browser component witness passed for both launch
affordances, explicit admission, programmatic edit refusal and retiring a late
capture after its editor closed. AppKit and Android controller witnesses passed
exact side and revision admission, selected-fence breakpoint filtering and
mapped frame handling using controlled transports. The main Hickory Docs Agent
pane's browser witness also passed both fence line anchors within its session
envelope, separate immutable message editors, and retained composer draft.
A focused main-window replay now seeds both recorded sides and checks the exact
admission address, mapped Step, Watch, Continue and preserved conversation. Its
fixture hydrates both exact sides. A mounted replay found that conversation CSS
hid the shared rail. Both browser variants now reserve its width with the
initial message, and the main window replay reached the exact recorded-fence
request. The fresh engine replay passed Step, Watch and Continue for both
message sides in the main window. Mounted AppKit also passed both, including
exact authored stops, sibling exclusion and unchanged recorded bytes. Its
receipt is in `target/apple-recorded-debug.ndogv_cd/receipt.json`. Simple
Browser passed the real debugpy replay for both sides after its stylesheet
stopped hiding the gutter overflow button. The replay used that button to
reach Watch. Rendered Android reached a real mapped user stop on the second
executable line. Its first action was visible in pixels but missing from the
accessibility tree under expanded neighboring hit regions. The Android rail now
uses measured hit bounds and includes recorded body and patch actions when
measuring the next occupied row. Its exact user replay now exposes separate
accessible rows for launch and both breakpoints, stops on the first executable
line, steps to the next, Watches `value` as `41`, and Continues to `42` with
the recorded and saved files unchanged. The receipt is in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-la8wooor/agent-user/`.
An assistant replay used a separate session with a shorter preceding user
message and the identical assistant message bytes. It stopped on the first
executable line, stepped to the next, Watched `value` as `52`, and Continued to
`53` with saved files unchanged. The receipt is in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-dbsv87hb/agent-assistant/`.
Mounted UIKit
passed both recorded sides with exact first authored stops, Step, Watch and
Continue. The receipt is in
`target/apple-recorded-ios-final-2026-11-18/recorded-debug-receipt.json`.
On the iPhone simulator, native line-number taps set breakpoints on both
recorded sides. The Debug and Step over gutter actions reached authored
lines 4 and 5 through real debugpy. Inspect opened on line 5. Tapping the
Watch expression failed to produce a keyboard, so the native replay could
not enter an expression or continue from the inspector. See the wire and
simulator attachments in `target/ios-recorded-debug-ui-v9-first/` and
`target/ios-recorded-debug-ui-v9-typed-retry/`.
The iPhone Agent pane now owns the inspector above the scrolling messages.
The rendered replay set a breakpoint, paused on authored line 4, and stepped to
line 5 on each recorded side. Tapping Expression kept the inspector open
with the software keyboard. Watch showed `42` for the user message and `53`
for the assistant message. Continue finished both debugpy sessions with exit
0. Dismissal restored the exact recorded text and editor frame. The wire
confirmed the selected side and scoped Watch, and the saved archive stayed
byte-identical. See `target/ios-recorded-debug-ui-v9-stable-sheet/`.
Mounted AppKit used native Debug, Step over and Continue gutter clicks for
both recorded sides with the current engine and debugpy. Each session paused
on authored line 4, stepped to line 5 and printed its expected `42` or `53`.
The start request contained only the selected fence's breakpoint. Both
recorded editors kept their frame and selection, and archived bytes stayed
exact. The locked console did not dispatch the native Inspect action.
Breakpoint setup and Watch used the production controller, so native input
for those actions still needs a separate witness. See
`target/appkit-recorded-debug-rail.9ef6tf33/receipt.json`.
After extracting the shared inspector, a focused current AppKit build and
user-side native replay again passed mapped Debug, Step over and Continue.
See `target/appkit-shared-inspector-compat.lhxb4dyq/receipt.json`.
A first-line gutter click on the locked console subsequently opened the
actual user-fence inspector. Native key events entered `value + 1` in its
focused Expression field. The non-key popover did not dispatch Watch via
mouse or local accessibility activation, so the rendered value still needs
foreground verification. See
`target/appkit-recorded-inspect-watch.kqo0rebk/receipt.json`.
The unlocked AppKit replay queued clicks to the owned native window and
completed Watch on both records. The user expression `value + 1` returned
`42` and the assistant expression `1 + value` returned `53`. Both evaluations
used the current mapped stop and frame, and native Continue finished the
same selected fence with its archive and editor intact. The controller
supplied the initial breakpoint in this replay. See
`target/appkit-recorded-inspect-watch.4hgq2f4x/receipt.json`.
An unlocked follow-up queued the click over the visible user line-number
glyph in the native gutter. It produced no pending marker and stopped before
Debug Start. The assistant line-number interaction was outside that run.
That attempt left native marker placement unverified. See
`target/appkit-recorded-native-breakpoint.1t_v6fnx/appkit.log`.
Executable line numbers now use native buttons aligned to their measured
text rows. The rebuilt AppKit replay placed two pending markers on each
recorded side before launch. Each Debug start admitted only the selected
fence's breakpoint and exact recorded hash. Native Step over and Continue
reached the next authored line and produced `42` or `53`. The archive,
editor identity, selection, and frame stayed exact. See
`target/appkit-recorded-native-breakpoint.vo3cw2jj/receipt.json`.
The rebuilt iPhone simulator used the same executable-line button on both
records. Each line 4 action moved from `No breakpoint` to `pending` and
the fence opener offered no breakpoint action. The exact single marker
was present on Debug Start. Both sessions stepped through authored lines
4 and 5, returned `42` or `53` through Watch, and continued to exit 0.
Each recorded editor kept its text and frame. The maintained simulator
workflow now checks the named action and pending state. See
`target/ios-recorded-debug-buttons-v10-36716d8c-current/`.
The rebuilt engine also rejected stale message hashes on both sides, withheld
execution after preparation, and refused escaped or non-UTF-8 startup reads.
The main Chromium window and Simple Browser also ran both selected Python
fences through the ordinary terminal runner. The user fence printed `42`, the
assistant fence printed `53`, and both returned process exit 0 with their
recorded bytes unchanged. No sibling fence ran. The main replay also refused
stale addresses without starting a terminal, then repeated mapped Debug on
both sides. A direct engine replay compiled and ran the selected C fence and
interpreted the selected JavaScript fence. It retained the same recorded
bytes and the workspace received no program files. See
`target/recorded-run-engine.T0OdmJ/compiled-and-interpreted-receipt.json`.
A fresh main Chromium replay repeated Run and mapped Debug on both recorded
sides. It rejected the wrong message hash without adding a terminal, stepped
from the original mapped breakpoint, evaluated Watch and continued to exit.
The original message editor, composer draft and archive bytes stayed intact.
See `target/main-recorded-debug-both-sides.fQQtyu/replay.log`.
Rendered Android passed both sides with their exact opening-line requests.
Opening the gutter menu started no process. Explicit Run returned `42` and
`53` with process exit 0 in the native floating terminal. The editor frame,
recorded bytes and saved files survived both runs. The assistant replay used
the exact assistant bytes with the earlier user message shortened to fit the
fixture. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-1u31agr8/`.
Mounted AppKit passed both openings through ordinary native gutter clicks.
Its actual terminal viewport displayed `42` and `53`, with process exit 0
for both. The mounted editors retained their identities, frames and selections.
The Composer draft and archived bytes were unchanged. See
`target/apple-recorded-run-appkit.qOn7w9/`. A separate UIKit UI test reached both
SwiftUI gutter controls on a disposable iPhone simulator. Its terminals showed
`42` and `53` with exit 0, including a screenshot for each. The recorded
editor texts and frames survived dismissal, and the archived bytes stayed
unchanged. See `target/apple-recorded-run-uikit.pfP9mH/`.

Patch review admission discovers canonical ordinary unified-diff targets,
derives ordered edits from guarded preimages and checks the entire proposed
transition before sealing. The proposal publisher still owns Apply. The
canonical Rust parser, exact transition comparison and rename planner passed
focused isolated witnesses. Browser, AppKit and Android controller checks
passed the sealed review entry for recorded ordinary fences. Real engine
preparation, generated consequences, Apply and Undo passed for the user-side
`diff` in Simple Browser and AppKit. Both replays exposed that the archived
assistant `patch` fence arrived without delimiters. Hydration now recovers
the original fence span for both sides. The exact unwrapped fixture passed
both sides, including guarded Apply, Undo and unchanged saved bytes, in
Simple Browser and AppKit. The receipts are in
`target/recorded-engine-patch-rerun.wI4g1f/replay.log` and
`target/apple-recorded-patch.ofyzmnpr/receipt.json`. Mounted UIKit passed those
same checks against the exact unwrapped fixture in
`target/apple-recorded-ios-final-2026-11-18/recorded-patch-receipt.json`.
The iPhone simulator opened the ordinary patch review from both recorded
gutter controls. Each review displayed its source digest, complete seal, and
generated consequence. Native Apply changed the selected draft in a separate
source editor. Edit → Undo restored that exact draft, and each recorded card
retained its immutable text. The engine confirmed both guarded applications.
All five saved fixture files retained their bytes and metadata. See
`target/ios-recorded-patch-native-v10-retry-2026-11-19/receipt.json`.
Mounted AppKit used native gutter clicks to open the rendered guarded patch
review for both recorded sides against the current engine. Each request bound
the selected side, message hash and opening line. Both reviews displayed the
generated consequence. The same recorded editor, selection and frame survived
closing the review. Controller Apply and the workspace Undo command replayed
the exact draft changes with the saved and archived bytes unchanged. The locked
macOS console left the review popover without keyboard focus. Native Close
worked. Click and accessibility activation left Apply undispatched. See
`target/appkit-recorded-gutter-patch.4xypqkm6/receipt.json`.
The follow-up scrolled the reviewer and placed Apply inside the visible
popover. Native activation still sent no validation or application request
on the locked console. See
`target/appkit-recorded-apply-focus-patch.cnv_y0zf/receipt.json`.
After the console unlocked, the rendered AppKit gutter and Apply clicks
passed for both unwrapped records. Each scrolled reviewer dispatched its
exact captured path and side through validation and the application report.
Native Close and the workspace Undo command restored both drafts. The
saved files and recorded messages retained their exact bytes. See
`target/appkit-recorded-apply-focus-patch.7z4r4n62/receipt.json`.
Rendered Android passed the user-side review, generated consequence, Apply and
Undo. Its compact assistant fixture passed the exact recorded `patch` review
and draft Apply with unchanged physical files. The assistant body matches the
original fixture byte for byte. The receipt is in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-dbsv87hb/patch-assistant/`.
Rendered Android also reviewed a separate assistant patch that changed a
managed author and a plain file together. The review showed the generated
Before and After. Apply, Undo, and Redo each changed both drafts together
while the saved sources, generated file, and archived message stayed exact.
Explicit Save All published each draft through its own source guard. The
generated file and archive remained unchanged. See the history and Save All
receipts in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-yzrqs76i/patch-assistant-history/`.
The main Hickory Docs Agent pane passed live sealed reviews for both sides
with its mounted file's original guard, generated consequences, retained
composer and unchanged source files. Closing the dialog returned focus to
the original gutter button without replacing the file editor. Focused
checks also cover capture revocation and receipt guard replacement. The
ordinary plain-file editor queues a disk write after a local edit, and its
managed editor sends local edits to the shared room
that persists source. Reviewed Draft Apply now detaches both edit paths from
those publishers while retaining their original saved guards. In separate live
main-window replays, the recorded user patch and the recorded assistant patch
each changed a plain file and a managed document together. Both preserved the
same two editors through coordinated Undo and Redo, with no source PUT or
collaboration write. Saved and generated files and the conversation stayed
byte-identical. After the focused Save completed, the plain draft stayed
private until Save All. Each publication used its own guard. Generated files
and the conversation stayed unchanged. See
`target/main-recorded-patch-both-sides.WYqm4K/{user,assistant}.log`.
The private recovery marker round-tripped in
the focused Rust storage test. The managed recovery test also refused a
same-byte physical replacement until a fresh explicit comparison.

Declared project patch fences need the compiler's exact owning document and
operation, the captured source closure and the predecessor tree. The engine
now binds a uniquely matching recorded document to the exact compiler opening,
checks that the closure reproduces the complete retained tree, then returns
the selected operation's local Before and After. The focused Rust cases
verified the local result even when a later operation and another document
changed outputs. They refused stale, ambiguous, absent, bare and empty
captures. The browser component displayed the bounded receipt in the original
gutter. The main Chromium window and Simple Browser passed the live replay
against a persisted conversation whose framed assistant message exactly matched
its captured document. Both showed `old` and `middle` for the selected output. The later
`final` result stayed outside the comparison. The recording and working file
were unchanged. The declared inspection is read-only. Focused AppKit and
Android controller tests bound exact source and workspace identities and
rejected foreign receipts. On the iPhone simulator a real gutter tap opened the
selected output comparison. The reading showed `old` and `middle`, preserved
the recorded editor text and frame, and supplied no Apply, Save or Run action.
The archive and working file stayed byte-identical. See
`target/apple-declared-uikit-final.MMaOz1/` for the replay with the final Apple
comparison view. Mounted AppKit passed both native clicks. The floating comparison showed
Before `old` and After `middle` in order, with no later result or write action.
The original editor, selection, frame, archive and working file stayed intact.
See `target/main-macos-packaged-startup.to8jhtpo/apple-declared-rendered-muueemph/`.
The current Android APK passed the same comparison on an owned API 36
emulator. The gutter requested the exact assistant opening and the native
reading showed Before `old` and After `middle`. Closing it kept the original
editor frame and exact text. The trace recorded no source write, and the
archive and working file stayed byte-identical. The owned emulator and its
temporary disk were removed. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-yn39rn6t/declared-assistant/`.
The owned Android emulator repeated the gutter reading for the archived user
message. The verified owner was `request.md`. The reading showed the same
Before `old` and After `middle`, and closing it left the card frame and bytes
intact. The saved fixture was unchanged and the emulator was removed. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-r9g2of58/declared-user/`.

Both browser frontends also inspected the user-side declaration when the
finalized response included the exact archived request as a contributing
document. Separate replays bound the assistant declaration to `response.md`
and the user declaration to `request.md`. Each returned the selected local
transition, supplied no write action, and left the archive and working file
byte-identical. See `target/recorded-construction-both-sides.1P3Iop/`.
The iPhone simulator repeated both cases by tapping each recorded card's
gutter. It showed the owning document and the selected `old` to `middle`
transition. Closing the reading preserved the same card text and frame.
The saved bytes stayed unchanged and the disposable simulator was deleted.
See `target/apple-recorded-construction-both-sides.5j8K0C/`.
Mounted AppKit repeated both cases with native clicks on the original gutter
and the comparison disclosure. The user reading identified `request.md` and
the assistant reading identified `response.md`. Each showed Before `old` and
After `middle` while preserving the original editor, selection, frame and
saved bytes. The exact engine receipts are in
`target/main-macos-packaged-startup.to8jhtpo/apple-declared-rendered-oedft7lt/`
and `target/main-macos-packaged-startup.to8jhtpo/apple-declared-rendered-09y_8o9x/`.

### Mounted iOS Paste, Cleanup and selected Rust Debug

An owned iPhone 18 Pro running iOS 27 mounted the actual editor against a fresh
engine. Native Paste replaced a Unicode selection, pasted at multiple carets
and appended text. Each action replayed through its editor-owned Undo and Redo.
Reviewed Cleanup also passed coordinated Undo and Redo. The selected saved Rust
test went from `start_test` to `started` in 6.83 seconds, stopped at its authored
line, continued and retained exit 0. Its sibling never ran. Editor identity,
selection, scroll, history, pixel geometry after keyboard settling and saved
bytes were checked. The XCTest result was one pass with zero skips, and the
four fixture safety cases passed separately. The owned simulator was deleted.

The original timeout was not reproduced with that fast fixture. A separate
fixture with a 32-second Cargo build step crossed the former deadline on a
fresh engine. The engine acknowledged the exact saved test at 5.77 seconds
and admitted the debugger at 43.10 seconds. The direct protocol then reached
one verified breakpoint, continued, and retained exit 0. A mismatched source
hash received only a refusal. The six ordinary physical and authored Rust
protocol cases also passed, including two inactive declarations refused before
any session. This isolates the valid preparation path that the former native
deadline could retire. Apple and Android now allow one bounded preparation wait
after an exact request, document and saved-selection receipt. Only the later
Started can create a retained session. Cancel and timeout avoid resubmission
and clean up a late owned session. The owned iOS 27 replay exercised the actual
UIKit editor with the slow fixture. Preparation arrived at 0.09 seconds and
Started arrived at 42.05 seconds. Paste and Cleanup Undo/Redo, the selected
breakpoint, Continue, retained exit 0 and original source and editor state
passed together in one XCTest with zero skips.
[Mounted iOS acceptance](../mechanisms/ios-mounted-parity-acceptance.md)

After the compact recovery and Paste replay fixes below, the same slow mounted
editing case passed again. Preparation arrived at 0.22 seconds and Started
at 40.98 seconds. The strict preparation parser was in this build. The ten
focused Apple source-test cases and nine Android Debug model cases passed
separately, including foreign, malformed and timed-out admission handling.

A separate mounted UIKit workspace test passed saved search into the exact
caret, Back and Forward, and clangd 21.0.0 Problems navigation with stale
location refusal. A source-adjacent pytest 9.2.0 Run reached its terminal
exit. The mounted WKWebView discovered the two intended tests, retained a
deliberate failure and pytest-cov 7.1.0 coverage, opened its exact source
location and retired the coverage marks when the source changed. The fixture
confirmed saved bytes and the executed test names. An independent UIKit test
reopened a sealed historical agent proposal after a verified engine restart.
It exposed the original explanation and material, withheld Apply and Save,
and preserved the live dirty editor, its selection and Undo. Each XCTest ran
once with zero skips.

The mounted private recovery replay read the connected behavior, observed its
source becoming stale and reached a selected process-only check. On compact
iOS, draft copies open as tabs in the existing code pane. Restoring and then
reopening the original preserved its widget, pane and layout identities, its
181.67-point width, its 90-point vertical offset and original selection. The
copy separately retained selection, scroll, draft and saved baseline without
write authority. Undo restored the original Paste's preimage and Redo restored
the dirty draft. The focused XCTest passed once with zero skips.

A live mounted managed-agent proposal reviewed exact `note.py` and
`construction.md` changes, the `generated.py` consequence and claims marked
unverified. Apply created one draft batch and one coordinated Undo and Redo
acted on both drafts. UIKit initially moved existing selections to the edits.
The corrected reflection maps the retained selections through the exact
replacements and restores the viewport. The replay kept the caret before an
edit, shifted the noncollapsed later selection by one UTF-16 unit on Apply
and Redo, and restored its original position on Undo. Both editor identities,
the nonzero viewport and all saved files were retained. The managed tool and
host report carried their own receipts. The focused XCTest passed once with
zero skips.

### Reviewed entry recovery and older Move lineage

The checkpoint's typed interruption scope carries its sealed original Move
through acquisition, Preview and explicit Save. A clean automatically opened
editor can receive the sealed dirty draft, and an existing dirty original
retains its current text. Saved publication follows the producer's physical
proof. The completion journal remains readable, and the partial namespace it
repaired is excluded from ordinary workspace Undo.

An ordinary older Move can be reversed through the shared publisher when the
retained producer sequence proves the return to its source address. The direct
protocol passed A → B → C → B → A → B → C, with one logical identity, six new
generations, separate saved and draft bytes and the original local history.
An independent same-path role cycle and an equal-byte, different-inode source
were refused. A separate protocol passed explicit creation Undo to absence,
Redo and a guarded Move inverse, with unsigned inverse Preview refusal before
a journal write.

Rendered browser acceptance exercised the six Move publications from the
existing command and review. It checked the original editor, selection, scroll,
geometry, local save history, and ordinary text Undo and Redo. It held a fresh
Undo preview across a same-byte inode substitution and refused publication.

The rendered browser also completed a live copied-destination interruption and
two cold interruptions, with the original present and already deleted. The
live case rejected an edit/Undo round trip after Preview and completed a new
review in the same editor. Both cold cases restored the sealed draft separately
from the saved baseline. A substituted destination refused both before and
after Preview. Two direct engine probes also covered a retained original draft
whose saved path was absent, an altered sealed generation floor and a changed
logical plan. Generated files stayed absent in the host fixtures.

The mounted AppKit and Android controller replays each passed one interruption
case and one older Move case with zero skips. They retained the same editor
or native text state, selection, scroll owner, saved baseline, draft and local
Undo. The older Move cases rejected the after-preview replacement and checked
the original request status without replaying the write. The final engine
passed 64 focused Rust cases across completion, lineage, interrupted evidence,
transition lifecycle, input guards, journal and publication. Two producer
fixtures established the copied and old-deleted starting states by interrupting
the actual publisher at the selected boundary.
[Older Move acceptance](workspace-entry-lineage-acceptance.md)
[Interrupted Move acceptance](workspace-entry-interrupted-acceptance.md)

### Mounted iOS shared publication and Quick Fix

The iOS 27 simulator completed the copied-destination and old-deleted cold
fixtures using the real shared publisher. Each replay recovered the sealed
dirty draft separately from the exact saved bytes and confirmed the original
entry identity and its new generation. The copied fixture retained the existing
UIKit editor. The old-deleted fixture mounted the recovered draft at the
destination. Both completed without generating outputs or adding ordinary
workspace Undo for the repaired partial namespace.

The mounted older Move replay completed A → B → C → B → A → B → C.
It kept the same role, editor, selection, 60-point viewport, geometry and
native Undo manager, and found the original saved-source history. A held Undo
review refused an equal-byte inode replacement. Reading the original request
status issued no second publication. The native widget now reflects the
adopted path in its accessibility label while preserving text and history.

The managed proposal replay separately prepared a literal-path rename, saved
it explicitly and published the reviewed inverse. Both steps adopted into the
same editor with its selection, frame and Undo manager. A fresh two-draft Apply
also passed. The clangd 21.0.0 Quick Fix reached its exact authored diagnostic,
reviewed the missing-semicolon edit, and applied and replayed it in the original
draft while preserving saved bytes.

The selected pytest/debugpy replay stopped at its exact authored test line,
evaluated the local value, continued and retained the filtered run. The sibling
stayed excluded and the saved bytes, UIKit editor and caret stayed exact.
Mounted application Debug used the configured Node/js-debug and .NET/netcoredbg
profiles. Both stopped at the saved source line, evaluated the local value 41,
continued, produced 42 and retained the original editor, pane and geometry.
The .NET adapter returned exit 0. The Node adapter finished without an exit
code. Their receipts are `mounted-node-application-debug-d54dlbbk` and
`mounted-dotnet-application-debug-ces9svoi`.

Each case passed one selected XCTest with zero skips. The result bundles and
receipts are under `target/ios-parity` in `publisher-copied-jduekzx0`,
`publisher-old_deleted-wv9_wsox`, `publisher-lineage-f4g49jzc`,
`mounted-managed-dujlatg4`, `mounted-quickfix-rl5lq716` and
`mounted-pytest-debug-r0p455_c`.
[Mounted iOS acceptance](../mechanisms/ios-mounted-parity-acceptance.md)

TypeScript Language Server's CRLF and LF extraction attempts reached Apply,
then coordinated Undo refused the altered preimage. UIKit had reflected
different bytes from the reviewed multi-edit patch. After the exact storage
fix, the CRLF and supplementary-Unicode replay preserved the planned text in
both the mounted editor and model. Undo recovered the exact dirty draft and
Redo restored the reviewed result with the original editor and saved bytes.
The focused receipt is `mounted-typescript-uvpaqkz1`. A separate mounted
UIKit replay of the genuine LF case passed one XCTest with zero failures
and zero skips. Review, Apply and both history steps kept
the exact LF bytes, the earlier dirty Paste, the original editor and saved
source. The XCTest bundle, fixture receipt and simulator cleanup are under
`target/ios-parity/lf-harness.3c5onn1o/focused.XXXXXXXX-receipts`.
The maintained hosted suite repeated the CRLF extraction and two-draft
managed Apply after the fix. Both passed one case with zero skips, with
receipts under `target/ios-parity/maintained-exact-reflection`.

The first scripted UIKit composition attempt failed its earlier history assertion.
The revised UIKit witness started from an independent native Paste. It kept
the provisional Japanese candidates in the primary field, mirrored on commit,
replayed composition Undo/Redo and then successfully undid the earlier Paste.
The retained result bundles are `mounted-typescript-fhnbwuu7`,
`mounted-uikit-composition-c3itk45s` and `mounted-uikit-composition-bends7zx`.
The paired physical iPhone was offline. A fresh Xcode 27.0 CoreDevice query
again reported it as `unavailable`. Physical input, device IME and VoiceOver
acceptance remain blocked. The two status receipts are under
`target/ios-parity/physical-device-availability-cjc9xsc7` and
`target/ios-parity/physical-device-availability-hh0i2srf`.

### Other fresh focused flows

Rendered browser replays covered basedpyright and TypeScript Language Server
navigation from ordinary fences, all five genuine TypeScript refactoring cases,
and exact basedpyright Problems navigation. The Rust source-test gutter replay
checked stationary adjacent controls, ran the selected Cargo test, retained its
output and refused a stale run from an edited buffer. A live pytest replay
discovered cases, retained a failed run and coverage, and reran only the
failure. A separate Vitest engine run distinguished the same test names in two
files and retained the three-run history.

Repository review passed a removed-file diff refusal, literal renamed paths,
saved-file stage and unstage, immutable history and saved attribution. The
browser now names the repository status separately from the nested conflict
status so the reading and its acceptance can address the correct update.

The connected browser reading replay reached the later fragment, its explicit
contract, explanation, compiler and test links, selected pytest and an exact
user profile. It retained current, stale, manual and process-only evidence as
separate states and rejected a substituted root and an obsolete engine reading.
The controlled provider engine run replayed whole-file diagnostic actions
across copy contributors, including opaque resolution and an unmappable
whole-action refusal. The saved-entry read refused an equal-byte substituted
source before verification. The agent browser replay read a sealed historical
proposal after restart and kept Apply unavailable. The independent engine
replay refused a tampered archive and preserved the exact original source.

The rendered browser conflict replays passed index, source and complete
namespace resolution. The rendered pytest/debugpy replay selected a test in
compiler-authored source, stopped at its body breakpoint, read locals and
watch values, then retained its source link and process outcome. The sibling
test stayed excluded, and the editor and saved source stayed intact.

An owned API 36 Android emulator also exercised the actual app's Files and
editor, whitespace Cleanup preview and draft application, basedpyright
Problems navigation, saved Git diff and stage, and default construction and
connected behavior readings. The reported evidence stayed missing when the
fixture supplied no related check. A real sealed agent proposal was readable
as historical, with its exact material digest and saved source preserved.

A separate rendered Move with clean source published `project.md` to the
extensionless `b`, preserving its logical identity, exact saved bytes and code
frame bounds. The old address and generated output remained absent. The older
Undo/Redo sequence and interrupted recovery ran on the Android controller.
The first rendered Rust Run attempt tapped the keyboard over the Run button.
A replay confirmed the window's keyboard was closed and captured exactly one
saved-source Run request with HTTP 200. The native panel retained
`rendered_android_case · name filter · finished · exit 0`, displayed the Cargo
case output, and preserved the exact saved bytes. Coverage and agent application
still need rendered Android acceptance.

The rendered recovery panel exposed a narrow-screen wrap in Close. A later
720 × 1600 replay confirmed that Close fits on one line beside Source recovery.
The older Move attempts read the exact dirty Cleanup draft and stopped before
dispatch while the keyboard or emulator was unavailable. A separate replay
prepared the cold producer fixture, then the emulator failed to launch the
app. Its Activity Manager service disappeared and its crash buffer recorded
`DeadSystemException`. No Move or completion request was dispatched by that
replay. The final isolated API 36 retry stopped before boot because userdata
needed 7,372.80 MB and 6,437.74 MB was available. The explicit smaller partition
option reached the same admission floor. After releasing reproducible focused
build intermediates, the last immediate preflight measured 7,364.54 MiB and
stopped before creating another AVD. The disposable data was removed after
each retry. Rendered cold recovery, the older Move chain and rendered Debug
still need a healthy Android runtime. The renderer and framework receipts are
under `target/parity-rendered-android.sgYIt7`.

### Rendered Android cold recovery, proposal Apply, coverage and Debug

With stable headroom and two successful framework readiness probes, the
rendered app opened the copied-destination interruption and reviewed its
sealed evidence. Preview left the saved namespace alone. One explicit
Complete returned HTTP 200, advanced the original entry identity, removed the
old address and kept the exact saved bytes at the destination. The reopened
editor held the separate dirty draft. The producer's existing private
`.history-state` files and other unaffected fixture bytes were unchanged.
The successful receipt is under
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-n88la8ql/cold`.
The rendered Android replay also completed the cold Move with the predecessor
already deleted. Its preview kept all saved bytes intact. One guarded Complete
returned HTTP 200, advanced the same entry identity, and confirmed the exact
saved destination. Reopening it restored the separate retained draft. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-75920z7_/cold-deleted/`.

The rendered app also applied a current engine-sealed, caller-prepared
proposal to two draft editors. One validation and one host report returned
HTTP 200. The report recorded the exact draft hashes. The saved files were
unchanged, no resource-save request was sent and no checks were reported.
The rendered review, wire trace and host report are under
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-lkerhb9u/proposal`.

The later Move replay reached the rendered form with Gboard open. Text
injection left the destination empty and the replay dispatched no Move.
Selected Debug was admitted and rendered pytest output, but the gutter tap
set the declaration breakpoint and the intended body-line pause was not
verified. The initial coverage panel exposed the intended pytest file in its
accessibility tree. The rendered WebView was largely blank, and only its
initial history read reached the engine. Those attempts are retained in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-tnas0o30`.

The embedded coverage page now scrolls in the WebView itself. Removing the
nested panel's dynamic viewport cap restored the visible controls. Rendered
Android Discover found both pytest cases in `test_app.py`. Run retained
one pass, one deliberate failure and 14 of 14 executable lines covered.
The `app.py` coverage matched its saved source hash. Failed-only rerun
executed exactly the negative test and rendered a separately retained result.
The rebuilt APK digest, screenshots, UI trees, engine trace and collected
runs are under
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-o6ite4sk`.
Mounted iOS also passed its workspace acceptance with the revised embedded
layout, including the failed pytest result, coverage and exact source
navigation. Its one-case, zero-skip receipt is in
`target/ios-parity/embedded-coverage-layout`.
The same AVD selected a saved pytest test, stopped at its body breakpoint on
line 2, continued and retained exit 0. The saved source stayed exact and the
sibling did not execute. The paused and retained-process screens are in the
same receipt directory under `debug`.

An earlier rendered lineage replay published the first dirty Move from
`project.md` to `b`. One resource-save request returned HTTP 200. The
editor retained the exact draft and frame without fetching a new baseline,
while the destination held the saved bytes. The logical entry kept its
identity at generation +1, and the old and generated paths remained absent.
The second destination field had focus but stayed empty after text
injection, and Gboard stayed hidden. The replay stopped before a second
publication. The confirmed first transition and the blocked second form
are in `final-android-5ug201y6/lineage` under the same receipts directory.

The subsequent API 36 replay began with 10.18 GiB available and used the
APK with SHA-256
`4c47f1c7456ff1ffe14fb53c65e518728b3078285f5f92e965dce7d2f2f94590`.
Both destination fields accepted Android key events with Gboard hidden.
The rendered commands used Android accessibility click actions. The app
confirmed `project.md → b → c → b → project.md → b → c` through two Move
and four restoration publications, each returning HTTP 200. Every step
retained the original entry identity with one generation advance, the exact
saved bytes and separate dirty draft, and the editor frame without fetching
a new baseline. The optional Local source history read stopped after the
sixth confirmation because its menu item was outside the reachable viewport.
The per-step screenshots, input and wire receipts, and replay sources are in
`final-android-xc10nz1r` under the same receipts directory.
The maintained Android controller lineage witness separately passed one test
with zero skips. It checked the same editor state, selection, scroll and
local history, plus refusal after an equal-byte inode replacement. Its result
is in `target/native-template-smoke/android-entry-lineage-move-handoff-last.xml`.

An earlier Gboard replay entered `b` on the visible software keyboard and
verified the first Move. It stopped in the test's menu lookup before entering
`c`, even though the Move action was present. The strict follow-up now verifies
Gboard's active destination field and Done action before tapping each visible
key. It requires all six publications. The first strict attempt stopped at
preflight with less free disk than the 7.2 GiB device boot floor. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-_giengpj/lineage/`.

The current API 36 replay tapped the visible Gboard `b` and Done keys.
The first guarded Move kept the entry identity, exact draft, and editor frame.
For the second destination, Android reported an active single-line Done
connection while Gboard still showed the multiline Enter key. The test
stopped at that mismatch before typing `c` or publishing a second Move.
See `target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-mmi27c0r/lineage-gboard/`.

A later API 36 replay entered both destinations with the visible stock
Gboard `b`, `c`, and Done keys. Both fields needed a second tap to show the
keyboard. The two Moves and four reviewed Undo and Redo publications all
returned HTTP 200. The entry kept its identity from generation 0 through 6.
Each step kept the exact draft, viewport, and saved bytes. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-wd5zj64w/lineage-gboard/lineage-gboard-summary.json`.

The Android application Debug follow-ups sent both genuine adapters a saved
line 2 breakpoint. The Node renderer paused at that line with local value
41. The Watch expression field failed to take focus on the earlier replay.
A later frame-local read hit a retired inspector control before Continue.
The .NET renderer read selected-frame local value 41, continued, displayed
42 and retained exit 0. Its fixture verified the exact source and editor
frame. The launch, pause and finish receipts are in
`final-android-rrq_6ur7/provider-dotnet` and the Node pause is in
`final-android-rrq_6ur7/provider-node`. The earlier expression-field attempts
are in `final-android-tdjwr952`.

Rendered managed Send stopped at the empty prompt despite key events,
Gboard taps and Android's accessibility text action. A separate fixture
initiated owned ACP using the app's private checkpoint to bind two pristine
openers, including their live editor IDs and saved guards. The normal Android
review applied the engine-managed proposal and both rendered drafts matched
the planned bytes. Validation and the exact host report returned HTTP 200.
The report marked saved and checks-executed false, and the wire retained
draft-only application. The keyboard overlay stopped the replay before
coordinated Undo or Redo. The checkpoint binding, tool result, review, Apply
and boundary receipts are in `final-android-rrq_6ur7/provider-managed`.

The later rendered Android checkpoint replay opened both original editors,
reviewed the engine-managed proposal, and applied it to two drafts. The live
host report bound both draft hashes and left resource publication and checks
untouched. The Android Edit menu then undid and redid the two drafts together.
Both saved preimages stayed byte-identical. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-75920z7_/managed-checkpoint/`.
The native Android composer also submitted the exact prompt and both mounted
editor identities, source revisions and saved guards. The owned relay routed
only the backend selection to the scripted local ACP provider. The engine
returned HTTP 202 and Android rendered the completed archived turn. The
composer kept its submitted text. This run made no subscription call. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-75920z7_/managed-native/`.
The unified native Send through coordinated draft history still lacks a single
completed replay. The follow-up reached the focused composer but neither the
hardware input command nor the accepted accessibility text action populated
it. That follow-up sent no request. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-4_rv2zul/managed-native/`.

A later replay found that the Android accessibility tree can report an empty
editor while a fragment of the prompt is visible. The owned relay required the
exact intended bytes and rejected an incomplete native Send before forwarding
it. The checkpoint-based Apply and draft history evidence remains current. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-tgm6itzy/managed-native/`.

The Android Agent form now bounds the scrolling composer and provider options
above its existing Send and Stop footer. The previous fixed-height form
clipped the footer when Gboard reduced the pane. On the rebuilt app the
original composer stayed focused with the keyboard visible, and both actions
lay above it. Tapping Send submitted the exact prompt and both original
editors to the owned local ACP fixture. The engine accepted the request and
completed the turn. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-qjdb4ps6/managed-native/`.
The combined retry's emulator preflight saw 6.723 GiB free against its 7.200 GiB
boot floor. The bounded recheck peaked at 6.890 GiB and no emulator was created.
That retry could not exercise native Send through Apply, Undo and Redo.

The Android composer now clears an acknowledged submission only when the
submitted editor revision is still current and no composition is active. The
held loopback test exercised a later draft with different bytes, a later edit
that returned to exactly the submitted bytes, unchanged success, and refusal.
Both later drafts and the refused draft survived intact. The same buffer and
native editor state were retained. See
`target/android-agent-send-draft-revision-unit.77ugzt4j/`.

The rebuilt Android app then passed native Send through the entire two-draft
review and history flow on one API 36 emulator. The composer retained focus
with Gboard visible at Send. The exact prompt, two original editor IDs and
guards reached the owned local ACP fixture. The rendered review applied both
drafts, a single workspace Undo restored both, and one Redo reapplied both.
The sealed validation and draft-only host report succeeded. Saved file hashes
were unchanged, with no publication or checks. The owned device was stopped
and removed. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-d1xcite4/managed-native/`.

The mounted AppKit native Send probe opened both original source editors and
entered the exact prompt in the native composer. Its captured window showed an
enabled Send beside Stop. The isolated click replay did not yield an Agent
request or archived turn, so the complete AppKit Send-through-draft-history
journey still needs acceptance. Earlier engine-managed review receipts begin
with an already submitted request. See
`target/main-macos-packaged-startup.to8jhtpo/apple-native-proposal-send-xcb43d7a/`.

The maintained AppKit replay then passed the complete native Send journey.
The mounted text view accepted the exact prompt and a native click produced
one accepted request with the two original editor IDs, revisions and guards.
An isolated relay changed only the backend name to the scripted local ACP
provider. The mounted review applied the sealed batch, and the native editor
views reflected coordinated Undo and Redo through the workspace commands. The
original editor identities, selections, frames, guards and saved/generated
bytes remained intact. The host report recorded draft application and no
publication or executed checks. See `target/apple-appkit-send-maintained.MUyXom/`.

On iPhone, the accepted Send clears and releases focus from the submitted
composer only while that exact draft revision is current. The prior replay
left the empty composer focused and the conversation at zero height. On the
rebuilt app, the simulator keyboard entered the exact prompt, Send returned
HTTP 202, the keyboard dismissed, and the recorded user card displayed that
prompt in the existing conversation. The relay routed the request to the
same owned local ACP fixture. See `target/apple-native-send-uikit-focus.xmfyVX/`.

The maintained iPhone simulator replay continued from the archived prompt
through sealed review, Apply and the Edit menu's coordinated Undo and Redo.
The two source editors showed the exact text and retained their frames after
each transition. Send carried both original editor identities, revisions and
saved guards. The host report recorded draft application, and all five fixture
files kept their exact saved bytes. The disposable simulator was deleted.
See `target/apple-native-send-uikit-history-2026-11-19/`.

### Provider admission

Language tools use the installed provider's capabilities and exact source
mapping. The candidates include alternatives per language, such as OmniSharp
and csharp-ls for C#. A missing provider or unavailable action leaves readable
source and reports unavailability. A discovered action grants no authority
until its complete source effects and original target versions can be reviewed.
[Language-server candidates](../mechanisms/crates/hick-plugins/src/languages/lsp.rs.md)
[C# provider declarations](../mechanisms/crates/hick-plugins/src/csharp.rs.md)

The saved test Debug providers exercised here are pytest with debugpy and Cargo
with LLDB. C# collects tests through the .NET runner. A recognized xUnit
candidate returned an unavailable C# Debug plan. A direct guarded Start on
that exact saved candidate returned only Failed, without preparation or a
session. The runner supplies no test debug recipe.
The separately installed netcoredbg 3.2.0-1092 passed a rendered browser .NET
10.0.401 application launch. The actual adapter stopped at line 7 of
`Program.cs`, evaluated `value=41`, continued and retained output `42` and exit
0. Saved bytes, editor identity and geometry stayed exact. Installed js-debug
1.117.0 with Node.js 24.22.0 also reached the rendered browser's breakpoint at
line 2, evaluated `value=41`, continued and retained output `42` and one
finished result. The adapter supplied no exit code and the UI reported none.
Mounted macOS replayed both launch profiles in the current AppKit build.
Both stopped at line 2, read local value 41, continued and retained output
42, the same editor frame and exact saved bytes. Node finished without an
exit code. .NET returned exit 0. The receipts are in
`target/mobile-provider-followup/mac-current-acceptance`.
The rendered browser also passed loopback netcoredbg PID attach for an
already-running owned .NET process. It paused with value 41 and withheld
saved-source authority. Detach ended the Hickory session and the target
remained alive. Releasing the owned fixture then produced 42 and exit 0
with exact saved bytes and the same editor frame. The wire and paused
screen are in `target/mobile-provider-followup/netcoredbg-attach-first`.
The browser's Node attach received Started and the target reported
`Debugger attached`. js-debug then requested a child session with
`startDebugging`. The host did not answer that request, and the target's
debugger statement produced no rendered pause in the bounded replay.
That replay could not exercise Watch or detach. The adapter and host traces are in
`target/mobile-provider-followup/js-debug-attach-first`.
The explicit `pwa-node` attach now follows the adapter's child on the same
loopback endpoint. Two focused TCP fixtures passed. They checked ordinary
single-connection attach, exact child configuration and stack reads on the
child. Each connection sent a detach with target termination disabled. Both
setups withheld saved-source authority. The tests compiled the current debugger
crate against a consistent cached dependency graph. A separate replay through
that debugger crate and genuine js-debug paused an owned Node process, evaluated
Watch as `41` and withheld saved-source authority. Both connections detached
without terminating the target. It remained alive, then produced `42` and exit
0 after the fixture released it.
See `target/dap-owned-js-attach-2026-11-19/`.
The rendered Simple Browser replay then passed on the rebuilt `224854bb`
engine. The real attached frame supported Watch `41`. Stop detached both
connections with target termination disabled, and the target remained alive
until release. It then printed `42` and exited 0. Editor identity, dimensions
and saved source bytes stayed exact. The browser did not claim source authority.
See `target/node-browser-rendered-224854bb/`.
The confined CodeLLDB 1.12.3 install and direct DAP initialize passed. Its
first simple-host Rust attempt stopped before Started. A fresh browser replay
started the actual adapter, stopped at line 3 and displayed the inline value
41. Selected-frame locals and Watch refused the frame. The raw adapter issued
IDs 1001 and 1017 in successive stack reads for the same paused thread and
source line. A raw Watch on the latest frame returned 41, and raw Continue
produced 42 and exit 0. A separate browser Continue attempt timed out during
page load. The receipts are in `target/parity-codelldb-replay-xvhbeppi` and
`target/parity-codelldb-frame-replay-noaumdne`.

The interactive bridge now binds displayed frame IDs to the published
stop and thread. It admits Locals, Watch, child values and adapter source
from that same stack, then retires those references on refresh. A controlled
adapter changed its frame IDs on every stack read. The focused bridge replay
verified one stack fetch per observation, live reads on the published IDs and
refusal of retired IDs or references before another adapter request. Two genuine
debugpy checks also passed for thread selection and captured adapter source.
The rendered Simple Browser replay then passed with CodeLLDB 1.12.3 and
the current bridge compiled against cached dependencies. It stopped at line
3, requested Locals and Watch against published frame 1001, and displayed
`41`. Continue retained output `42` and exit 0. The editor identity, frame
and saved source stayed exact. See
`target/parity-codelldb-replay-current-hlftmoio/`.

Hickory's installer reported that it has no Delve recipe. The browser
replay used independently installed Delve 1.26.3 and Go 1.26.8. The adapter
ended before sending `initialized`, so the selected breakpoint remained
unverified. Both confined Java installs passed with the owned JDK available.
The rendered Java Debug profile failed before adapter startup with
`a debug adapter command cannot be empty`. The Go and Java browser wire
receipts are under `target/mobile-provider-followup/go-browser-q5oorx5j` and
`target/mobile-provider-followup/java-browser-lcvkp401`.

A later rendered Go session with Go 1.26.8 and Delve 1.26.3 verified the
saved breakpoint, showed Watch `41` and finished, while the program's
`42` appeared in Delve's host output. Hickory now requests Delve's remote
output mode. A direct Delve 1.26.3 replay confirmed DAP stdout and termination.
The rendered Simple Browser replay with the rebuilt engine stopped at saved
line 5, showed Watch `41`, and retained stdout `42` and exit 0 after Continue.
The same editor, its dimensions and exact saved bytes survived. See
`target/mobile-provider-followup/go-browser-opnyelru/` and
`target/mobile-provider-followup/go-rendered-current-remote-output/`.

Go Debug now has a managed Delve setup path. Hickory selects a pinned Delve
1.26.3 archive for its host, checks the digest before unpacking into the
workspace, and discovers that adapter from nested Go sources. A clean Simple
Browser workspace requested the real install only after Debug was selected.
The installer returned HTTP 200, and the retry started the workspace's new
`dlv dap` process. With no Go SDK on PATH, that replay timed out waiting for
the retry's terminal response. It establishes first installation and adapter
discovery. See `target/delve-install-browser-receipt-v1/`.

With the verified Go 1.26.8 SDK on PATH, the final clean browser replay
installed the official Delve 1.26.3 binary and retried once. The real
debugger paused at the saved breakpoint in `main.main`, line 5. Continue
displayed `42` and retained Finished. The adapter did not report a numeric
exit code. The same editor and exact saved bytes survived. See
`target/delve-install-browser-completion-proof/`. The focused Rust discovery
and browser setup tests passed separately.

The installed js-debug adapter also passed a rendered Node.js project with
a separate copy contributor. Step Into moved from authored `launch.md:7`
to the exact captured source at `helper.md:6`. Scoped Watch read `seed = 41`.
Continue retained stdout `42` in the launch tab. Reselecting it kept the same
editor and viewport dimensions. Both authored files kept their exact bytes,
and the generated JavaScript stayed unpublished. See
`target/js-contributor-browser.RuI2FP/receipt/`.

Java Debug now asks the installed language server for its hosted DAP endpoint.
The main class must match the selected source file. The adapter receives the
profile's arguments in its string format and the session owns the temporary
language server. Four focused helper cases passed. The rendered Simple Browser
profile stopped in `Main.java` at saved line 4, displayed Watch `41` and printed
`42` after Continue. The program independently confirmed arguments with spaces,
an empty value, quotes, backslashes and Unicode. The original editor and saved
source stayed exact, and the hosted language server exited. See
`target/mobile-provider-followup/java-hosted-current-ua6i1lvj/profile-exact-argv/`.

Java files with a conventional static `main` now produce an application
profile without workspace configuration. The rendered plain-file replay stopped
at saved line 4, read Watch `41`, and verified that Stop ended both the
debuggee and the hosted server. Both editor and saved source stayed intact.
See `target/mobile-provider-followup/java-hosted-current-ua6i1lvj/discovered-plain-stop/`.
The browser also refused a stale displayed Java revision before starting a
hosted server. The Java Run terminal completed through the discovered profile
with output `42`, exit 0 and exact saved source in a folder with a literal
`#`, `%`, spaces and Unicode. See the `stale-refusal` receipt beside the
plain-file Debug case and `target/mobile-provider-followup/java-run-terminal-h875o49g/`.

A separate workspace with `#`, `%`, spaces and Unicode in its folder name
used symlinks to its installed Java provider. The rendered Debug attempt
timed out waiting for LSP initialize. A direct provider probe exited before
replying with those symlink paths, then initialized with the same workspace
URI and the canonical installation paths. The hosted launcher now resolves
the installed artifacts before starting Eclipse. The same rendered workspace
then discovered `Main.java`, stopped at saved line 4, read Watch `41`, and
ended both the debuggee and language server on Stop. The editor and source
stayed intact. A focused provider-exit test also verified that closing the
reply stream fails a waiting initialization before its timeout. See the
`canonical-installation-stop` receipt under
`target/mobile-provider-followup/java-hosted-current-literal # % λ/`.

A real Apple clangd 21.0.0 Simple Browser replay opened the exact authored
missing-semicolon position from Problems, previewed the single insertion,
and applied it to the draft. Undo and Redo retained the same CodeMirror
editor and viewport, with the caret following the edit. The saved bytes
remained exact and the browser made no source writes. See
`target/clangd-quick-fix-browser-v2/browser.json`.

The rendered Android 17 replay opened the same clangd diagnostic from
Problems and requested the source-bound Quick Fix at its exact position.
The review showed a single semicolon insertion. Apply inserted only that
character in the draft. Native Undo removed it and Redo restored it.
The original editor kept its frame after the keyboard appeared, and the
saved source, provider configuration and workspace entries stayed exact.
See `target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-9wtgynfc/clangd-quick-fix/rendered-clangd-quick-fix.json`.

Configured csharp-ls 0.28.0 on .NET 10.0.401 supplied rendered browser
definitions, usages, back navigation and a two-file rename. Review applied the
rename to drafts and coordinated Undo restored both original texts. Its plain
saved C# diagnostic reported the string-to-int compiler error as current and
Next Problem reached its exact UTF-16 location. The original editor, its
geometry and all saved files were retained. This provider advertises a static
document diagnostic selector and sends unversioned pushes. A bounded document
pull can establish current source-bound context using the exact child connection
and authored revision. Four focused Rust cases and a controlled-child replay on
the final engine exercised same-byte revision retirement, coalescing, timeout,
child exit and ordering against unversioned pushes. Other document mappings
and workspace diagnostic pulls remain outside this receipt. The confined .NET
tool installer could not load the NuGet service index and reported an arithmetic
overflow with a local feed. The live witness used an explicit `.hick-lsp.json`
workspace command to launch the package from the public NuGet release.
The configured OmniSharp 2.0.0 Apple Silicon replay reached the rendered
definition command, which timed out waiting for `textDocument/definition`.
The launch configuration and trace are under
`target/mobile-provider-followup/omnisharp`.
[Debugger adapter declarations](../mechanisms/crates/hick-plugins/src/languages/dap.rs.md)

## Inspect authored code in ordinary Markdown fences

An installed language provider receives the exact fenced source in a private
inspection document. Each reading maps through the current fence to the
authored editor. The engine retires inspection documents when their source
revision or rooted workspace changes, and the provider gets no edit or
execution authority from a pathless fence. A distinct fence has its own
document identity. Shared imports or inferred relations between different
fences remain unknown.

Five focused fence and routing cases and eleven position-mapping cases passed.
Actual Python and TypeScript children returned hover and definitions with
indented Unicode and CRLF source. The protocol also exercised dirty changes,
Undo, close and unsupported edit requests. Rendered browser navigation and
mounted macOS and Android controller inspections reached the authored
locations and preserved the original editor, saved bytes and workspace files.
Two mounted Android 17 replays used the same ordinary CRLF and Unicode
Markdown fixture. Native Language Tools selected `notes.md:5:7` for
basedpyright and `notes.md:11:31` for TypeScript Language Server. Each
selection returned to the same editor. A second Definition request from
the selected caret reached the exact declaration inside its fence. Both
replays preserved saved bytes, provider configuration and the file list.
The Python keyboard shortened the viewport at the bottom while its top
and side edges stayed fixed. The TypeScript editor held its full frame.
See the rendered receipts in
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-v_z3avk0/fenced-definition/`
and `target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-jjuqbi_2/fenced-definition/`.
The standalone iPhone replay opened the same fixture in two launches on
one owned simulator. Measured native taps invoked Go to Definition at each
author's caller caret. The floating Language tools panel scrolled to reveal
the rendered results `notes.md:5:7` and `notes.md:11:31`. Native selection
moved the same UIKit editor to each authored declaration. A second request
from that caret reached the exact declaration in the private provider
document. The saved source, provider configuration, workspace entries and
Undo manager stayed intact. Earlier attempts tapped during menu movement.
See `target/ios-fenced-definition-direct-stable-both/{python,typescript}/receipt.json`.
Physical device input and assistive technology remain unverified for this flow.

## Debug a saved Rust test selection through Cargo and LLDB

The source-test Debug command prepares Cargo artifacts in its own target
directory. The launcher checks the compiler's artifact identity and the target's
manifest before listing built-in harness names. It resolves one qualified name
and gives LLDB its exact filter. A custom Cargo harness refuses before listing.
The retained session reports process exit separately from a collected test
verdict. The physical Cargo dependency closure remains unknown to this check.

The focused groups covered artifact confinement, an exact Cargo run, physical
root replacement, custom harness refusal and process-group cleanup. The real
Cargo and LLDB fixture exercised ordinary Rust source and compiler-authored
Rust. Browser rendering, mounted macOS and the Android controller followed the
same path through Breakpoint, Continue and retained history. They excluded the
sibling test and preserved editor state and saved bytes. The existing real
pytest Debug path also passed. This receipt uses qualified name-filter
selection. The later declaration-to-artifact witness adds the guard below.

Stock Android opened the saved Rust source from Files, chose the exact
`test_selected` row and kept the same readable editor while Cargo prepared.
The native preview showed preparation, and the engine admitted the debugger
after 40.48 seconds. The selected test passed, its sibling was filtered and
the retained process exited 0. This replay measured preparation and
selected execution with an empty breakpoint list. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-t5aatbe0/rust-debug/debug-wire.json`.

A compact Rust fixture then painted the exact native line 7 breakpoint.
Cargo admitted the genuine LLDB session after 40.75 seconds, and execution
stopped at that bound line. Returning from the test panel opened the
keyboard and scrolled to the typing caret below the stop. The Return to
editor debugger controls button now dismisses the panel without requesting
text input focus. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-dduu3k15/rust-debug-unwrapped/debug-wire.json`.

The rebuilt app returned after a genuine line 7 stop with the keyboard
closed. The source filled its original frame and the enabled Continue
control stayed visible beside line 7. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-iff63q2p/rust-debug-unwrapped/failure-rendered-state.png`.

The line-number gesture now stays attached while the layout updates and
checks the current authored source before toggling a breakpoint. Stock
Android painted line 7 on the first press, admitted the selected Rust test
after 38.43 seconds of preparation, and stopped at that line with `n=41`.
Returning from the panel
kept the editor frame and the visible Continue control with the keyboard
closed. One native tap sent Continue for the paused session, which then
exited 0. The sibling was filtered out, and the editor identity and saved
file metadata stayed fixed. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-o_pz8w83/rust-debug-unwrapped/receipt.json`.

The stock startup replay also exposed a discovery race. The editor could
compose before its test index attached, then keep the same source text
through promotion without running discovery again. Test discovery and
coverage now observe their own attachment and use the source address in
their effect keys. A focused Compose witness observed those handles arrive
separately at the same source and selection.

The rebuilt stock Android app opened the requested Rust file and its
containing folder while the saved-source guard was held for 8.09 seconds.
When the guard settled, one discovery request populated both authored test
rows on the same editor without reopening the file. The editor ID, viewport
bounds, saved bytes, inode and modification time stayed fixed. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-d7rnxm3e/startup-discovery/receipt.json`.

### Match the saved Rust test declaration to the emitted function

Before Cargo lists a harness, the launcher checks its root against the captured
package targets and active module routes. Read-only LLDB metadata must bind the
qualified test symbol to the selected physical input and declaration span.
Ambiguous module routes, unavailable source metadata, and unsupported custom
harnesses refuse. The receipt describes the selected declaration and emitted
artifact. The compiler, build scripts and dependency graph remain independent
of that proof.

Six focused declaration cases, six emitted-source cases and three existing
debug-plan checks passed. The focused Rust launcher also exercised real Cargo
artifact listing and refused a custom harness. The actual pytest Debug case
passed with its original authored source and sibling exclusion.
The genuine workspace protocol exercised six inline, external-module and
disabled-sibling cases across ordinary and compiler-authored Rust. A disabled
declaration refused before a debug session started. Rendered browser, mounted
macOS and the Android controller each stopped in the four supported cases,
continued to a successful process exit and preserved saved source, editor
identity and history. The sibling test never ran. iOS feature runtime, Android
rendering, physical input and assistive technology remain unverified.

## Use expanded transforms in source-bound completion

A completion host explicitly advertises the transform programs it can execute.
The engine binds that choice to the original query and resolve, along with the
source and selection. Legacy requests retain program 2. The newer program uses
bounded matching, shared Unicode data and one work budget for the complete
edit. Lookbehind, named captures, backreferences, set operations and scoped
flags can produce linked edits. Evaluation that exceeds its budget refuses the
whole linked edit and leaves Undo history intact.

The focused Rust cases checked 637 executable oracle cases, 221 syntax
refusals, 23 replacement format cases and 23 work or output bounds. The
browser, Apple and Android models consumed the executable fixtures. A fresh
engine with a real child process checked initialization, original resolve
payload, stale and foreign replies, and legacy refusal. Browser rendering,
mounted macOS and the Android controller each exercised completion through
linked Unicode input, budget refusal and Undo/Redo. Those runs retained editor
identity, source and saved bytes. The iOS simulator product built. Reusable
template negotiation, iOS runtime, physical input and assistive technology have
separate acceptance requirements.

## Review and apply a complete template group

Catalog and Preview retain every logical selection, the same graph and one
variable capture. Each selection receives its own cursor values and replacement.
Nested template review preflights every linked field with one bounded plan.
Apply commits the group in one history step. A moved secondary caret invalidates
the pending plan. The clipboard is read once when the complete expansion needs
it, and denial leaves source untouched.

The real browser, mounted macOS editor and Android controller each exercised
embedded code and a dirty reused fragment through Preview, Apply, linked
Unicode edits, complete Undo and Redo, and refusal after a secondary selection
moved. They retained the original editor and saved bytes. Thirty-four focused
browser checks passed with type checking. The native focused runs passed eleven
Apple and twelve Android cases. Each native live fixture then executed its one
opt-in case with zero skips. A second native run moved only the secondary caret
away and back while the primary and source stayed fixed. Both hosts rejected the
old preview and retained the draft and history. The iOS simulator product built
with the selection-identity change. A preceding iOS build was installed and
launched separately in an owned simulator.

## Use expanded transforms in reusable templates

Catalog, Preview and Apply retain the original supported program list with the
complete source, caret and context capture. Explicit program 3 enables the
expanded transform syntax. Existing callers can retain program 2, including the
earlier single-caret request shape. Variable demand and expansion use the same
parser and shared work budget. Android compares independently decoded program
values structurally so a freshly acquired identical plan can apply. A changed
program or original selection still retires the complete preview.

Nineteen focused engine template cases passed. Browser template and view cases
and TypeScript checking passed. The focused native runs passed twelve Apple and
thirteen Android template cases, plus three Android transform cases. Rendered
browser, mounted macOS and the Android controller each passed the new
lookbehind and Unicode-property flow and the ordinary compatibility replay.
The replays retained embedded and dirty reused source, nested mirrors, editor
identity, unchanged saved bytes, and exact Undo and Redo. The arm64 iOS
simulator product built. iOS feature runtime, Android rendering, physical input
and assistive technology remain unverified for this extension.

## Embedded source template evidence

Templates now use the authored language at the selection. The same captured
compiler graph supplies the language when a fragment belongs to a selected
construction. Multiple compatible uses retain that language. Mixed languages,
unavailable contributors, generated ownership and changed entry proofs refuse
the affected action while the editor keeps showing the source.

Twelve focused engine cases and 29 browser cases passed, along with TypeScript
checking. The focused Apple and Android catalog runs executed nine and ten
cases, respectively. Their opt-in tests were executed separately with the
owned engine fixture. The rendered browser, mounted macOS editor and Android
controller each passed fenced code and a reused fragment with dirty
contributors. The flows verified contextual comments, authored filenames,
Unicode transformed mirrors, exact undo and redo, retained editor state and
unchanged saved source. The browser also held Apply while the physical entry
changed and rejected that old proof. Mixed-language and generated-location
refusals were exercised in the browser. The native fixture exercised dirty
contributor changes, mixed languages and read-only source. The iOS simulator
product build is build evidence. UIKit, Android rendering, physical input and
assistive technology remain unverified.

Embedded language follows each fragment's actual compiler occurrence. When an
authored fragment contributes to a generated Markdown code fence, the resolver
uses that fence's language and preserves direct output coverage. Link metadata
adds no language authority. Seven focused resolver checks passed, including
incompatible languages inside one output. The real browser, mounted macOS and
Android controller replay included the generated Markdown occurrence alongside
the two saved-output candidates. Preview and Apply still changed only the
authored draft. The guide was never materialized.

## Multi-caret template editing evidence

The browser editor passed grouped typing, coincident empty logical carets,
primary selection retention and exact insertion/input undo and redo. It
refused changed editor intentions even when the visible selection was equal.
The macOS AppKit editor and Android text-state witnesses retained both native
selections through insertion, linked typing, field traversal, undo and redo.
The focused groups passed 22 browser, 11 Apple and 10 Android checks.
A later focused replay passed nested insertion, return to each outer field
and coordinated undo with 23 browser, 15 Apple and 14 Android tests. The
browser also continued two logical empty carets drawn as one cursor. The
catalog still needs to prepare and review all caret expansions from one
captured source. Physical input, assistive technology and device rendering
were not exercised by these witnesses.

An input inside an active editable field belongs to its linked edit. If a
transformation or grouped expansion exceeds its budget, the existing editor
keeps the draft, field session and selection, and reports the refusal. The
browser CodeMirror transaction, macOS AppKit input and Android text-state
witnesses exercised both a transform above its input limit and a grouped edit
above its combined output limit. The following valid edit and Undo/Redo still
reach the original field history. The final focused groups passed 27 browser,
14 Apple and 17 Android cases. This receipt covers ordinary input. Refused
marked composition still needs its own acceptance.

The template service prepares the complete caret roster against one captured
compiler graph and catalog. Each row retains its selection and cursor values.
The roster shares the clock, random seed, work budget and clipboard decision.
A changed secondary selection retires the whole catalog. Eighteen focused
engine cases passed. An owned HTTP fixture exercised ordered rows, a single
clipboard demand, denied fallback, stable requery and changed-secondary
refusal with unchanged saved text. The host catalog, preview and guarded
application of this ordered batch still require their own integration.

## Native marked template input evidence

The macOS editor's marked-text interface showed two provisional Japanese
candidates in the current field while leaving its mirror alone. Commit
updated both fields, and one Undo restored the original source, field and
selection. The next Undo still reached the independent earlier edit. A
separate unmark case retired the linked intent before commit and kept the
ordinary native insertion and its Undo. The focused Apple run passed 14
snippet and history tests. The iOS simulator target builds this path.
Android now observes the existing Compose input connection. The text-state
witnesses keep candidate text in the primary field until commit, keep
ordinary input when the connection closes, and replay one logical undo or
redo without consuming earlier text history. All 12 snippet interaction and
five source undo checks passed. The Android input connection compiled with the
installed SDK. Physical IME input, actual UIKit interaction, Android device
interaction and assistive technology remain unverified.

The refusal witness adds an oversized committed candidate after a provisional
Japanese character. macOS restores the original AppKit text, selection and
field context without adding an Undo step. Android replays only its observed
native candidate revisions to the original state. The earlier edit still has
Undo and Redo, the rejected candidate cannot return through Redo, and the next
ordinary edit gets its own history. The focused runs passed 25 macOS and 19
Android cases, including adjacent source-identity and history checks. The iOS
simulator product builds. Hardware keyboards, device IMEs and assistive
technology remain unverified.

## Canonical entry reading evidence

Explicit construction selection reaches the compiler independently of the
filename language. A fresh engine returned authored test candidates and an
unmaterialized compiler context for both a literal `.spec` address and a `.py`
address. The exact same `.spec` request without entry selection refused as an
unknown plain language. All saved source bytes stayed unchanged.

### Move and reverse the current construction without saving its draft

The existing Edit command captures the current main and prepares the exact
destination through the shared resource review. Saved compilation follows the
original disk bytes. Draft compilation and editor history follow the retained
dirty text. A live intent, the complete review and the original physical
namespace guard publication. Each confirmed Undo or Redo requires a new review
and advances the same logical entry generation.

The real browser, mounted macOS editor and Android controller each completed
Move to a literal `.spec` path, reviewed Undo and Redo, and local editor Undo
and Redo with the original dirty draft and saved baseline. The browser refused
a foreign destination created after Preview, preserved editor geometry and
followed the relocated entry in connected readings. The actual engine protocol
also retained the role after restart, refused the old live token and an
equal-byte inode substitution, and kept a recreated old spelling independent.
Neither compiler layer materialized generated outputs during review or Move.
The shared Apple host code built for the arm64 iOS simulator. Chained older
inverses, interrupted publication recovery, iOS feature runtime, Android
rendering, physical input and assistive technology still require acceptance.

After restart, Files reattaches the retained entry's document identity and
language independently of its filename. The actual engine reopened the moved
`.spec` source through the document route with its original saved bytes and
physical guard. A neighboring `.spec` with the same declaration remained an
ordinary file. The rendered browser Move and connected-reading replay and
TypeScript checking also passed.

### Retain the editor when publication ends before any writer

The entry publisher checks the live room before its filesystem callback. A
confirmed no-effect result binds the exact request, intended transition and
unchanged source address. An unused reservation consumes its generation only
after exact original-namespace revalidation. A started or uncertain writer
keeps its pending evidence for recovery.

The real HTTP fixture held a foreign Yjs room on an absent destination. Move
returned a confirmed zero-write result with the original generation and no
adoption. Repeating the request returned the same outcome, and a fresh review
after daemon restart completed one Move. Rendered browser, mounted macOS and
Android controller witnesses each exercised the same refusal. They retained
the draft and editor history and opened a fresh review. Forty-five focused
browser cases and TypeScript checking passed. The arm64 iOS simulator product
built. Interrupted completion and device interaction remain separate work.

Six focused engine cases and the actual workspace-entry HTTP flow passed. The
flow retained the entry identity across a fresh engine process, issued a new
process-bound proof, and observed new source proofs after an edit and after
restoring the original bytes. A malformed bound entry remained readable. A
missing entry, symlink replacement, tampered private record and renamed
workspace namespace refused the entry reading while unrelated text stayed
readable. A workspace with no entry could discover its first authored entry.
The focused engine cases also submitted stale proofs to the shared validator.
Template consumer admission, entry Move and its inverse still require their
own integration evidence.

### Keep the original entry in a connected reading

The existing Construction reading surface resolves a blank entry field to the
current workspace entry. An explicit address uses its literal bytes. A default
reading keeps its original proof through the construction preview, behavior
graph, source pages, profile choices and verification. Retiring an address,
contributor selection or draft roster retires the in-flight selection.

The actual engine refused a proof paired with another path and an original
proof after an identical-byte file replacement. The old proof could start no
verification. A fresh proof and the explicit address still read the source.
The browser passed the existing menu flow, draft comparison, contributor link
guards, and editor identity, dimensions and undo/redo checks. The mounted macOS
editor and Android controller each passed a live connected reading with the
entry proof, full source paging, an exact selected check, a chosen manual
profile and retained editor history. Saved source stayed unchanged. The
browser group passed 30 focused cases and type checking. Focused native groups
passed 13 Apple and 14 Android cases, with each opt-in live case exercised
separately. Physical input, assistive technology, Android rendering and iOS
runtime acceptance remain unverified.

### Keep the original role during a folder watch

The actual Move and inverse watcher fixture kept the original implicit
subscription active while its entry generation advanced. It remained retired
even when Undo returned the original spelling. A failed explicit watch
preserved differing unrecorded output and released its subscription. A new
explicit watch then published an independent output, and a fresh implicit
watch published from the restored role. Ctrl-C completed through the same
detachment path during startup and after activation.

An implicit folder watch binds the retained entry identity, generation and
address when it starts. It requires an engine that acknowledges that complete
binding. Capture and publication keep a shared lease on the physical root and
selected role. An explicit watch keeps its literal address. Both paths require
a real project declaration, including when the supplied entry has a different
filename extension. Discovered contributors still require Markdown filenames.

Forty-three focused engine checks passed after tightening private namespace
admission. They cover shared readers, competing publishers, damaged records,
stale role stamps, exact handshake acknowledgement and connected verification
admission. A separate compiler witness checked the explicit entry distinction.
Actual folder and explicit-file watches each published twice from their own
sources. The explicit watch used a literal `.spec` path despite a different
`project.md`. An ordinary `.spec` file refused before publication. The actual
default-reading proof still rejected identical-byte replacement. Move, its
inverse and exclusion against ordinary source writes still require their own
integration evidence.

### Coordinate ordinary Save with a retained reading

Save and guarded source creation now acquire the same root publication lock
before they begin history or write source. A shared generation or verification
reader can continue holding the root while the Save returns a retryable
refusal. Unrelated reads remain available. The initial rule coordinates every
workspace source write through this Save path because even tag-free source can
contribute to construction. Other writer paths and inventory-based narrowing
still need integration.

Fourteen lease and twelve source-history checks passed. The owned engine
fixture held one shared descriptor, read the original file, received Save
refusal before a new history entry, and then published the exact same guarded
request once after release. The existing strict save, creation and deleted
source fixtures also passed. Both native connected-reading fixtures passed
selected and manual-profile verification against the preceding leased watcher
engine. The ordinary Save compatibility branch covers engines without this
retained-role provider. The executed checks used the Unix engine.

### Coordinate grouped replacement and mounted source writes

Find and Replace holds one publication lease for the complete path set. It
checks original physical source guards and current generated claims before it
records the grouped inverse. A blocked request leaves source and history
untouched. The existing browser result popover gives the writer's retry reason.
Merged worktree publication and mounted authored writes use the same lease.
Mounted namespace writes refuse retirement of the retained main or its parent,
including physical aliases and case spellings. Ordinary source admission reads
the retained role only after acquiring the exclusive lease, so a pending role
created during admission refuses the write.

An actual engine fixture held the shared descriptor during Find and Replace,
observed zero replacements and no new history act, then replayed the same
request after release and observed one exact act. The mounted engine checks
confirmed pre-publication Markdown validation and path/link confinement. The
selected-worktree writer saved and undid only the chosen target. Twenty focused
entry and lease checks covered pending-role races and alias refusals. Twelve
source history and nine agent checks passed, along with three focused browser
checks and TypeScript checking. The earlier Find preview carries a content hash.
The physical guard begins when the server plans the replacement.
External filesystem and Git writers have separate publication paths. Entry Move
and its inverse still require guarded publication and host acceptance.

### Guard publication through the raw Files verbs

The raw Files API now shares the same publication lease. Its held path checks
protect the retained main and its aliases. Copies publish from a complete staged
tree. A refused child leaves the visible destination absent. macOS and Linux
rename through an exclusive destination operation.

The real engine refused folder creation under a held reader and accepted the
same request after release. It refused direct removal of the main, retirement
through a hardlink, and a symlink out of the workspace. A refused nested copy
left no partial destination, and an ordinary copy then succeeded. The focused
macOS kernel check confirmed that an occupied destination preserves both files.

### Revert a retained history act through the same source guard

The CLI considers the exact affected paths and the separate subset it would
retire. It may restore the current main's text through the shared lease. Raw
removal still needs a reviewed entry transition. An eligible legacy main is
protected before the first default reading without creating a role record.
Declared generated claims and exact confined source paths are checked again.

The real CLI refused to revert a Find act while a separate reader held the
publication descriptor. It changed neither saved text nor history. The same
command restored the original bytes after release. Reverting its recorded
inverse reapplied the replacement and left both acts readable.

## Checkpoint requested during implementation

The requested checkpoint was committed as `4a612978`. Its focused receipts include
Rust source-ownership guards (18 cases), proposal admission, browser guard/graph/
conflict models and typechecking, Android integration models, and the Apple build.
The Apple graph port builds for macOS and iOS and passes its four local cases;
the actual native graph flow now passes after correcting retained-process query encoding.
Android's corrected complete adapter-source actual flow now passes without skips.

The subsequent safety checkpoint passes eleven publisher cases for physical
equal-byte namespace swaps and rollback refusal for foreign replacements, eight
source-history cases for persisted inverse proof and legacy reading, and five
restoration cases. Eighteen shared graph cases and forty focused browser cases
pass. The complete seven-field browser graph flow passes original-guard and
anonymous-construction refusals while retaining readable relationships, followed
by owned-process completion after root replacement without foreign cache writes.
Both native graph captures now carry named drafts, original guards and typed
unaddressed drafts. Six Apple and ten Android focused graph/capture cases pass;
each native actual flow passes unknown-construction refusal while keeping known
contract text readable, then restores verification after removing that draft.
Agent proposal host application and the other open ledger outcomes remain
required. No broad regression run or status monitoring was performed.

Native source identity now compares UTF-8 bytes across buffer maps, tabs, pending
reads and plan snapshots. Six focused identity cases pass, including mounted
canonical-distinct editors with independent Undo/Redo. The native views now retain
separate undo managers, closing the cross-editor undo failure exposed by that
witness. Thirty-five affected focused cases pass individually, and macOS/iOS
builds pass. Utility-tab name collisions and real names resembling unnamed draft
sentinels remain open; these checks do not certify every filename or physical
input/accessibility acceptance.

The debugger now has an endpoint-attach engine primitive. Its real TCP protocol
witness passes: attach rather than launch, detach with terminateDebuggee false,
external frames retain raw coordinates and source references without editor
revision proof, and mapped run-to/jump/breakpoints refuse. The four focused source
admission cases and two partial/canceled breakpoint transaction cases pass. This establishes the transport safety boundary;
explicit profile selection now reaches the browser through the existing Debug
gesture. Seven shared profile cases and ten browser picker cases pass. The owned
debugpy browser flow passes pause, watch, step and detach while the independently
launched target remains running, with editor identity/selection/geometry/undo and
unsaved text retained. Native attach acceptance, PID-driven provider attachment
and other supported adapters remain open; these receipts establish only the
explicit loopback debugpy endpoint workflow.

## Sealed proposal and recovery implementation checkpoint

Browser proposal review passes 48 focused proposal/resource/API cases,
TypeScript checking and the controlled ACP tool-to-review flow. Its original
send-time editor identities, revisions, guards and full retained roster remain
sealed; before-open edit/Undo, independent handles and unknown owners refuse.
The real flow retains an unrelated unsaved cached editor, supports coordinated
draft Undo/Redo, retries only the exact lost report, and separately publishes and
reverses a guarded namespace change. Android passes nine focused cases and two
separate actual flows: caller-provided review and managed-tool original-context
review. Cancellation/status retention is a focused local HTTP witness, not a
claim of real-engine response-loss injection. Apple proposal review passes eight cases without skips, including the owned
managed-tool flow through draft Apply, coordinated Undo/Redo and literal
namespace Save/recovery. Applying original sealed ranges preserves the same
native editor, caret, geometry and unsaved state; whole-file replacement was
rejected by this acceptance and corrected. The macOS product build passes.
Scripted provider acceptance does not establish a semantic verdict or native
physical-input/accessibility acceptance.

Native Apple endpoint attachment passes four focused cases and the real debugpy
flow: selected profile only, pause/watch/step, unbound source positions and detach
leaving the external target alive. Its iOS product builds; iOS physical input is
unverified. Android endpoint attachment passes the real owned debugpy flow and Kotlin
compilation, preserving editor state and leaving the external target alive
after detach. Other adapter/provider breadth stays open.

Private editor-session recovery now has seven focused engine cases and an owned
HTTP flow covering two window slots, delayed snapshot rejection, exact retry,
engine restart, same-byte source/root recreation and corrupt sibling records.
Snapshots retain literal named/unnamed/generated text, selection/scroll and host
layout without a source save. Browser automatic capture and the existing Source
recovery → Editor sessions floating review now have a real renderer-reload
flow: exact sealed named/unnamed snapshots recover as new draft copies with
layout and scroll beside an independently edited source. That source retains
its editor, selection and Undo; disk is unchanged. Eight focused browser cases
and TypeScript checking pass, including exact lost-reply retries, generated
read-only copies, stale leases and atomic malformed-layout refusal. Utility
instances remain in place. Apple automatic capture and draft/layout/view
restoration now pass two separate native processes against one owned engine: the
capture process exits before restoration starts. Source, unnamed note, selection
and viewport survive; stale selection refuses, existing native editor/Undo
survive and disk is unchanged. Six focused Apple recovery cases pass (the owned case was skipped in that
run), then both actual phases pass without skips. Android passes seven focused
recovery cases and its own two separate JVM phases, preserving TextFieldState,
selection, scroll and Undo beside restored copies. Browser passes nine recovery
cases. Shared raw admission rejects duplicate keys, protocol aliases and deep
material before decoding; all three actual recovery flows pass through that
boundary. The iOS simulator product builds. Private storage retains at most
32 sessions per window and three revisions per session; user-facing management
at capacity remains open. Physical-input/accessibility acceptance remains
unverified.

Deleted-source history passes ten focused history cases and the real strict
save/create/history HTTP flow. Missing remains distinct from empty File, only
confirmed File text can become an unmaterialized draft response, absence ABA and
independent replacement refuse, and no history response materializes a file or
releases a save retry. Browser, macOS and Android now pass owned-engine deletion/restoration flows:
confirmed File text restores into the same editor as an editable unmaterialized
draft, the original save guard is cleared, text Undo/Redo never recreates the
path, and a separate guarded Save As publishes to a new path. Browser passes
eight model cases plus TypeScript checking; Apple passes nine focused cases
(one actual fixture case skipped in that run), and Android passes six focused
cases (one actual fixture case skipped). Both native actual cases then ran
separately with their owned fixtures and passed without skips. The iOS simulator product build also passes after regenerating the Xcode project
to include the newly added proposal source. Physical-input/accessibility
acceptance remains unverified.

## Native resource acquisition safety checkpoint

Apple resource preparation now retains original named and retained owners across
closed-target acquisition. Newly loaded targets must match the first observed
saved bytes and physical guard and have no intervening source intent. Original
edit/Undo, handle replacement, guard changes and unrelated owners refuse before
a preview can renew authority. Provider requery retains this same lease rather
than recapturing later drafts. Twenty-one focused acquisition, editor-routing
and recovery cases pass without skips, including the real-engine resource
save/restoration/redo case; the iOS simulator product builds. Older mock
responses were updated to the already-required guard fields, with an explicit
missing-guard refusal case. Complete Apple conflict composition is separate work.

## Apple complete conflict composition checkpoint, 2026-11-17

Apple repository review now opens a workspace-held complete-group composer inside
its existing floating conflict view. Members, output identities and destinations,
text or immutable Git stages, executable mode and explicit deletion remain part
of one complete review. Draft Apply uses coordinated source Undo/Redo; guarded
Save adopts the confirmed resource receipt, and separate Stage changes only the
reviewed index group. Pending outcomes retain their original request and never
replay Save or Stage. Complete raw review material is admitted and paged away
from the main actor under the original source acquisition lease.

Five owned real-Git/native-editor cases pass without skips: text composition,
deletion, three-member rename, intervening source edits and external index
mutation. Preview retains unsaved text, selection and geometry. Save preserves
the same editor and index, Stage preserves HEAD and unrelated index entries, and
native Undo/Redo restores the pre-Save draft and selection without changing disk
or index. Thirty-one affected conflict, acquisition, resource-editor and recovery cases pass
without skips; the iOS simulator product builds. These checks exposed and corrected whole-file native selection history
and absent-member Missing→Missing patch validation. Native receipt adoption now
uses the existing source draft history on macOS and iOS. Physical input,
assistive technology and iOS device acceptance remain unverified.

## Complete-file provider diagnostic fixes in progress, 2026-11-17

The existing Edit menu and action search now route a complete-file fix request
through the ordinary provider action preview on all simple hosts. Requests carry
the entire source range and `context.only = ["source.fixAll"]`; the shared child
bridge now preserves that filter while supplying captured provider diagnostics.
Hosts retain only explicitly classified complete actions, requery the same family
after unopened-target acquisition, and refuse changed resolved kinds, commands,
disabled actions and obsolete originating revisions. No complete action reports
unavailable rather than claiming diagnostics were fixed. Draft Apply and
coordinated Undo/Redo remain separate from Save and verification.

Four rendered browser checks and TypeScript checking pass. The focused bridge
forwarding witness, eight Apple action/change checks and five Android action,
change and command checks pass. The generic Android focused invocation skips its
owned acceptance; that exact acceptance then runs separately without skips.
Browser, macOS and Android owned-provider flows all pass: the existing command,
full-source bulk filter, lazy resolution, unopened-target synchronization and
requery, complete two-file preview, draft Apply/Undo/Redo and unchanged saved
files. The actual AppKit editor and Android TextFieldState/scroll owners remain
retained. The iOS simulator product builds. Logs for these focused checks and
owned flows are retained under `target/bulk-*` and `target/native-bulk-*`.

This closes the connected standard provider bulk-action path for the exercised
plain source case, not semantic completeness across installed providers or
all literate position mappings. Physical input/accessibility remains unverified.
Original target-owner revision coverage during acquisition and approval is
established by the lease checks below; coordinated history retains its separate
safety follow-up. No full parity is claimed.

## Source-change acquisition and approval leases, 2026-11-17

Provider preparations now extend their original retained-owner lease only for the
explicitly acquired pristine target. Existing editors keep their original
identity, source revision, baseline, save guard, metadata and namespace. Seal the
new target before awaiting language synchronization; an edit followed by equal
text Undo cannot become a fresh baseline. Unexpected new owners and independent
handles at the same path refuse. Cleanup and externally offered source reviews
reuse the same authority instead of comparing text alone. The Apple adapter
reuses resource acquisition leases; browser and Android resource ownership
capture now expose the same bounded extension without renewing old observations.

Twenty-eight browser cases and TypeScript checking pass. Thirty-two Apple
cases pass, including actual resource Save/restoration/redo, plus ten executed
Android action/change/resource cases; the generic Android run skips the owned
provider case, which then passes separately without skips. Browser, macOS and
Android owned-provider bulk flows all pass under the new leases, retaining the
editor and unchanged disk through normal target acquisition and draft Undo/Redo.
The iOS simulator product builds. The source-mutation witness now refuses at
authority preflight rather than the later text-only plan comparison. Focused logs
are retained under `target/proposal-lease-*`.

Coordinated draft history still needs its separate target identity and revision
audit: inverse text equality alone must not authorize a replacement editor. No
full safe-change parity is claimed by this implementation checkpoint.

## Identity-bound coordinated draft history, 2026-11-17

Draft inverses now retain their original editor bindings. A fresh synchronous
Undo/Redo resolves each editor’s unique current source address and validates the
complete current preimage before changing drafts. This follows the same editor
through Save As or a guarded namespace move without authorizing an independent
editor that reused the former path. Lost or ambiguous bindings refuse without
consuming the history step. Ordinary Save and late capability metadata preserve
text history; unlike a held asynchronous proposal, this fresh command acquires
current draft authority and never writes saved files. Provider, agent, conflict
and local-history draft producers use the same model.

Local browser/native cases cover Save metadata, a moved literal source named
`files`, reused old paths, aliases, foreign handles and retry after restoring the
original owner. Browser type checking and 57 focused browser tests pass. Real
engine source-history flows pass on browser, Apple and Android, including deleted
source restoration, native Undo/Redo, guarded Save As and coordinated inverses at
the new address without materializing the old source or changing saved bytes.
Apple exercises a mounted native editor; Android exercises its native controller.
Physical input and accessibility remain unverified. The separate asynchronous
collector authority fixes and provider contributor work have their own pending
integration evidence and are not established by these history checks.

## Complete contributor provider bulk routing, 2026-11-17

Project source actions now fan out over the actual contributor/output graph,
deduplicate repeated origins and retain explicit qualified provider choices.
Engine-issued source proofs bind origin, contributor context and known edited
preimages; per-document revisions reject edit/Undo and close/reopen ABA without
blocking an unrelated preparing source. Exact capture acknowledgements report
processed source, hash, revision and absence of source effects.

Nine focused Rust cases, the existing provider protocol and the new maintained
real-engine bulk protocol pass. The latter exercises prose-start discovery,
four contributor/output actions, repeated origins, cross-fragment refusal,
standalone exclusion, forged metadata, stale and held tokens, dirty contributors
and the complete-action size bound while preserving original source bytes.
Host proof admission is separate work in progress. Coherent unsaved contributor
overlays and reserved-URI project projection remain internal limitations; these
receipts do not establish complete provider parity.

## Source-bound debugger retention and deep frames, 2026-11-17

Breakpoint intentions retain their source context across edits and Save. Archived
intentions remain readable and cannot silently become setters for a different
runtime; fresh local intentions remain available. Complete bounded stack paging
reaches deep frames through existing floating debug controls, without changing
editor space. Source receipts validate current status, coverage and line bounds,
and late capability replies cannot replace a newer source context.

Thirty-nine focused browser debugger cases and focused native receipt/retention
cases pass. Owned real debugpy flows pass on browser, Apple and Android: at least
131 frames, selecting frame100, its depth100 local, conditional breakpoints,
edit/Save/relaunch, archived and current intentions, and preserved editor/source
history and geometry. Apple uses a mounted native text editor; Android uses its
controller and native text state. Exact source-preparation receipt tests also
pass, including plain versus literate acknowledgement rules. Physical input,
assistive technology and device IME remain unverified. Installed LLDB launch and
attach acceptance remains a separate ongoing slice; discovery does not certify it.

## Installed LLDB launch and explicit PID attach, 2026-11-17

Installed adapter discovery now resolves the available Xcode command-line
`lldb-dap` through its platform tool lookup when it is absent from PATH. Nine
focused discovery cases and six owned actual flows pass: launch and explicit PID
attach through browser controls and Apple/Android integrated controllers. Each
native flow executes exactly one acceptance case without a skip. The canonical
compiled C fixture stops at its source breakpoint, evaluates its local value and
preserves saved source; detach permits the owned target's heartbeat to advance.

These receipts certify that compiled canonical C artifact and PID provider path.
They do not certify C++/Rust toolchains, arbitrary external processes, device
attachment, relocated or aliased precompiled symbols, or every adapter family.
Relocated symbols require the provider's source mapping configuration and remain
unverified. Native pointer input, IME and assistive technology remain unexecuted.

## Exact source-bound rename and formatting engine, 2026-11-17

An exact preparation acknowledgement now authorizes custom rename/formatting
queries only through its source hash and opaque revision. The engine checks
origin before dispatch and after every reply, freezes mapping and affected
preimages, and returns one complete stamped no-effect envelope. Edit/Undo and
independent reopen cannot renew that intention. Formatting refuses a failed or
unmappable member instead of returning an apparently successful subset.

Four focused bound-query cases and seven existing bulk integration cases pass.
The maintained real protocol passes exact-capture rename/format, immutable
synthetic resolution, stale and held origin ABA, affected source replacement,
unknown-source acquisition followed by a fresh query, later formatter failure
and whole-format mapping refusal; disk bytes stay unchanged. Existing action and
literate bulk real protocols also pass against the new engine. Browser/native
controller consumption and cross-platform user-flow acceptance are separate
integration work in progress; backend receipts alone do not establish parity.

## Connected collectors and original-root result authority, 2026-11-17

Actual pytest discovery, failed runs, retained results, failed-only reruns and
coverage now reach each simple host. Browser controls and the mounted Apple
coverage renderer exercise the usable path; Android exercises its integrated
bridge/controller. Run completion refreshes retained sessions without manual
reload. Source, session, renderer, selection and exact API-origin guards reject
closed, foreign and equal-byte stale coverage navigation.

Collectors retain their original physical root and record bucket through process
completion. Private report/cache artifacts and descriptor-relative publication
prevent replaced roots or buckets from acquiring this run's evidence. Terminal
original-process outcomes remain readable as historical; they cannot authorize
new work. Missing-history reads create no cache. Tool execution remains ordinary
project-code authority, not a sandbox for arbitrary configured commands.

Focused results: shared coverage10 and type checking pass; backend20 pass with
five explicitly opt-in cases ignored in that group. Separate genuine installed
pytest failed-run/failed-rerun, manual-stop flush and replaced-root/cache cases
each pass. Actual browser/Apple/Android collector flows pass, as do both native
navigation guard matrices and the real HTTP root/bucket/noncreating-read flow.
An installed .NET10.0.401/Coverlet6.0.4 fixture discovers two tests without running
their bodies, retains one failure and real C# line coverage, reruns just that
failure and retains both runs. Its first execution exposed startup diagnostics
being misread as tests; a bounded contiguous-list parser and focused noisy-output
case now pass. Canonical replay of that C# fixture is being retained separately.
Other collector families and physical native coverage interaction remain
unverified unless separately evidenced; these checks are not full test parity.

## Coherent dirty literate project graph, 2026-11-17

Provider graph rebuilding now evaluates one immutable snapshot of retained
source drafts through the authoritative project loader. Dirty contributors and
dirty inventory additions/removals participate in the same graph. A contributor
change refreshes affected retained roots outside its own gate; unrelated roots
remain independent. Invalid or missing contributions retire obsolete analysis,
while source text remains readable. Literal `%` and `#` project filenames use
encoded path segments rather than URI joins that reinterpret their contents.

Three compiler projection models, one backend lifecycle model and seven action
integration cases pass. The real bulk protocol verifies exact dirty source hashes
and fixes, child-observed draft bytes, dynamic output replacement, literal URI
identity, stale/ABA/held refusals and unchanged disk. Existing action and bound
rename/format protocols also pass against the rebuilt engine. Closing a draft
contributor returns analysis to saved source; reopening recovers a retired graph.
Project paths that the canonical compiler rejects (`?`, backslash, non-ASCII)
remain explicit compiler restrictions; these checks do not broaden that language
contract or certify complete external-provider semantic analysis.

## Connected source-proof review across the simple editors, 2026-11-17

Browser, Apple and Android now consume guarded rename/formatting envelopes and
engine-issued action source proofs through their existing preview paths. Every
discovered source, including retained dirty targets, synchronizes before a fresh
query. Frozen snapshots and original editor leases remain fixed; a query cannot
rebind them. Readonly contributor context can be analyzed without acquiring edit
permission. Every actual edited preimage must have a matching engine hash;
ordered resource preimages come from the engine's complete input analysis.
Missing, stale, incomplete or malformed proofs refuse the complete action.

Browser type checking and31 focused preparation/acquisition/proof/controller
cases pass. Both native policy/controller groups and real transport wire cases
pass, including typed no-effect envelopes and equal-byte edit/Undo refusal.
Real plain-source rename/formatting preview, draft Apply and coordinated Undo/Redo
pass on all three hosts; the browser also exercises installed basedpyright's
two-source rename. Real whole-file fixes from a prose caret pass on all hosts,
acquire the copy contributor, query again and preserve the original prose editor
and selection. The browser additionally exercises a retained dirty contributor.
Saved Markdown remains unchanged and generated output directories stay absent.
Apple uses a mounted editor and Android its native state/controller; browser
bulk and ordered resource Save/inverse flows also pass under production proofs.
The regenerated iOS simulator app builds. Native physical input, IME and assistive
technology remain unverified; full external-provider semantic breadth is not
established by controlled-provider receipts.

## Integration ledger, 2026-11-17

This ledger reconciles the required outcome with the implementation inspected on
2026-11-17. The dated paragraphs below retain focused evidence for their own
revisions; they do not collectively certify complete parity. A foundation or a
model witness is not a completed host interaction. Work in progress stays open
until reconstruction, focused safety checks and the connected path pass.

| Required outcome | Established slices and remaining acceptance |
| --- | --- |
| Navigation and intelligence | Semantic queries, bounded workspace acquisition, search and retained navigation trails have focused evidence below. Complete provider indexing and native physical interaction are not established by those receipts. |
| Safe changes, cleanup, generation and templates | Multi-source draft review/undo, cleanup and nested source snippets have focused evidence. Ordered resource publication and saved/draft undo now have connected browser/provider and native controller acceptance. Canonical-entry moves and ambiguous native wire path identities still refuse. A reusable template catalog now has reviewed Apply and coordinated field/history acceptance across the hosts, with rendered browser/macOS and native Android controller evidence. Genuine TypeScript extraction and accessor generation now pass the existing reviewed draft/undo path on all three hosts. Semantic cleanup, broader move/generation families, full snippet transforms/variable contexts, multi-caret templates and mirrored native IME remain open. |
| Diagnostics and contextual fixes | Source-bound Problems, exact provider action resolution and multi-source action review have focused evidence. Standard complete-file provider bulk actions now have connected browser/macOS/Android acceptance and an iOS build. Preparation/acquisition/approval leases now retain original target-owner revisions; literate mapping breadth and provider analysis completeness remain open. |
| Tests and coverage | Source declarations, saved-run admission, discovery/results and source-bound imported coverage have focused evidence. An actual Python collector now retains failed-run coverage and reruns only the failed case, marking the earlier source evidence historical after a source change. Python/pytest source-adjacent Debug now has actual writable physical/authored gutter acceptance on browser, macOS and Android, with the shared Apple path building for iOS. Other test-debug providers remain explicitly unavailable. Collector-to-host pytest acceptance now passes on all three host adapters, and a genuine installed C# collector fixture passes discovery/run/coverage/rerun. Remaining supported collectors and native physical interaction remain open. |
| Review and recovery | Diffs, staging, immutable history, attribution and choosing a conflict stage have focused evidence. Resource recovery publication/adoption and ordered text/resource undo now have actual-engine acceptance. Invalid historical construction drafts and partial-state restoration still refuse. Shared conflict composition/delete/rename publication and complete-group staging pass real-engine checks; browser, Apple and Android now have connected usable paths, including Apple composition/deletion/rename and stale source/index refusal. Ordinary strict Save/Save As, retained local-history previews and draft-only restoration now have actual browser and native controller/editor evidence. Deleted-source history now has connected browser and native controller/editor acceptance, including absent-path preservation and separate guarded publication. Browser reload and native process-restart recovery now have connected draft/layout/view acceptance. User-facing private-session management at capacity and complete resource physical-admission/replay/inverse acceptance remain open. |
| Debugger | Inline launch, profile selection, breakpoints, frames and evaluation have focused slices. Explicit threads, scoped caller locals, immediate running-session controls and pre-launch protocol admission now have actual-engine browser and native controller evidence. Adapter-owned literal source reading passes controlled-provider browser and Apple native flows; Android model/controller checks and its complete actual flow pass. Complete stack paging has owned protocol evidence. Explicit loopback debugpy attach has real-engine browser/macOS/Android acceptance; Source-bound breakpoint retention after edits and deep-stack host acceptance now pass on all three hosts. Installed LLDB canonical C launch/PID attach also pass on those hosts. Other attachment/provider families, toolchain breadth and device acceptance remain open. |
| Connected behavior readings | Captured construction/contributor/output readings have focused evidence. Connected graph/source paging and declared contract/implementation/test/evidence distinctions now pass real-engine browser and Android flows. Apple and Android capture the complete seven-field retained context and pass actual graph/source-reading flows; physical-root and stale-baseline admission have focused browser/engine and native integration evidence. Physical-input acceptance remains unverified; detected tests are not execution evidence. |
| Change impact and relevant verification | Compiler saved/draft output consequences exist. Bounded declared relationships, selected checks/profiles and retained process-only results have real-engine browser and Android acceptance. Apple integration and post-effect root-loss retention now have focused and actual-engine evidence. Retained process-only results never acquire independent verification authority. Physical-input acceptance remains unverified. Undeclared dependencies stay unknown. |
| Agent proposal review | Retained agent conversation and existing guarded source-change mechanisms are foundations. A shared immutable review service and agent-tool preparation now have focused and real-engine no-effect admission checks. Sealed material and cross-platform coherent review/application now have focused and owned managed-provider acceptance, including coordinated draft Undo/Redo and separate resource Save/recovery. Sealed historical readings now survive an engine restart with actual browser/macOS/Android acceptance; they never revive application authority. Provider breadth remains open. |

Physical native input, assistive technology and device IME acceptance remain
unverified unless an explicit executed receipt says otherwise. No row is a claim
of complete JetBrains/ReSharper parity.

## History, connected readings and conflict composition in progress, 2026-11-16

The ordinary-save history service now retains before/after source evidence and
confirmed, failed or uncertain outcomes separately. Restore prepares an editor
draft, preserving native undo; it does not directly publish historical bytes.
The first canonical focused Rust run passed four cases and exposed a root-alias
failure in the fifth. After the fix, all five cases passed, including verified
root aliases, exact retry/no reexecution, stale recreation, tampered evidence and
failed/unknown writer outcomes. Three Apple
model/native-editor cases passed; the live case initially skipped without a
fresh-engine fixture. The subsequent owned live fixture passed all four Apple
cases without skips: actual WorkspaceView/menu, ordinary Save, retained preview,
guarded draft restore and native Undo/Redo preserved source bytes, editor identity,
selection and geometry. Strict save identity, creation and proven resource-move
lineage remain under implementation; this is not complete local-history parity.

Connected readings now assemble bounded compiler, declaration and observation
relationships, preserve unknown impact after invalid/deleted drafts, and admit
explicit relevant checks through existing source-test/profile authority. The
first canonical shared run passed five cases and failed an invalid-construction
fixture that used an unbound prefix; a policy-invalid fixture replaced it and
all six shared cases then passed. Twenty-three browser cases passed. Actual graph/check
execution and the native adapters remain unverified at this checkpoint.

Git composition/delete/rename reviews and complete-group staging are being
implemented separately from source publication. The legacy stage path now
refuses unmerged paths so it cannot bypass review. A conflict-only publisher
policy requests guarded rollback of this attempt's still-matching writes; it
retains a separate rollback report without rewriting forward receipts. Foreign
edits and authority loss must remain untouched and uncertain. Seven focused
publisher tests passed, including own-write rollback, retained rollback evidence,
foreign-edit preservation and rollback authority loss. These new paths
are not yet a verified full conflict workflow and remain open in the ledger.
The initial Android integration passed six focused methods with no skips. Browser
composition is now wired inside the existing floating repository review; seventeen
focused history/conflict controller and strict-save checks plus TypeScript checking
passed, including no-op staging and pending-request retention across close.
Complete real-engine/browser/native acceptance of this combined path is pending.


## Focused integration checkpoint, 2026-11-16

The latest canonical shared build passes seven ordinary-history cases, including
logical resource move/delete/restore lineage and refusal of recreated physical
occupants. Thirteen connected-reading cases pass, including scalar source paging,
source/namespace admission, stale known-output refusal and binary evidence hashing.
The private ordinary-save and resource-lineage records are excluded from generic
resource inverse publication by a focused executed refusal test. These are shared
mechanism receipts; writer-held resource lineage integration remains in progress.

The browser passes six strict ordinary-save, six history and six conflict review
cases. Adapter-source reading passes five model cases, two channel/hook cases and
one literal-rendering case. The two owned DAP transaction cases also pass the new
literal-source, MIME and quota assertions. Actual adapter-source browser acceptance
and native integration are still pending; an owned provider cannot certify vendor
adapter support. The guarded creation route builds, but its end-to-end Save As
acceptance remains pending on each host. No complete-parity row is closed here.

The rebuilt owned-engine HTTP witnesses pass strict source creation and all eight
conflict scenarios. Creation confirms full submitted text directly and replays the
same intention without another inode; changed intentions, foreign workspace,
dirty construction output claims, missing construction guards, saved generated
targets and same-byte independent recreation refuse. The conflict witness covers
text, no-op, delete/rename groups, stale source/index and foreign/staging locks.
Android's actual compose/preview/draft Undo/Redo/Save/separate Stage flow passes,
preserving its native text state and scroll. These do not establish crash recovery
or native physical input acceptance.

The enhanced actual browser graph flow passes full later-fragment paging,
relationship and unrelated-test distinctions, selected pytest and explicit profile
execution, exact-intent replay, stale output/inventory refusal and retained
process-only results. It exercises genuine pytest coverage and JUnit imports
separately from manual, stale and legacy observations. Editor identity, selection,
geometry, unsaved text and Undo/Redo survive the flow. Thirty-one focused browser
graph/API checks and TypeScript checking pass. Android compiles and passes 24
focused history/graph/conflict/receipt/source-reading cases, including six scoped
adapter-source cases; rendered native graph acceptance remains pending.

The owned stdio-provider/browser adapter-source flow passes exact reported frame
reference admission, literal HTML-MIME rendering, stepping while source IO is held,
late-receipt retirement, local provider refusal, Watch/Stop and preserved editor
identity/selection/geometry/undo/unsaved text. This explicitly controlled fixture
proves the connected protocol path; it does not certify a vendor adapter.

## Native source and verification checkpoint, 2026-11-16

Apple's ordinary-history and same-buffer strict Save As flow passes all thirteen
focused cases without skips, including the actual File-menu route, same
NSTextView/identity/geometry, native Undo/Redo and explicit creation absence.
The final iOS simulator build passes. Android's same-buffer Save As passes six
focused cases and its maintained actual-engine menu witness without skips;
its original source and independent old-name occupant remain intact. Browser
Save/Save As/history passes thirty-five focused checks, TypeScript checking and
the maintained actual flow, including native and coordinated workspace Undo/Redo.

Apple adapter-source reading passes eight focused cases without skips and the
owned stdio-provider/native floating-view flow; macOS and iOS builds pass.
Browser literal paging passes seven focused cases after correcting an overlapping
surrogate boundary; commit `75688152` preserves exact consecutive-page text and
removes an empty final page. Commit `b801be9c` contains the earlier shared/browser
source-reading integration. These controlled fixtures do not certify vendor
adapter breadth, native physical input or assistive technology.

The corrected shared ownership boundary passes seventeen focused cases: ordinary
unaddressed notes remain allowed, unaddressed or unselected construction cannot
silently disappear, and a new named contributor is allowed only through the same
complete explicit creation plan with its selected draft hash. The new persistent
root fingerprint survives child edits and distinguishes a recreated same path
across restart. Source text remains independently readable when this capability
is unavailable. Draft original-guard capture, publication protocol2, replay and
inverse proof are subsequent in-progress integration; these tests do not certify
those still-changing paths.

The physical-root graph passes the maintained browser flow plus same-engine
same-byte root replacement refusal and old-reading/new-engine refusal. Android's
actual graph passes exact namespace/runtime/opaque physical identity, full scalar
paging, selected pytest/profile execution, retained process-only observations,
stale edit/Undo refusal and unchanged editor state. Its latest focused Kotlin
batch passes twenty-two methods without skips. Browser graph/API/resource/conflict
checks pass fifty-three methods and TypeScript checking. The graph explanation
fixture still needs its latest source-only legacy-origin wording regenerated;
its previous canonical run passed twelve of thirteen cases.

The proposal service passes its two focused intention/refusal checks, all six
ordinary-entry refusals in the prepared-private-session safety fixture, and the
owned real-engine review/validation/lookup/report witness. Reports remain
explicitly host-reported draft transitions and cannot become saved source or test
evidence. Sealed envelope and original editor-guard integration are not yet
certified by those earlier API receipts, and no host Apply path is claimed here.

## Debugger transition and retained process safety, 2026-11-16

Commit `e67dd250` replaces reusable terminal counters with fallible random
process identities before spawn. Its selected real owned-process restart witness
passed; a retained Run/verification link cannot acquire the replacement process.

Commit `4b3d7d5c` serializes temporary Run To delivery, live user breakpoint
replacement and stop cleanup. Pending cleanup is retained before adapter effects,
including partial delivery or cancellation, and cleanup failure is explicit.
The two owned stdio-adapter transaction witnesses passed with concurrent root and
contributor replacements, repeated Run To refusal and cleanup preserving the new
user wishes. The same cases read all 123 reported frames across pages and refuse
repeated frame identities or premature page exhaustion. Live deep-stack and
concurrent provider/host acceptance remain separate open checks; these protocol
witnesses do not establish attach or adapter-owned source support.

## Guarded debugger integration in progress, 2026-11-16

Protocol 3 adds exact source hashes and intent receipts for breakpoint changes,
stop/thread/frame admission for motion and observation, and explicit retirement
of a pause when an effect cannot confirm its replacement. Child expansion must
use a reference actually observed in the selected frame. These are shared
provider boundaries, not proof that every host has adopted them. Two actual
shared debugpy witnesses passed scoped nested reading, forged-reference refusal,
stale motion refusal and source-bound breakpoint refusal without changing source.

Physical workspace discovery now has a read-only inventory separate from a
constructed portable output tree. Its exact filenames must not weaken generated
output publication rules. Compilation and two focused inventory checks passed, preserving distinct Unicode
identities and generated-output refusal. The URI resolution witness passed exact
percent decoding, malformed/traversal refusal and symlink confinement. Session
identities include a per-registry random namespace; its restart/late-cleanup
witness passed. Android’s isolated actual startup, worker/main recursive reading
and literal-filename breakpoint witnesses passed without skips. Apple’s isolated actual startup, worker/main recursive reading and literal-source
condition/breakpoint paths also passed without skips under full Xcode. The first
Command Line Tools build failed; the later SwiftBuild run emitted no requested
XML, so the runner now requires exact per-case XCTest start/pass receipts. Neither
exit status nor test discovery alone is accepted.

Captured project addresses now use one shared encode/decode boundary for language
and debugger providers. Two context checks and one bidirectional mapping/hash
check passed literal percent/hash filenames, malformed address refusal and
foreign captured identity refusal. A subsequent actual debugpy witness caught a
missed primary-breakpoint decoder, which has been corrected. Its rerun passed
canonical literal-source launch, bound breakpoint, exact frame URI/hash, scoped
`41` evaluation, conditional binding and Stop without changing host sources or
materializing the context in the host workspace. Connected browser
captured-literal acceptance subsequently passed through the real patch gutter:
encoded launch, exact source bytes/hash, distinct percent-looking sibling,
`marker = 41`, generated-source navigation, Stop and unchanged host bytes.

A failed multi-file breakpoint replacement may have partially affected the
adapter. The engine suspends accepted receipts during delivery and restores them
only after whole-operation success. Two focused transition checks passed for
failure/cancellation and successful replacement. Desired DAP sets now commit only
after successful delivery, retaining possibly touched file addresses for cleanup.
Two fake stdio-adapter Session tests passed partial delivery failure and
cancellation across root/new-source replacements, retained accepted wishes,
explicit clearing of possibly touched files and no failed-wish retry. The same
checks now assert that temporary run-to delivery preserves the user's root wishes.
Android passed its new failure-authority case in a sixteen-case model/protocol
run with four opt-in live cases skipped; that run preserves the prior nineteen
case no-skip positive integration receipt rather than replacing it.

Browser protocol 3 passed a 194-case focused checkpoint and subsequent captured
reading/launch rail checks, with TypeScript checking. Actual ordinary and physical
literal-source flows passed thread/frame selection, recursive `answer = 41`,
floating Run To/Jump and retained editor geometry/source. The full resource
move/swap/delete/recreate/undo/redo/Stop/guarded Save As flow passed exact editor,
selection, scroll, undo and geometry checks. Two browser-local actual flows passed
with engine HTTP/WebSocket access blocked, including offline stepping/watches,
source rebind, explicit errors, shorter edited documents and Stop. The captured-literal patch-gutter flow also passed launch, watch, correct paused
generated-source reading and Stop without host writes. A byte-preserving captured
reading check keeps BOM text readable without inventing an unknown frame position.

Android's final focused set passed all twenty cases without skips, including four
actual-engine cases and the correlated failure-authority check. Paused resource
move, guarded menu Undo and fresh conditional-breakpoint binding retained editor
identity, selection, scroll and undo. The isolated fixture included the
literal-name sibling throughout. Source-test Debug remains an open outcome.
Apple's final thirty-eight-case focused run passed thirty-seven executed cases
with one independent stock-SwiftUI accessibility baseline skip. All four actual
flows passed: startup/condition/step/run-to/jump, literal physical filename,
worker/main recursive reading, and paused resource move/menu Undo/menu Redo with
independent same-byte name recreation. Runtime identity, native editor, selection,
geometry and undo remained retained; no replacement source gained position or
breakpoint authority. The iOS build passed separately. Physical input and assistive technology
remain unverified. Both owned native-runner commands also executed their four
actual controller/transport cases without skips. Concurrent Run To and live
breakpoint replacement still need serialized-state acceptance; executed movement
flows are sequential. Firefox/WebKit and supported native adapter breadth are not
established by Python/debugpy and local Chromium receipts. This entry is a
progress ledger, not complete debugger parity.

## Reviewed reusable source-template catalog, 2026-11-16

The [shared template service](../mechanisms/source-template-service.md) reads
built-in and bounded data-only `.hickory/templates.json` definitions. Catalog and
plan replies bind the canonical source path, exact UTF-8 hash, UTF-16 selection,
workspace and definition fingerprint. Literal template lines follow captured
indentation; variables retain their own text. The shared snippet mechanism owns
field expansion, explicit unsupported syntax and output limits. Plans neither
execute commands nor save files.

Six focused shared checks passed for Unicode/scalar selection, indentation,
workspace definitions, changed catalogs, unsupported syntax and foreign catalog
links. A real-engine HTTP witness passed repeat-plan identity, catalog revocation,
generated-file/physical-alias/escaped-path refusals and unchanged authored bytes.
The focused read-gate witness passed with both template routes. Six shared snippet
checks passed after fixing empty-choice metadata amplification and a malformed
choice terminator that previously consumed a character. The real HTTP witness
also refused an all-empty mirrored choice definition before emitting a plan.
Every alternative now reserves storage, even when empty. Session creation and
nested mirroring reserve their complete field/choice cost before copying; undo
context admission rejects oversized entries without evicting older valid context.
The browser cache also charges empty snapshots and bounds its token count.

Browser TypeScript checking and 145 focused template/snippet/history/command/API
and affected saved-test checks passed. Its actual visual flow passed Edit menu
and action search, unchanged preview, held-Apply catalog revocation with no draft
change, reviewed Apply, Unicode and selected-text variables, mirrors, choices,
field traversal and exact undo/redo. Stale source, rendered read-only editor,
disabled command and engine refusal passed. Editor identity, geometry, unsaved
source and disk stayed retained. A real saved-test Debug regression passed after
sharing the raw protocol codec and correcting the driver's DOM editability probe.

Android passes full Kotlin compilation and 24 focused expansion/session/history/
interaction/template methods with zero skips. Its actual engine/controller flow
passed shared command dispatch, fresh-plan Apply, native fields, menu undo/redo,
manifest revocation and generated-source refusal. Editor state, selection, scroll
and saved bytes remain retained. That witness does not render a Compose window or
verify geometry, physical pointer/IME or accessibility.

Apple passes macOS and iOS simulator builds and 21 focused methods with zero
skips. The fresh-engine actual WorkspaceView flow passed unchanged preview,
reviewed Apply, linked native typing, undo/redo, retained editor and geometry,
provider revocation and read-only refusal. Native emoji/combining-text field and
history checks passed. Physical keyboard/pointer, VoiceOver and device IME remain
unexecuted. Full snippet transforms, embedded-language/complete variable contexts,
mirrored native IME and multi-caret templates remain required open work. These
receipts establish the catalog and guarded common-subset behavior, not full IDE
parity.

## Guarded saved source-test Debug, 2026-11-16

The shared service now plans a saved test selection without launching a process,
then recomputes candidate, contributor hashes and provider admission at the
separate `start_test` operation. Ordinary Debug cannot silently accept that test
payload. Started binds the exact author, selection and checked contributors;
retained process outcomes do not become invented test-case verdicts. The initial
provider is Python/pytest; other language providers explicitly remain unavailable.

Four admission/wire checks and two provider-plan checks passed. The imported-author
mapping check passed, including literal coordinates and retained entry ownership.
The actual debugpy witness first exposed rejection of a present null program for
module launch. The shared DAP boundary now omits that explicit clearing sentinel;
its focused check passes. After adding the fixture's explicit project-environment
declaration, the actual shared witness passed both physical and authored inputs:
selected body breakpoint, scoped `41` evaluation, Continue through exit zero,
sibling sentinel absent, unchanged authors, no authored output leaked to the host,
and finished registry reaping. Four admission checks and the class-qualified
candidate check also passed on that revision. This checks the shared path, not a
substitute for each host's connected acceptance.

The browser passes TypeScript checking and 110 focused checks, including stale
choice/binding guards and a process finishing before its retained row arrives.
Actual physical and authored flows passed through the existing source gutter and
floating test surface: explicit selected-family Debug, user body breakpoint,
locals/watch `41`, retained debugger link, Stop, then a fresh selected-family run
through exit zero. The sibling sentinel stayed absent. Editor identity, geometry,
source and disk bytes stayed unchanged; the authored path also checked selection
and scroll through the retained link. An initial authored attempt caught and
removed an unrelated ordinary-profile selection dependency; the dedicated admitted
test plan owns this launch path. No ordinary Debug fallback is permitted.
Apple's Swift and iOS builds pass, and both actual writable physical/authored
paths pass through the real WorkspaceView and source-test gutter. User-only
breakpoints, locals/watch `41`, Stop, retained links and a fresh zero-breakpoint
selected-family run through completion pass. The sibling sentinel remains absent;
real native insertion/Undo before execution and Redo afterwards preserve the same
editor, caret and geometry. Its canonical numeric-protocol guard remains included
in the focused model checks. Android passes nine focused guard checks,
full Kotlin compilation and both actual writable physical/authored gutter flows.
Its index-to-gutter adapter opens the existing floating test model; explicit
selected-family Debug reaches the user breakpoint, evaluates `41`, completes with
exit zero and leaves the sibling sentinel absent. Native insertion/Undo before
execution and retained Redo afterwards pass with the same editor, selection,
scroll and text. Closing/reopening a popover retires pending admission permanently;
active-process outcomes remain retained independently of visibility. Public
native-smoke preparation caught an Android SDK directory spelling mismatch; its
exact major/minor-layout correction passed a focused check. Six preparation checks
and an explicitly executed cold/reuse/failure witness passed. Both public native
smoke commands then executed exactly four actual debugger flows without skips;
the workflow is committed separately as `b1f7f8aa`.
Native undo acceptance subsequently exposed a fixture collision: the authored
example claimed the nominal physical file as its generated output. The shared
Rust witness already uses separate plain/authored fixtures; the host runner is
being corrected to construct a distinct authored target and require that its
physical file has no generated owner. Earlier host physical runtime receipts do
not establish writable physical-editor acceptance. The browser reran both inputs against the corrected fixture and passed, explicitly
checking actual CodeMirror readOnly false/editable true before launch, with exact
selection and scroll preservation on both. Apple reran both corrected-fixture inputs and passed, including actual writable
native insertion and retained undo/redo. Android also passed both corrected inputs
through its real index-to-gutter adapter.
These receipts do not establish complete cross-platform testing parity, other
providers or physical-input/accessibility acceptance. The supported saved-test
Debug implementation and its paired evidence are committed as `ef463247`.

## Saved test-debug choice guard, 2026-11-16

The [Android choice policy](test-debug-android.md) captures a defensive copy of
source-test facts and the originating editor, observation and workspace identity.
A one-shot take reuses saved-run source admission and requires an explicit
`test_debug_protocol = 1` capability. Six focused policy checks and full Kotlin
compilation passed. This is a foundation: no test-debug provider, wire launch or
usable source-adjacent Debug action is supplied by this model alone.

## Debugger threads and scoped observations, 2026-11-15

The shared engine now lists actual adapter threads and admits selection against
an exact stopped generation. Frame-local, expression and child reads carry their
request identity and selected scope. Stack and variable failures stay explicit;
they do not become successful empty observations. Signed adapter identities,
including zero, remain opaque. Missing line positions never become the first
source line. Adapter-owned sources do not borrow authored-file navigation merely
because a path is present. Captured mapping hashes identify launch/construction
bytes, not a proof that a running process executed those bytes.

Focused executed evidence: three mapping-snapshot checks; strict thread and frame
decoder checks; and two actual debugpy engine witnesses. The live witnesses
selected a Python worker and main thread, read caller locals and evaluated in the
selected scope, refused stale stops and foreign frames without source writes,
and stopped a running launch after changing its breakpoints. Delivery stays
inside the session operation gate. An initial worker blocked inside a native
wait produced an adapter evaluation refusal; that remains a readable optional
failure rather than a claim of successful locals. The browser passed TypeScript checking and 114 focused cases, then an actual
Python/debugpy flow through profile resolution, named worker/main selection,
caller navigation, selected-frame locals with canonical `str` type information,
and distinct computed watches. The source editor, its dimensions and disk bytes
remained unchanged. Android passed eleven focused cases without skips, including
two actual-engine debugpy flows. Apple passed its two actual-engine flows,
including exact worker-local `41` and main-local `7` evaluation. Its focused
controller checks and iOS simulator build passed separately. These native tests
exercise controllers and actual transport, not touch, pointer or VoiceOver.
Native physical-input and assistive-technology acceptance remain unverified;
the macOS renderer witness skipped only after an independent stock SwiftUI
Button also proved inaccessible in this XCTest host. Its product reachability
assertions remain unexecuted. The final native suite had nineteen executed passes,
including both live flows, and that one explicit harness-capability skip.

At that revision, additional integration work remained explicit: resource-renaming an active debug
source must hand off the retained session identity; stale raw movement requests
need engine stop/thread admission; source references need adapter-backed reading; normalized stdout projection still needs connected browser verification; literal URI path handling needs shared normalization;
and the current
stack request is bounded to fifty frames. These are not completed by the thread
model or by the live shared-engine witnesses. The 2026-11-16 entry above records
which of those items subsequently passed; source references and stack paging
remain open. Older engines now refuse before
Debug launch effects through catalog protocol admission; ordinary Run remains
independent. The Python manual-collection stop/flush witness also passed,
retaining uncovered branches without inventing a manual verdict.

## Semantic intelligence extension, 2026-11-14

The extracted TypeScript, Swift and Kotlin intelligence modules now normalize nested outlines, flat workspace symbols, declarations and caller/callee items into named source locations. The existing menus and floating language surfaces expose these nine queries; the later type-hierarchy extension adds supertypes and subtypes. Position reads flush pending source before asking a provider. Six focused browser semantic witnesses and three focused model/snapshot witnesses on each native implementation pass. TypeScript and native compilation, plus the iOS simulator build, are checked separately. The real Python browser flow exercises cross-file definitions/usages, exact columns, file outlines, workspace symbols, incoming/outgoing calls, stationary geometry and unchanged disk bytes. Native pointer and live-provider acceptance remain unverified by these model checks.

## Navigation history

Every simple platform exposes Navigate back and Navigate forward through its existing menus. A workspace-owned, bounded trail records departure carets and destination positions, deduplicates identical visits and retires the forward branch after a new visit. History reuses buffers and cached editors, including unsaved text, rather than reopening or saving files. The full workspace file index remains separate from the directory currently shown in Files. Positions that no longer fit the current text report that a fresh navigation is needed; historical carets are navigation aids, not execution or save evidence.

Focused TypeScript, Swift and Kotlin trail witnesses cover caret retention, branch pruning, duplicate visits and the 256-position bound. The real-provider browser flow checks Back/Forward between symbol locations, exact carets and retained DOM editor identity, along with unchanged source files and editor geometry. Native model checks and the iOS build do not establish native pointer acceptance.

## Workspace Problems

The existing status bar and Edit menu open a floating, filterable Problems list on every simple platform. Retained workspace buffers contribute language-provider messages independently of mounted panes. The list is paged; revealing or paging it cannot resize code. Next/previous navigation uses the same source-location index and wraps over current locations. Unversioned and stale messages remain readable and offer a file open, while precise navigation requires a captured matching source version. Source expectations survive asynchronous destination opening and are checked again before a precise reveal. The count is reported provider information, never an execution or verification verdict.

The browser language-session binding is a separate workspace module shared by the editor and Problems; replacing either renderer does not replace transport or source-snapshot rules. Native diagnostic values retain their decoded locations so status rendering does not repeatedly scan whole documents. Full-project analysis and provider analysis scope are separate from the retained-buffer report index.

Verification: four focused browser component/protocol checks passed for source/version admission, stale/unversioned retention, wraparound and editor retention. Two focused diagnostic/Problems checks passed on each native implementation; TypeScript checking and the iOS simulator build passed. The real Python browser `--problems-only` flow passed for reported messages, status/menu access, stationary geometry, retained draft and unchanged disk bytes. Native pointer/popover acceptance remains unverified.


## Bounded workspace knowledge acquisition

Workspace analysis is available from the existing Edit menu and status bar on browser, macOS, iOS and Android. Its floating report distinguishes attempted files, readable sources, provider-returned symbol lists, unavailable or stale reads, and generated/unclassified exclusions. Two workers each await a source load and outline before taking another file. Stop prevents further admission while current reads drain; another run cannot multiply that limit. Closing the report does not stop the run. It reads retained drafts without changing the active pane, caret or layout and does not save them. Background diagnostics join the existing source-bound Problems index independently; an empty outline or absent diagnostic is not a verification verdict.

The model accepts source/outline adapters rather than owning UI or transport. Native workspace connections and browser retained sessions are shared with existing editors. Background outline reads depend on source and provider identity, not the caret or language-tools popup. A changed source rejects its reply. Source text remains available even if a provider read fails. Changing native workspace servers stops the previous scheduler and creates a fresh report model. Report rows become stale when their captured source no longer matches the retained buffer.

Focused scheduler witnesses on all three implementations cover the two-read bound, drain-before-restart, stale draft rejection, independent failure and continued work. Native wire witnesses also exercise caret-independent outline reads and changed-source refusal. The real Python browser `--workspace-analysis-only` flow passed for unopened-source diagnostics joining Problems, retained active draft/editor dimensions, status/menu report access and unchanged disk bytes. TypeScript checking and the iOS simulator build passed. These checks do not establish provider completeness or native pointer acceptance.

## Search Everywhere and shared command adapters

Every simple frontend exposes Search Everywhere through its existing Edit menu and primary-P keyboard shortcut. The floating view searches the full workspace file index plus retained drafts, source-current provider symbols and shared command descriptors. Known LSP kinds distinguish types and members; other symbols remain explicitly general. File search includes generated outputs with their read-only ownership. Symbol results require a matching retained source and carry that snapshot through the existing guarded reveal path. Changing a draft retires its previous symbols immediately. The view shows a bounded 200-result page and the full local match count, without claiming complete provider indexing. Read workspace invokes the same two-worker acquisition used by the analysis report.

File, Edit and View/Panes menus now consume the same command values as action search. Shared descriptors name operations; scene/activity/browser callbacks adapt existing operations. Availability uses a fresh context getter at invocation, including source ownership, saving, dirty state, history and pane requirements. Unsupported platform adapters stay discoverable with a reason. Native window actions belong to the scene/activity and are not stored in the workspace. Native menu buttons and keyboard shortcuts are separate from the search view so replacing search does not rewrite menu rendering. Existing editor operations retain their authoritative guards and preview paths.

Browser acceptance `--search-everywhere-only` passed against real Python language services for menu and keyboard entry, disabled action reasons, type/member search, exact declaration columns, action-to-Problems transition, stale-type retirement, retained editor identity/drafts/geometry and unchanged disk. Focused search/command models passed on all three implementations. A real macOS AppKit/SwiftUI view witness passed Down/Return selection and retained-draft opening. The iOS simulator build and Android compilation passed separately; native device pointer, IME and assistive-technology acceptance remain unverified.

## Captured construction readings

View → Construction reading opens a floating reader on browser, macOS, iOS and Android. An explicit Markdown entry compares its saved construction with retained authored drafts over pinned compiler inputs. Optional contributor/output focus restricts the reading. Added, deleted and modified outputs retain byte hashes, file kind and executable metadata. Saved/draft previews describe constructed trees; the reader does not materialize them or save source. A failed baseline or draft refuses instead of reporting an empty impact.

Compiler contribution ranges retain source/output byte offsets and distinguish equal byte slices from unconfirmed reported origins. The response bounds output and test pages at 300, origins at 100 per output, and text previews at 2,000 characters, with explicit truncation. Detected test declarations are separate from execution/coverage evidence, whose status is not queried. These are construction relationships, not a complete semantic dependency or verification graph. Declared fragment IDs and tag kinds identify the smallest containing named body without inferring a contract or proof. Source links require service-supplied scalar UTF-16 positions, matching captured hashes, exact file identity, and current workspace/reader/link lifetimes. This reader does not authorize edits or clamp stale positions into a document.

Focused reader lifetime checks passed on browser, Apple and Android; TypeScript checking and the iOS simulator build passed. The real-engine browser witness checked retained draft consequences, both compiler preview sides, editor identity and dimensions, stale saved-source link refusal, exact matching-draft contributor navigation, undo/redo, unchanged saved sources and absent generated materialization. Native physical input, IME and accessibility acceptance remain unverified.

## Existing and extended editor behavior

Given a reachable existing Hickory engine, each frontend exposes the same four pane kinds: files, text document, workspace agent and terminal. Plain and literate source open as text. Generated outputs open read-only in this slice. File and pane commands remain in menus or tab strips; editor content has no permanent feature toolbar.

The workspace agent presents previous prompts and replies as read-only literate
Markdown documents, and the unsent message as an editable literate document with
Send beneath it. Keep the draft editor alive through provider changes and pane
moves. Explicit ChatGPT, Claude Code and Grok subscription choices and model IDs
are sent to the Hickory Python agent; only its Monty interpreter admits generated
code. Provider-native tools and API billing fallback are unavailable in this
surface. Effort is explicit for ChatGPT and Claude; Grok currently manages its own
effort, so its control remains at Default. A model may reject an unsupported effort.
Subscriptions use the native CLI's existing sign-in on the workspace server.
Current provider process confinement requires macOS; clients on other platforms
can connect to that server. No unconfined fallback is permitted. Completed answers
refresh through durable history polling; token-by-token subscription streaming is
not claimed. Drafts are retained in the workspace session, not saved as files.

Terminal panes accept typing, paste, cursor keys, Ctrl+C and Ctrl+D directly in the terminal emulator, without a separate command field. ANSI colors, cursor positioning and full-screen terminal modes are interpreted by xterm. The PTY receives measured cell dimensions when the pane resizes. Moving or hiding a pane detaches its renderer without closing the backend session; reattachment receives retained replay before live output. Transport failure is visible, and uncertain input is never automatically retried.

Custom chrome uses short paint-only motion: 80 ms feedback, 100 ms tab
background emphasis and 120 ms floating-search reveals, with the same
`(0, 0, .2, 1)` curve in CSS, SwiftUI and Compose. Drag-preview geometry follows
the current target immediately while its first appearance fades in. Native
controls, menus, popovers and sheets retain their platform feedback. Custom
panels disappear immediately on dismissal. Reduced-motion preferences disable
added browser/Apple motion; Compose uses the system animator duration scale.
Animation never delays commands, saves, editor content or capability attachment,
and never interpolates pane dimensions or gutter anchors. The focused
`python3 scripts/test-simple-browser.py --search-only` witness checks live normal
and reduced-motion search reveals, stationary editor geometry, unchanged text,
retained editor identity and undo of an unsaved draft. Native visual timing,
keyboard/IME and accessibility behavior still require device acceptance.

Motion verification on 2026-11-13: the focused real-engine browser search flow
passed under normal and reduced motion, including unchanged editor identity,
geometry and text throughout rendered frames, unsaved-draft undo and exact regex
replacement bytes on disk. Browser TypeScript checking and the simple production
bundle passed. macOS built with full Xcode, shared Apple source type-checked for
the iOS simulator, and Android `:app:compileDebugKotlin` passed offline. These are
compilation receipts for native targets, not visual, IME or accessibility
acceptance and not an iOS app or Android APK packaging receipt.

Embedded diagrams, tables and equations use their content size; small content
must not reserve a large empty preview rectangle. Oversized content may fit the
available width or offer nested scrolling. Scrolling keeps its recipient across
gestures while the pointer rests: content moving under the cursor cannot capture
the next gesture. Actual pointer movement allows a new recipient.

Each embedded block offers a right-gutter source icon that toggles source and
preview in both directions without changing saved text or the retained editor.
Graph nodes may declare an `href`, and Mermaid flowchart nodes may use
`click node "relative/path.md"`. Double-click opens a local file relative to the
containing document; file extensions do not restrict it. External schemes,
absolute paths and paths escaping the workspace are rejected. Rendering remains
strict and never enables authored script callbacks.

Verification scope: focused browser controls and native layout/path witnesses
exercise this behavior; native pointer routing and iOS visual interaction still
need device acceptance. This change does not claim Android preview parity.

Semantic navigation uses a provider query and a captured source position, independently of pane layout. All simple platforms offer Go to definition, Find usages, Go to type definition and Find implementations through their existing menus or Language tools. Results float over the editor without reserving content space. Location and LocationLink replies normalize to a list; unsupported identities remain visible but cannot open arbitrary filesystem or network targets. Workspace-authored `hick:///` paths open through the workspace, retaining existing buffers. A source, selection or connection mismatch refuses an obsolete result. Same-file navigation works on unsaved text; cross-file navigation refuses a dirty target whose provider coordinates cannot be established. These reads never save or generate files. Missing provider support reports an unavailable/empty response without blocking text.

Verification, 2026-11-14: twenty-one focused browser component/model and File-menu witnesses passed, including obsolete-response refusal, destination-editor ownership, one-shot column reveals, save safety and math rendering safety. TypeScript checking and the simple production bundle passed. The real-provider `python3 scripts/test-simple-browser.py --semantic-navigation-only` flow passed against an isolated workspace: definition opened another file at its exact column, usages retained the importer after switching tabs, editor dimensions stayed fixed and disk bytes stayed unchanged. Fresh providers can require repeated explicit reads while indexing; the flow retries only those reads through the visible menu.

Two focused Swift and two Kotlin location/snapshot witnesses passed. The macOS package, iOS simulator app and Android source/test compilation passed. Native pointer/popover behavior and live native semantic-provider interaction remain device acceptance work; type-definition and implementation results depend on provider support and have not received this Python system-flow verification. Only authored `hick:///` targets open in this slice; captured-project and generated-output URI targets remain visible unsupported locations. LSP targets carry no target-content hash: origin snapshots and dirty-target checks guard navigation, but externally changed saved target files can require another query. This is semantic navigation, not the complete IDE feature backlog.

Browser language sessions now belong to cached workspace buffers rather than mounted panes, matching the native workspace lifetime. Hiding a file must not close its provider document and erase the semantic knowledge needed by usages. Session publication and editor attachment remain independent of immediate readable text.

Opening text does not wait for rendering, provenance, debugging or language services. Pane splitting, resizing and moving retain document buffer identities and unsent agent messages. Drag a tab to the outer twenty percent of pane content to split left, right, above or below; the shaded half previews its destination. The middle appends the dragged tab to that existing group and activates it without adding a split. Corners select left/right. Center drops preview the whole target pane. A successful drop moves the existing tab into a fresh equal split and focuses it; any empty group closes automatically and its surviving sibling fills the freed rectangle. Closing the last tab in a group does the same, except that closing the final document retains its empty pane and tab strip beside Files. Reopening a file uses that document pane, including files whose cached buffers survived closing. If all tabs are closed, one empty root remains available. An explicit Split command may create a temporary empty destination until the next close or move. Stale or unknown payloads do not create splits. Drag a tab onto another tab to insert before it, or onto unused strip space to append, within a group or across panes. Drag indicators overlay tab edges without resizing the strip. Self-drops and obsolete source/destination identities leave membership unchanged. Both drag and menu movement are available. Dirty document tabs refuse to close.

A plain save includes its original content hash. A literate save includes its original source as both room and disk expectation. Failed saves retain edits. A successful reply marks only the submitted snapshot saved; typing during that request remains dirty. Save and agent mutations are never automatically replayed by these frontends after transport failure.

The File menu exposes Open File, Open Directory, New Tab, New Window, Save, Save As, Save All and Refresh. Paths refer to the connected workspace; an empty directory path restores its root. New Tab uses Ctrl+N / Cmd+N in the browser and desktop simple UIs and retains unnamed text until Save As gives it an unused path. Save As refuses existing destinations and keeps the original edits on failure. Save All saves named writable documents and requests a destination for unnamed drafts. Refresh observes files without replacing open buffers. New Window gives the new surface its own editor state.

Tab detects indentation from nonblank leading whitespace in an initial sample (up to 200 lines and 16,384 characters): predominant tabs remain literal tabs; spaces use the common indentation width, falling back to four spaces when there is no usable evidence. At a partial space indent, Tab advances to the next stop. No parser or language server is required. Right-gutter actions have recognizable pictograms and accessible names. A draggable right divider adds icon columns. Actions wrap into available rows until the next populated action row; they never displace that row. If those slots run out, reserve one for ⋯. It opens a compact floating grid containing only the excess icons. An icon may open its own detail panel; those details are never the overflow contents.

Editor settings offer independent Ctrl+Left/Right and Alt/Option+Left/Right
policies: Platform default, Whole identifier and Word pieces. Defaults leave native
bindings intact. Whole identifiers include underscores; pieces separate underscore
names, camelCase, acronym endings and number runs. Shift extends and contracts a
selection with the same boundaries. Preferences persist locally and apply to open
document editors, including read-only documents; terminal, agent and form fields
retain their own bindings. OS-reserved shortcuts may need OS configuration before
they reach the application. Custom navigation follows logical source order.

All simple editors offer **Enter indentation** with persisted **Smart (language
rules)** (default), **Match current line**, and **None** modes. Changes apply to open
editors. Smart uses the same bundled registry on browser, macOS, iOS and Android:
brace/parenthesis/bracket continuations, Python suites and completed return/raise/
break/continue/pass statements, keyword-delimited blocks (Ruby, Lua, shell, Elixir,
Julia, SQL and Visual Basic), layout rules (Scala, Haskell and F#), YAML mappings and
literal bodies, and HTML/XML tags. Brace rules cover C/C++, C#, Objective-C, Java,
JavaScript/TypeScript, Go, Rust, Kotlin, Swift, Dart, PHP, Perl, Groovy, R, Zig, Nix,
Terraform and CSS-family files; JSON/TOML containers also indent. Named Markdown
fences and Hickory file bodies select their language without indenting prose as
code. This is local editing policy, not a claim of complete language parsing.

Smart ignores recognized strings/comments, retains unfinished expression indent,
and returns to an opening delimiter's indentation when the expression closes.
Enter between an opening and an existing matching closer creates an indented body
line and aligns the retained closer; it does not insert a new brace or quote. Copy
retains the leading prefix before the caret; None inserts only a newline. All
modes preserve the local LF/CRLF convention and replace the selected source through
one native/CodeMirror undo transaction. Protected source, read-only state and IME
composition retain their existing guards. Android applies Enter policy in the input
transaction, covering software keyboards. Smart browser multi-cursor Enter copies each prefix to avoid overlapping
closer proposals. Native multi-cursor input retains its existing broadcast path; custom smart proposals target a single native selection.

The scan is bounded to 65,536 preceding UTF-16 units and 4,096 per line; unknown
languages, unavailable rule resources, unfinished literals and out-of-budget
contexts carry indentation. Raw strings, heredocs, preprocessor branches, embedded
language expressions and JSX tag syntax are not full-parser guarantees; use a
formatter or manual indentation when the heuristic is insufficient. No typing
waits for a language service or a later asynchronous correction. Backspace in a
space-only prefix removes through the preceding inferred stop. There is no
automatic trailing-whitespace cleanup or save-time formatting.

Every text document offers **Edit → Find and replace** and Cmd/Ctrl-F. Search supports next/previous navigation, case sensitivity, regex mode and replace-one/all; regex replacements accept `$1`, `$&` and `$$`. Controls float above text without resizing it. Native matching runs in the background and uses UTF-16 source positions. Source, query and permission checks reject stale replacements. Replace-all is one undo operation; read-only text remains searchable. Native searches skip zero-width matches and refuse replacement when more than 10,000 nonempty matches would truncate the result. Advanced regex syntax follows the platform's engine.

Every text document offers Fold/Unfold on indented logical blocks through a dedicated left-hand gutter: − collapses an expanded block and + expands a collapsed block. Tabs count as four indentation columns; blank lines do not create fold headers. Folds hide the body while retaining its original source bytes and logical line numbers. Nested folds and read-only folding are supported. Native edits retire fold positions; an edit touching hidden text first reveals it without applying that edit. Navigation to a hidden native debugger location unfolds the document. Browser folding also uses available server ranges and the standard CodeMirror fold shortcuts. Markdown files additionally offer ATX and Setext heading folds through the next heading of equal or higher rank, including nested sections and the final section. Heading-like text inside fenced code and literal Hickory bodies does not create section folds.

Every text document has a left line-number gutter, starting at 1, including empty documents and the empty line after a final newline. Numbers follow logical lines through edits, scrolling and wrapping; wrapped continuations have no additional number. The number gutter is separate from the right contextual-action rail. Clicking a supported nonblank code line toggles a pending breakpoint, including Python bodies inside Hickory file elements. Markdown prose and pathless fenced examples do not offer breakpoints. A path-bearing code fence has file-block ownership. An explicit patch fence is a project construction: supported destination headers offer launch, and added nonblank code rows offer pending breakpoints and source-mapped language tools. Context/deleted rows remain review text. Ordinary diff fences stay inert examples. The engine validates surviving provenance and adapters bind or refuse stops. DAP binding/relocation/refusal remains authoritative; source candidacy does not establish that an adapter can stop there.

While paused on unchanged source, tint the next execution line independently of the selection. Show watched expression answers in muted trailing whitespace at the selected frame where they were evaluated. Resume, frame changes and source edits retire those positional values. Right-gutter pictograms expose hover hints on pointer-capable surfaces. Container setup bodies receive shell syntax colors on each simple UI.

Stdout and stderr from an ordinary debugger run appear in read-only source-adjacent terminals at debugger response boundaries. Preserve an adapter-supplied file and line through source mapping for placement; absent locations remain unknown and output falls back to the producing program block. Never attribute output to the most recent paused line merely because no output location was supplied. Buffering is bounded to 256 chunks, each capped at 16,384 UTF-8 bytes, with a truncation notice for a capped chunk. This is ordinary interactive output, not private prediction capture or a replayed execution receipt.

Wheel input retains its document recipient across successive gestures while the pointer stays still. A table or diagram moving beneath that pointer must not steal the next wheel gesture. Moving the pointer allows the nested surface to become the wheel recipient. Touch scrolling retains platform behavior.

Every document has an interactive fixed-width minimap. Language services attach independently after readable text. Browser, Apple and Android use the existing engine LSP channel for semantic colors and completions. Native **Edit → Language tools** (Ctrl-Space) exposes diagnostics, hover, signatures and inlay information. Source, selection and connection guards reject obsolete replies. Native diagnostics without a matching version remain readable messages without navigation positions. Unsupported capabilities fail independently. Edits retain semantic colors on unchanged regions while background refresh runs; native UTF-16 caches resize a token containing a single-line edit and discard spans crossed by a replacement. Pending, null and failed semantic reads do not blank the cache. Cached language-tool results remain readable, but stale diagnostic navigation and completion edits are disabled and still guarded at application time. Fresh replies replace only their own capability. The browser already maps semantic decorations through editor transactions. Code-model bulk indexing can supply compiler facts for static TypeScript exports, but is not a cross-language replacement for this live LSP cache. Later completion and semantic-change extensions below provide retained resolution, import edits, navigation, rename and code actions. Snippet field interaction remains required.

All editors support multiple selections. Browser uses its established rectangular and multi-cursor commands. Native Find offers Select all matches; Edit offers column selection and additional cursors above/below, bounded to 256 selections. Apple broadcasts ordinary typing through one native undo transaction. Android broadcasts physical-keyboard typing and offers explicit Apply to selections for touch input. IME composition retires secondary native selections, preserving normal platform composition.

Background presentation preserves the visible source position. Enter may reveal its new caret, but parsing and formatting must not jump the viewport. Native tables retain mapped presentation while their next parse is pending, including edits before or after a grid and ordinary cell-value changes. Presentation mapping does not grant debugger ownership or authorize stale source edits.

Inline presentation preserves editable source. Browser reuses the existing rich table, equation and recorded-output widgets inside the editor. All simple editors replace Markdown prose table lines with an inline spreadsheet grid: lettered columns, numbered rows and editable source-backed cells. Apple embeds the established spreadsheet in a local WKWebView inside AppKit and UIKit; Android embeds the same local bundle above the retained field’s output transformation. Native cells remain available until that bundle is ready, and an active native cell edit finishes before presentation switches. A terminated rendering capability restores readable native cells. Grids support up/down/left/right arrow navigation. Moving from one cell input to another places a collapsed caret, never select-all: up/down retain a preferred visual horizontal position across consecutive moves, temporarily clamping in short/empty cells and restoring the original column on a longer cell. Explicit horizontal caret movement, edits or pointer placement reset that goal; left enters the previous cell at its end, and right enters the next at its beginning. Left/right move the caret normally within a cell and leave native selection/modifier gestures intact. They cross cells only from a collapsed caret at the matching text boundary; an outer table edge must not trap interior caret movement. Native grids apply on Return/Done, arrow navigation or focus loss. A bordered Edit source text button reveals only that table; Edit tables as grids restores the presentation without rebuilding the editor or its undo stack. CSV declarations and standalone CSV files use the same grid with CSV quoting retained on edits. Apple source carets stay hidden within a rendered table’s retained range. Source Delete/typing at that hidden anchor leaves the grid and source unchanged; Edit source is explicit. Native undo/redo and edits inside the grid remain available. Diagram blocks render inline on every Simple UI. Native previews support graph JSON and Mermaid using bundled local assets, reserve a stable preview rectangle, and offer explicit source mode. Invalid/unknown diagrams retain editable source with a preview error. Obsolete asynchronous diagram results cannot replace current rendering. Browser graph styles are installed inside each rendered shadow root. Apple dollar formulas render within their surrounding prose, with following text flowing after the measured formula. Apple math elements replace their complete syntax with one compact row sized to the typeset content. Their source remains readable until the local typesetter is ready; obsolete source or retired-view measurements cannot hide current text. Tapping a rendered equation or moving a caret into its retained syntax reveals source, and explicit source mode can be returned to rendering. Presentation preserves source bytes, selection and native undo. Android equations retain a typeset preview in reserved whitespace below their source paragraph. Equation floating editors remain available on both native frontends. Recorded execution output appears in a selectable, read-only terminal beneath its producing block. Authored expect bodies use the same terminal surface, including expectations nested in exec elements. Both interpret ANSI styling and carriage returns without providing stdin. Channel and location metadata never become visible prefixes inside that stream. An expected-output terminal does not by itself claim that a run occurred or matched. Applying a contextual edit verifies its source snapshot and uses native undo. Other native Markdown retains its styled source; this is table replacement, not full native Markdown WYSIWYG. Known fenced-code language hints receive syntax colors even without a target path or LSP. Unknown languages remain literal; a pathless code example does not acquire debugger or workspace ownership. Native fallback colors are lexical previews and yield to semantic server spans. Formatting scans are bounded to 200,000 UTF-16 units; larger text stays readable. Recorded outputs require an exact document/source hash and are never replayed automatically. Conversation Markdown formatting remains browser-only.

Simple inline tables and standalone CSV files share the established grid’s rectangular selection, Shift extension/contraction, formula cell/range picking, fill, clipboard handling, row/column insertion and removal, and independent viewport height. The formula syntax and evaluator remain the existing language-backed engine, including dependency ordering and trace debugging. A declared table language remains authoritative. Plain CSV and Markdown offer an explicit formula-language choice for the current grid session; that choice does not alter file syntax or silently persist a language declaration. Formula and parser failures retain readable data. Native formula bridges expose only language discovery, evaluation and tracing, not arbitrary HTTP requests. Each native write names an exact source revision and passes the host’s live permission/composition checks and undo transaction. Height is presentation state and never enters CSV bytes. Pending structural parses retain the table’s range without granting old cell ranges write authority. Empty CSV files can start with the grid’s first cell, and Markdown formatting comments remain part of their table’s replacement range.

Named tables use document-local identifier IDs. Every simple frontend resolves
`Prices!B2`, locked addresses and cell ranges through the existing formula
backends. Whole-column selectors exclude declared headers. Cell dependencies
order formula evaluation; ranges and columns remain selectors. Declared Python
`tables=` inputs use that same graph and expose computed values alongside raw
source. A block's `table=` attribute publishes one complete read-only table.
Generated equals-prefixed strings remain data, and truncated previews cannot
silently become inputs. Observing formulas never executes producing blocks.
Only current saved recordings supply generated values. Refreshing recorded
output recalculates dependent grids without replacing their editors. The small
[tables example](../../examples/simple-ui/tables.md) is included in every newly
prepared default project. See the [cell graph and execution boundary](../mechanisms/crates/hick-formula/src/workbook.rs.md)
for construction and focused witnesses. Browser acceptance uses the real engine
and the shared local native bundle; physical iOS/Android gestures, IME and
accessibility still require device acceptance.
The focused `python3 scripts/test-simple-browser.py --table-dataflow-only` witness
passed against the real engine: observing formulas did not create the producer's
file witness; explicit runs published totals of 11 and 13; refresh updated the
existing grid; and the local native bundle evaluated qualified references and
kept generated equals-prefixed text read-only. Eighteen focused browser component
and reference-rewrite tests also passed. These do not establish physical device
acceptance.


All simple frontends attach the existing engine DAP debugger inline for plain files and Hickory file blocks. Pressing a gutter launch icon immediately starts the application matching that file block (or the viewed plain file), without asking the author to select it again. A sole discovered application also starts automatically; a picker remains only for multiple applications when none matches the clicked source. Source freshness and profile fingerprints still guard startup. Launch icons occupy a fixed right column; active step, continue, stop, stack, watch and evaluation controls follow the paused line. Sessions survive tab switching and movement. Unsaved text refuses launch and hides positional values until saved. The browser reuses the existing editor debugger; Apple shares a SwiftUI rail across macOS and iOS; Android uses a Compose rail. Native controllers retain one active workspace session. Native launch icons appear at supported file declarations, equation icons at equations, output icons at receipts, and session controls at the paused line. Detail panels contain stack, condition, evaluate and watch controls; adapter capabilities gate optional actions. Native locals describe the stopped top frame; selecting a caller clears those locals and evaluates watches in that selected frame. Launch uses the clicked declaration’s file path and saved-source expectation; the engine validates the program and source mapping. Native source candidates are computed in the background snapshot, avoiding a document rescan for every displayed icon. A missing Apple math resource bundle falls back to readable equation text instead of trapping in SwiftPM’s generated resource accessor.

Configurable Smart Enter, 2026-11-11: the same 87 source/caret fixture cases
passed in TypeScript, Swift and Kotlin, including common brace/keyword/layout/tag
families, Python statement completion, comments/strings, paired existing closers,
CRLF, tabs, modes and embedded code. Four focused Swift tests passed, covering the
shared cases, actual AppKit mode changes and one-step undo, retained whitespace
undo and hidden-table refusal. Two focused browser tests passed, covering the
shared cases and mounted-editor settings/Enter/undo. Complete cached Android
source/tests compiled and its focused fixture witness passed; TypeScript and iOS
simulator source type checking passed. The focused real-keyboard system flow
(`--smart-enter-only`) was attempted; Chromium aborted before rendering during
macOS Mach-port registration. Native physical keyboards/IME and packaged Android/
iOS builds remain device acceptance work. These are bounded heuristics, not full
language parsing or measured latency guarantees.

Editor navigation and whitespace, 2026-11-11: five focused Swift witnesses
passed, including actual AppKit key events in read-only text, Shift anchor reversal,
native Enter/Backspace undo and hidden-table deletion refusal. Three focused browser
witnesses passed for whole/piece boundaries, independent settings persisted through
visible controls and applied to an existing editor, and detected Tab stops. Unicode
witnesses include decomposed accents and supplementary characters. TypeScript
checking, complete cached Android source/test compilation, its focused identifier
and whitespace witness, and iOS simulator source type checking passed. The focused
real-browser flow (`--editor-navigation-only`) was attempted but Chromium aborted
before rendering during host startup. Full Xcode simulator build exited 134 before
compilation; Gradle dependency resolution failed at `dl.google.com` DNS. Native
physical-key/IME acceptance, Android packaging and the real-browser flow remain
unverified. These checks establish neither a measured performance improvement nor
full system-test equivalence across platforms.

Preferred vertical-column follow-up, 2026-11-10: seven focused browser checks and four Swift checks passed, including a real AppKit long → short → long traversal, reversing direction, and resetting the goal after horizontal motion or an edit. Browser checks also reset on pointer placement in a short cell. The focused Kotlin stored-goal witness passed; Android source/tests compiled and iOS simulator source type checking passed. The Python browser flow now uses real keyboard input to assert 8 → 1 → 8 through a short CSV cell, then a horizontal reset; it remains unexecuted on this host because of the previously observed Chromium startup failure. Phone pointer/selection callbacks still need device acceptance.

Horizontal-caret correction, 2026-11-10: native handlers had consumed left/right even inside cell text, and an outer table edge could turn that into a no-op. They now leave interior caret movement and nonempty selections to native editing, handing off only a collapsed caret at the matching boundary. Three focused Swift checks passed, including actual AppKit text commands that move the caret, collapse a selection and request boundary exits. Six focused browser checks passed, including interior/modifier ownership, cell entry and formula pointing; TypeScript checking passed. The Kotlin boundary-policy check passed, complete Android source/tests compiled, and iOS simulator source type checking passed. Phone hardware-keyboard gestures still require device acceptance.

Caret-navigation follow-up, 2026-11-10: twenty-five focused browser checks passed (twenty-four table editing/navigation checks and the CSV integration check), along with TypeScript checking. Five focused Swift checks passed, including an actual AppKit field focus/selection transfer between grid cells using proportional-width text. The focused Kotlin caret-policy witness passed; complete Android source/tests compiled, and iOS simulator source type checking passed. Vertical entry uses the source caret's measured horizontal position and a collapsed target selection; horizontal entry uses the target edge. Empty/short cells clamp, and Apple/browser caret matching retains whole graphemes. Browser formula point mode and Shift-arrow range selection remain separate. The real-browser driver now asserts collapsed CSV entry; physical keypresses, IME and native device gestures still need platform acceptance.

Interaction follow-up, 2026-11-10: 64 focused browser model/component checks passed, including CSV keyboard editing/quoting/undo and Markdown source-to-grid restoration. Thirteen Swift checks passed and the opt-in live DAP check skipped: these include actual AppKit grid mounting, independent source reveal, guarded CSV editing/undo, missing-resource lookup, diff styles, code-line context and the shared close scenarios. Four focused Kotlin checks passed for CSV contexts, diff styling, table projection and layout scenarios. Apple source type-checked for iOS simulator; complete Android source/tests compiled with the installed Kotlin/Compose compiler. These are not APK or native gesture acceptance results. Browser system acceptance was extended for CSV save, arrow navigation, grid restoration and gutter resizing, but Chromium still aborts at macOS Mach-port registration before rendering. Native physical-key focus, divider dragging, popover geometry and live DAP remain device acceptance work; debugpy is absent from the shell Python environment. The workbook's highlighted diff records a change already present in its executable file block; it is review text rather than a separately executable project patch.

Verification for the 2026-11-10 file-editing extension: three focused browser component/command witnesses passed (indent detection and stops, the complete File menu with new-draft Save All, and Save As refusal retaining edits). Apple and Android indentation witnesses passed, including selection-preserving indentation. macOS and complete cached Android source compilation passed. Real-browser acceptance was attempted with `--file-editing-only`, but Chromium aborted before rendering during macOS Mach-port registration. Device input, visual icon layout and new-window interaction remain acceptance work; compilation is not evidence for those interactions.

These are the initial frontend requirements, not full parity with the old frontend. Full native WYSIWYG, capability menus, provenance, additional agent backends, settings, durable layout and crash recovery are outside this slice. A framework build or shared model fixture cannot establish native input-method, accessibility, gesture or latency equivalence.

---

Spreadsheet extension, 2026-11-12: partially verified. Fifty focused browser component witnesses passed (13 Markdown/native-host, 33 formula/pointing/height, four simple grid/source-editor cases). `python3 scripts/test-simple-browser.py --spreadsheet-only` passed against the real engine and the exact locally bundled native table surface: mouse/Shift selection and contraction, real touch selection and formula-range picking, row/column add/remove, real Python evaluation, independent height changes, and first edits in empty CSV. Eight focused Apple witnesses passed, including actual WKWebView loading, stale-revision refusal, repeated edits while an old shorter parse range is offered, native undo, empty CSV and Markdown format-comment ranges. Five focused Android witnesses and complete Android source/test compilation passed. Complete Apple source type-checking passed for the iOS simulator target. The local spreadsheet assets build from the locked web dependencies without a download. UIKit/Android device gestures, IME, native clipboard and accessibility have not received live system acceptance; the shared browser surface does not establish those native platform outcomes. Existing fill and formatting mechanisms are reused, not exhaustively reverified by this flow.

Scroll stability, 2026-11-12: three focused Swift checks passed for repeated Enter with/without tables, retained table geometry through parsing, source/undo and grid edits. The Kotlin pending-parse table witness passed. Working-source reconstruction and ownership audit passed. iOS/Android device scrolling and IME acceptance remain unverified.

Language-cache extension, 2026-11-09: focused AppKit witnesses cover cached colors during real edits/undo, retained language-tool results after a closed-channel refresh failure, UTF-16 shifts and stale completion refusal. Kotlin witnesses cover UTF-16 cache mapping and diagnostic-version safety. The browser attachment witness holds semantic replies while typing and confirms cached colors, selection and undo. iOS source type checking and full Android source/test compilation are build evidence only. The new `python3 scripts/test-simple-browser.py --language-cache-only` system flow requires the real Python server; this host aborts Chromium at Mach-port registration before rendering. A shared native system-automation adapter for this flow is still missing, so device/IME and live-provider cache acceptance remain unverified. No broad regression suite was run.

Apple table-anchor guard, 2026-11-09: two focused AppKit witnesses passed for repeated backward/forward Delete at table boundaries and within hidden cells, selection deletion, unchanged source/grid ownership, explicit source editing, ordinary prose deletion and cell-edit undo after the grid is rebuilt. The native macOS app built and iOS simulator source type checking passed. Both native caret implementations use the current presentation ranges; grid cell inputs keep ordinary carets. The UI tool returned `cgWindowNotFound` for rebuilt previews, so the caret suppression has not received live visual acceptance on this host. iOS device behavior remains unverified.

Inline spreadsheet replacement, 2026-11-09: three focused browser witnesses passed for plain/literate table grids, cell edit/undo and readable editable source during parser loading. Three focused Swift witnesses passed for scanner boundaries, CRLF/Unicode table ranges, native grid geometry, automatically refreshed line rails, source reveal and guarded edit/undo. Complete Android source/tests compiled with cached Kotlin/Compose dependencies; the focused table projection and guarded source-edit/undo witnesses passed. iOS simulator source type checking and browser TypeScript checking passed. Manual inspection showed the actual workbook grid in both the Apple preview and Safari after a fresh Vite start. The new real-engine `--table-only` witness checks grid edits, exact saved Markdown, undo and source reveal, but Chromium aborted at host startup before that flow could run. Android/iOS device gestures, keyboard/IME behavior and screen-reader acceptance remain unverified. Native CSV declarations continue to use contextual cell editors.

Apple equation flow repair, 2026-11-12: `InlineEquation` in `InlineMarkup.swift`
now keeps full syntax ranges separately from editable bodies. `presentMath` and
`FoldLayout` in `NativeEditor.swift` use measured control-glyph slots, while
`MathCoordinator` in `InlineMath.swift` rejects stale renderer revisions and the
editor rejects measurements from retired views. Five focused Swift witnesses
passed: `testEquationRangesSeparateEditableBodiesFromPresentation`,
`testNativeMathFlowsWithinProseAndCompactsBlocks`, `testNativeEquationTypesetter`,
`testNativePreviewFitsInlineViewport` and
`testNativeOutputTerminalUsesBundledEmulator`. They check actual AppKit/WebKit
layout, wrapping bounds, compact block spacing, source reveal, guarded contextual
edit/undo and source retirement; these do not replace a framework-independent
system flow or device visual acceptance. Full UIKit source type checking passed
for the iOS simulator; UIKit rendering and device gestures remain unverified.

Apple bundled preview repair, 2026-11-12: four focused Swift witnesses passed under full Xcode: installed Mac resource lookup, live KaTeX rendering and unsafe-link refusal, output-terminal rendering, and inline viewport bounds for graph JSON, nested fractions and asynchronous Mermaid. The graph also retains its aspect ratio after resizing. The Mac development launcher now installs the SwiftPM resource bundle before signing. These are packaging and WebKit checks, not complete document visual acceptance or iOS device acceptance.

Source-adjacent presentation, 2026-11-12: seven focused Rust session witnesses passed, including reported and missing output locations. Targeted web debugger, tooltip and rich-presentation witnesses passed; TypeScript type checking passed. Four focused Apple witnesses passed, including a live WebKit equation render and URL-safety check, source-revision output guards and Unicode receipt placement. Two focused Android model witnesses passed for shell/prose isolation and Unicode receipt placement. macOS and iOS builds passed. Full native document visual acceptance, wheel/touch gestures and cross-platform system acceptance remain unverified by these model and renderer checks.

Inline/editor extension, 2026-11-08: 78 focused browser witnesses passed, including an actual inline table-cell edit/undo and local math HTML/URL safety. Seven focused AppKit/model witnesses passed for styled source identity, guarded table edits, multiple-selection undo, diagnostic versions and source positions. Live Python engine witnesses passed on Apple and Android for semantic colors, hover and completion. iOS source type checking passed. Complete Android source/test compilation and five focused Android inline/language/multi-selection witnesses passed; these checks do not establish emulator/device gesture, IME or accessibility acceptance. XcodeGen includes the resource folder for both Apple targets. The opt-in native WebKit witness could not load its page in this host test process, so native equation visual acceptance remains unverified. Earlier browser startup and APK assembly limitations below remain unresolved. No speed or memory advantage has been measured.

Search/editing extension, 2026-11-08: browser witnesses exercise the actual search panel, regex replace-all, undo, literal backslashes and read-only controls. AppKit witnesses exercise a real attached text view, atomic replace-all undo and stale-source refusal; Swift and Kotlin model witnesses check Unicode offsets, captures, invalid patterns and the native match limit. Kotlin's actual TextFieldState witness checks replacement undo, stale-source refusal and read-only searching. iOS simulator source type checking passed. Native completions now share the guarded source-edit path. The focused real-engine `--search-only` flow checks source preservation, saved bytes, stationary editor geometry and undo, but this host aborts Chromium before rendering. Native search gestures, panel sizing with software keyboards, accessibility and UIKit undo acceptance remain unverified.

Folding extension, 2026-11-08: the browser component witnesses cover logical regions, right-menu fold/unfold, unchanged source/dirty state and retained undo. Three focused AppKit tests cover glyph collapse, restored geometry, hidden-edit refusal, indentation offsets and line numbers. iOS simulator source type checking passed. Complete Android source and tests compiled with the installed Kotlin/Compose compiler and cached platform dependencies; the focused fold-projection witness passed. These do not establish UIKit/Compose gesture, IME or visual acceptance. The real-engine `--fold-only` flow is implemented, but Chromium aborts at macOS bootstrap registration before rendering. Android APK assembly is blocked by Maven DNS/cache dependencies, and `xcodebuild` aborts before compilation on this host.

Inline DAP extension, 2026-11-07: six focused browser component checks passed. Three focused Swift controller checks and one Kotlin controller check passed; their opt-in live-adapter witnesses were skipped without a configured server. macOS built, iOS simulator source type checking passed and the complete Android source and focused tests compiled with the cached Kotlin/Compose compiler. The controller checks cover session identity, breakpoint relocation/conditions, cross-file navigation, launch source expectations, evaluation frames and stopping a pending launch. These are not native gesture or visual acceptance receipts. The real-engine `--debug-only` flow was attempted, but Chromium aborted at macOS Mach-port registration before rendering. A Python adapter installation attempt failed at PyPI DNS resolution; live adapter and native UI acceptance remain unverified.

Browser Markdown extension, 2026-11-07: three focused component witnesses passed for nested emphasis, headings, literal code, HTML safety, source preservation and undo. TypeScript checking passed. The focused real-engine browser witness (`python3 scripts/test-simple-browser.py --markdown-only`) could start its engine and Vite server, but Chromium aborted during macOS Mach-port registration before rendering. Visual browser acceptance and real provider-message acceptance remain unverified; native formatting is outside this extension.

Edge docking extension, 2026-11-07: all 27 shared layout scenarios and fifteen edge-coordinate cases passed in TypeScript, Swift and Kotlin. All 58 focused browser witnesses passed, including all four edge drops with editor identity, unsaved text, selection and undo retained. Ten Swift witnesses, iOS simulator type checking, Android source compilation and the browser build passed. Live browser acceptance remains blocked by host Chromium startup, and native gesture/provider acceptance remains unverified.

Tab reordering extension, 2026-11-07: all eleven shared layout scenarios passed in TypeScript, Swift and Kotlin. All 23 focused browser witnesses passed, including workspace drag/drop insertion with editor identity, unsaved text and undo retained. Ten Swift witnesses, iOS simulator type checking, Android source compilation and the browser build passed. Real browser acceptance remains blocked at Chromium host startup; native gesture acceptance remains unverified.

Language UI extension, 2026-11-07: fifteen browser frontend witnesses passed, including controlled JSON-RPC colors/completion attached after typing with editor identity, selection and undo preserved. Ten Swift witnesses passed. The Apple source type-checked for the simulator and the complete Android source compiled with the installed Kotlin/Compose compiler. The browser bundle built. Live browser acceptance still fails during Chromium host startup, and Android APK assembly remains blocked by Maven DNS. Native UI acceptance remains unverified.

Interactive terminal replacement, 2026-11-13: the focused real-engine browser witness (`python3 scripts/test-simple-browser.py --terminal-only`) exercises direct input, shell cursor editing, a raw-mode key without Enter, ANSI alternate-screen output, pane resize, a partial command across pane movement, Ctrl+C and Ctrl+D. Its local native-renderer protocol check exercises Unicode output split into individual bytes, ANSI color, input/control-key encoding, cursor availability and measured resize messages. TypeScript checking and the focused session-identity witness passed. Source/output checks and ownership audit passed before final verification. Apple and Android host transport/lifecycle code was inspected, but their platform builds, native WebKit/WebView keyboard/IME interaction and device runtime acceptance have not been executed on this Linux host. The shared HTML browser check does not establish native platform acceptance.

Literate subscription agent extension, 2026-11-13: the focused real-engine
browser witness (`python3 scripts/test-simple-browser.py --agent-only`) passed
for retained history, read-only previous messages, exact subscription/model/effort
and Python-backend request, failed-send draft retention and editor identity through
pane movement. Six selected browser document/session witnesses passed. Two Monty
worker witnesses passed, including refusal of filesystem access, subprocess imports,
`eval`, excessive allocation and unbounded loops. The real confined-worker broker
witness `model_cannot_invoke_shell_or_verify_through_the_broker` passed. Two runtime
profile unit witnesses passed. Claude authentication/result admission unit checks
passed. These are focused checks, not broad regressions or a live subscription
acceptance run. Apple and Android tests were added but cannot be executed here:
this Linux checkout has no Swift/Xcode or Android SDK/compiler toolchain. Native
layout, keyboard/IME, sign-in and live provider acceptance remain unverified.

Earlier baseline verification: 2026-11-07, Codex. Partially verified. Eleven TypeScript witnesses, seven Swift witnesses and four shared Kotlin pane scenarios passed. The complete Apple source type-checked for an iOS simulator target. A native-browser walkthrough observed a plain-file save and undo after splitting and moving its tab. A real PTY check observed Unicode input, screen output and resize, then closed its test session.

The Python browser acceptance suite could start its isolated real engine but Chromium launch aborted while registering a macOS bootstrap service; the complete UI suite is not a passing receipt. Xcode project builds also aborted in this host before compiling, although Swift package compilation and simulator-target type checking succeeded. The complete Android source compiled with Android Studio’s bundled Kotlin and Compose compiler. APK assembly remains unverified: Gradle is available from Studio’s cache, but shell DNS failures prevented downloading remaining Maven artifacts, including Kotlin build tools 2.2.10. The build is now aligned with the installed SDK platform 37.0 and Android Studio JBR 25 through AGP 9.1.1 and Gradle 9.3.1. Native input methods, accessibility and gestures remain UI acceptance work. No speed or memory advantage has been measured.

Implementation and focused tests are constructed in the [frontend index](index.md). The [developer guide](../../docs/developers/simple-frontends.md) records commands and verification limits.

## Find saved workspace text

The existing Edit menu opens a floating workspace search on browser, Apple and Android. Literal, regular-expression, match-case and whole-word queries use the engine's exhaustive find API, including generated-file ownership. Results enumerate occurrences, paginate presentation and state when the server cap makes the result incomplete. Search is explicitly over saved workspace text: unsaved buffers remain intact, and precise navigation requires their text to match the result's source hash. Byte columns are converted at Unicode scalar boundaries to editor UTF-16 positions. Query changes and reconnects retire pending answers; search itself never saves or edits text.

The focused workspace-search witnesses passed: two browser component/model checks, two Swift checks, two Kotlin checks, TypeScript compilation, the iOS simulator build, and a real-engine browser flow covering regex options, Unicode reveal, stable editor geometry, retained drafts and stale-source refusal. Native pointer interaction and whole-project provider analysis are not established by these checks.

## Project refactoring proposals remain whole

The shared language bridge projects each writable range through exact compiler provenance. A range crossing contributors or discontinuous segments refuses the proposal. All mapped targets are retained; repeated identical copies collapse to one authored replacement, and conflicting overlaps refuse. An unmappable rename target returns a visible error. Each code action is independent, so an incomplete action is disabled with its reason while its valid siblings remain available. Provider resolve data and command arguments remain opaque. Virtual version counters do not become authored-buffer versions.

The existing single-editor application path now refuses multi-document changes before touching source. The cross-platform workspace preview and coordinated application are separate required work; mapping safety alone does not establish that those interfaces exist. Four focused Rust provenance/proposal tests, four browser language-action tests, TypeScript checking, reconstruction and ownership audit passed for this boundary.

## Captured changes are independent of their presentation

Browser, Apple and Android have a pure source-change planner: typed before/after patches, UTF-16 replacements, complete-target validation and exact inverse patches. Unknown targets, read-only/saving sources, unsupported filesystem operations, unproven document versions, invalid scalar boundaries and overlapping replacements refuse the whole proposal. Applying or undoing a plan requires all captured source texts to match. Shared data cases exercise two-document Unicode edits and refusal boundaries on every platform. These are model guarantees; a planner alone is not a refactoring UI or proof of coordinated editor application. The shared cases passed in 13 focused browser checks and one case-driven check each on Swift and Kotlin, alongside TypeScript checking, reconstruction and ownership audit.

## Review provider changes from the existing Edit menu

Rename symbol, Refactor and quick fixes, and Preview formatting prepare source-change previews on browser, Apple and Android. Each asks the installed provider, retains its exact action choices and checks source, selection and provider identity again before preview and application. Every retained target must be writable, unchanged and outside active composition. Rename first reads provider references and refuses a proposal that omits any authored source identity in that answer. The preview reports the observed reference-target count while leaving project analysis scope explicitly unestablished; coherence between two provider answers is not exhaustive coverage. Missing/unmapped targets are visible refusals. Lazy actions resolve through the captured provider before preview. Provider commands and resource operations remain unsupported here and never apply an edit portion alone.

The floating preview pages complete before/after text by source file. Applying updates drafts; it neither saves files nor claims generated output or test verification. Existing editors remain retained. Apple commits model snapshots before reflecting undo-aware native text edits; Android commits source changes in one Compose mutable snapshot; browser prepares all CodeMirror transactions before committing. Undo workspace change and Redo workspace change in the same menu use the exact inverse patches across the batch and refuse if any later local edit, save state or permission makes the snapshot obsolete. History retains at most 32 batches per connected workspace.

Every affected source must be retained before application. Rename and provider actions now acquire unopened targets as described below, before obtaining a fresh proposal. Provider support bounds offered refactorings and code-generation actions, and observed references do not establish exhaustive project analysis. Native compilation/model checks do not establish pointer, IME or accessibility acceptance on devices. The focused change checks passed: 14 browser model/editor cases, two Swift cases (including actual macOS editor undo), two Kotlin cases, TypeScript checking, iOS simulator compilation, reconstruction and ownership audit. The real-provider browser witness runs against the freshly built engine and confirms two-file rename preview/application, stationary geometry, unchanged disk, coordinated undo/redo and later-edit refusal. It also exposed the child initialization bug: an explicit workspace-folder identity is required to avoid basedpyright single-file rename mode; the shared bridge now supplies it.

## Reports carry source evidence across shared connections

The language bridge requests versioned child diagnostics and rejects explicit obsolete child versions. For a matching bridge-owned child version, each translated report carries an engine source digest beside its original provider data. Project contributors use their compiler-owned source bytes. Direct parser/lint reports bind their known authored input. A merged publication never fabricates a client version or relabels an old contributor with a newer hash. Unversioned child messages retain null evidence and do not become diagnostic context for code actions.

Each simple UI compares available source with the outer engine evidence before enabling source positions. A coincidentally equal window counter cannot override contradictory hash evidence. Browser messages appear before asynchronous digest checking; obsolete hash replies cannot attach to edited or switched source. Each report retains its own evidence: an unknown contribution cannot disable a current contribution beside it. Unknown evidence leaves messages readable without granting positions. This is revision evidence for reported source, not a project-analysis completeness claim or proof that tests passed. Two focused Rust notification-path tests, four browser proof/Problems checks, three source-review compatibility checks, one hash/counter safety case each on Swift and Kotlin, TypeScript checking, iOS compilation, reconstruction and ownership audit passed. The rebuilt-engine real-provider Problems witness confirms current hash-bound locations, precise Next problem navigation, unchanged editor geometry, retained edits and unchanged disk. Native background reply gating is implemented; device-level IME, pointer and accessibility acceptance remains separate.

## Acquire workspace source independently of panes

The file tree exposes optional language-routing facts from the engine without parsing each file. Background discovery consumes those facts on all simple platforms, counts generated outputs and unclassified files separately, and never treats either omission as analysis success. Candidate enumeration uses the retained workspace root rather than the folder currently shown in Files.

All simple workspaces share one pending raw read per path between background acquisition and foreground opening. A completed read prefers an already available buffer, preserving unsaved text. Native acquisition checks workspace generation and cancels old reads on reconnect. Retaining source attaches language services without opening a pane, moving selection or recording navigation; only a foreground request activates layout. The browser publishes newly retained buffers so independent capability owners can attach even without a visible editor. The analysis orchestration and presentation are separate work; these acquisition primitives alone do not prove whole-project semantic coverage. Focused verification passed: one real serialized-tree Rust witness, two browser scope/acquisition witnesses, three Swift witnesses (including shared and delayed HTTP reads), two Kotlin witnesses against a local HTTP fixture, TypeScript checking, iOS compilation, reconstruction and ownership audit.

The Apple and Android workspaces now give each pane its own pending file-open
request. A later tab selection or active-tab change can retire that request
while the file read still populates the cache. A pane that finishes its current
open in the background does not take focus from another pane. The focused Swift
HTTP fixture preserved the selected editor, draft and caret. The focused
Android HTTP fixture also verified scroll, redo, active-tab close and focus
across panes. See `target/apple-delayed-tab-open-focused.log` and
`target/android-delayed-open-focused-receipt/` for the controller replays.

Android now carries the selected provider result's source and caret evidence
through the destination read. The held HTTP regression changed the originating
caret after selecting the result. The destination reply warmed the cache
without activating its tab or recording a visit. Selecting a current result
opened the cached source at the exact position. Three related native
controller checks passed. See
`target/android-semantic-navigation-focused-receipt/`.

Apple now carries the originating provider result through tab admission and
checks the destination editor again while waiting for it to mount. A held
provider read whose originating caret changed only warmed the cache. Leaving
and reselecting a tab retired its pending reveal. A newer saved-search choice
retired an older held row, and a fresh valid choice reached the exact position.
Seven focused Swift cases passed and the arm64 iPhone app and UI test bundle
compiled. See `target/apple-semantic-navigation-leases-final.log` and
`target/ios-semantic-navigation-leases-arm64.log`.

Source links from native reports now carry the originating editor choice
through destination loading and hash checks. Swift held reads for precise
coverage, unbound coverage and a test result, then verified that a newer
draft stayed active and a fresh choice opened the cached target. Android
Test Sources also retired the held read after dismissal and reopening or
after a round trip through another tab. A fresh choice revealed the exact
position and closed Tests. Nine focused Swift cases and four focused Android
cases passed. See `target/apple-bound-report-navigation-final.log` and
`target/android-test-source-navigation-focused-receipt/`.

Android Construction Reading now captures the foreground editor choice when
the contributor link is clicked. A focused native controller test held the
file read, switched away from the draft and back, then released the response.
The draft, selection, scroll, history and reading stayed in place. A fresh
click opened the cached, hash-verified source at its exact UTF-16 position.
Five focused Android tests passed across Construction Reading, Coverage and
Test Sources. See `target/android-construction-navigation-focused-receipt/`.

Coverage uses a separate reply for a retired source link so the report stays
open. A new report selection retires the old link before loading the new
report, and the native hosts also retire a superseded link or editor choice.
A real Simple Browser replay selected another hash-bound report while the
source GET was held. The HTTP 200 left the newer report open and retained the
editor, draft, selection and viewport. A fresh link closed the report and
revealed the verified line 3. Saved files stayed exact and no source write
occurred. See `target/coverage-navigation-held-browser-final/browser.json`.
The Android bridge cases confirmed false replies for retired links and a
true reply for a fresh exact reveal. Three focused Swift cases passed,
including the actual bundled WebKit bridge on a connected workspace. Ten
shared web cases and TypeScript checking passed. See
`target/android-coverage-navigation-final-receipt/`,
`target/apple-coverage-navigation-connected.log` and
`target/coverage-navigation-outcomes-web-final.log`.

Main Hickory Docs uses the same coverage link contract in its Tests &
coverage dialog. The workspace validates the captured hash before it opens
a precise link. In a rendered replay using imported LCOV reports, a held
file read received HTTP 200 after another report was selected. The newer
report and originating editor stayed in place, including the selection and
editor dimensions. A link with an incorrect historical hash was refused.
A fresh matching link closed the dialog and opened line 3 at UTF-16 offset
27. Saved source stayed exact. The replay exercised a plain source file.
Forty-two focused web tests and TypeScript checking passed. See
`target/main-coverage-navigation-held-browser-final/browser.json`.
In a second replay, pressing Enter on a retained peer tab during another
held coverage read left that tab active and the report open after HTTP 200.
No target tab appeared. See
`target/main-coverage-focus-held-browser-v1/browser.json`.
The real basedpyright F12 replay passed again through the shared destination
reader. See `target/main-definition-held-browser-post-coverage/browser.json`.

Simple Browser binds opening, history, source reveal and follow-up inspectors
to the same request for the pane. Selecting a retained tab retires an older
request for that pane. Work in another pane can still complete its read
without taking focus. A Chromium replay held a genuine file-tree read and
then a guarded workspace-search result. After the user reselected the draft,
both reads completed with HTTP 200. The selected editor, exact draft,
nonempty selection and working Undo survived. Neither late reply added a
tab or a history entry. A fresh visit opened the target with its original
caret, and a fresh location reveal reached the requested authored column.
See `target/simple-tab-open-focus.IMjINF/`.

Simple Browser also carries the originating provider result's source, caret
and query through the target read. With real basedpyright, a definition read
was held while only the origin caret moved from UTF-16 offset 41 to 42.
The HTTP reply completed without adding a tab or a navigation entry. A fresh definition
reached the exact target position and Back restored the retained origin at
41. The saved files kept their exact bytes. See
`target/simple-provider-origin-held.f6SsrA/replay.log`. The rendered F12
route passed the same held-read and fresh-reveal sequence. See
`target/direct-provider-origin.VjdUUT/receipt/`.

Main Hickory Docs carries the originating caret and UTF-16 column through
destination loading too. In a rendered basedpyright F12 replay, a held file
read received HTTP 200 after the origin caret moved from offset 41 to 42.
The original editor and caret remained active, and no target tab opened.
A fresh F12 reached offset 12 in the target. Both saved files kept their
exact bytes. See `target/main-definition-held-browser-v1/`.

Main Hickory Docs now resolves a closed managed destination through its
document identity. In a rendered replay, the first managed read was held
while the reader activated an existing peer tab. The HTTP 200 left the
same peer editor active and opened no destination tab. A fresh link mounted
the correct managed source and selected UTF-16 offset 31 while the saved
render and follow-up document reads were still held. Releasing the render
preserved the editor and selection. No source write occurred. The replay
exercised the guarded location event and real document room. Two focused
position tests also verify that leaving a selected destination retires its
queued position, including an eventual return to the same tab. See
`target/main-managed-reveal-held-final-v3/browser.json`.

A guarded managed source link now draws the saved text as a read-only preview
while the live document room loads. In the browser replay, the full
4,705-byte source appeared at UTF-16 offset 1877 while real room sync frames
were held. The reader selected a range and scrolled. Typing was refused
until the room and saved-file authority were confirmed. On release, the
same editor kept its selection, scroll offset, viewport dimensions and
first visible row at the same screen position. A genuine editor input
then reached the room and Undo restored the exact initial source. See
`target/main-managed-room-held-preview-final/browser.json`. Focused tests
also refused a missing save guard, an equal-text replacement guard and a
workspace change during confirmation.

An ordinary Main Files click now uses that managed preview as well. In the
rendered Files replay, a held document read returned HTTP 200 after the user
selected a peer. The peer stayed active, no late tab opened and no history
entry was added. A fresh Files click created one entry and showed the
entire saved document read-only while real room sync was held. Sync kept
the same editor, nonempty selection, scroll offset and viewport position.
The first edit reached the room and Undo restored the exact source. See
`target/main-managed-tree-room-held-preview-v2/browser.json`. A separate
source-link replay replaced the saved file with identical bytes and a new
save guard during sync. The editor stayed read-only with the same viewport
and sent no source update. See
`target/main-managed-room-held-replaced-final-v2/browser.json`.

Main guarded links to generated outputs now reuse the checked disk read.
Two rendered browser replays opened the complete 4,597-byte file read-only
at UTF-16 offset 3017 before the real provenance response. Each made one
disk read. Matching provenance attached editing in the same editor.
Different provenance left the original source read-only. In both cases
the nonempty selection, scroll offset, viewport dimensions and first
visible row's screen position held. See
`target/main-generated-provenance-matching-layout-v1/browser.json` and
`target/main-generated-provenance-mismatch-layout-v1/browser.json`.
Focused tests also preserve a queued position across editor mount replay.

Direct Files opens use the same rule once disk text is visible. Two
browser replays held real generated provenance after Files showed the
complete source. Matching provenance enabled editing. Different
provenance left the original source read-only and refused a keypress.
Both preserved the same editor, nonempty selection, scroll offset,
viewport and first visible row's screen position. Saved bytes stayed
unchanged. See `target/main-generated-files-provenance-matching-v1/browser.json`
and `target/main-generated-files-provenance-mismatch-v1/browser.json`.
A focused test also exercises explicit regeneration from the reviewed disk
and produced versions and rereads the applied result before enabling edits.

Main ranked file search now checks the complete reported chunk before
opening its tab. A real search replay held the destination read, selected
a peer, and let the read finish with HTTP 200. The same peer editor stayed
selected and no result tab opened. A saved chunk that moved during another
held read was refused, with a prompt to search again. A fresh pointer pick
revealed the original start of a ranked span at line 65, UTF-16 offset
1344. No browser source write occurred. See
`target/main-ranked-search-held-fixed-v1/browser.json`.

Main Folder Find verifies the saved file's hash and line before using
the result. A rendered replay clicked a real Files result and held its
destination read. The Find dialog closed, and selecting a peer left its
editor active after HTTP 200 with no late tab. An edit to a different
line invalidated a second result even though its matching line stayed
identical. A fresh result on line 3 converted byte column 6 after an
emoji to UTF-16 column 4 and revealed exact offset 20. The browser made
no source writes. See `target/main-folder-find-held-fixed-v1/browser.json`.

Main Problems now passes the published source and exact UTF-16 column to
the workspace before jumping. F8 cycles verified errors and warnings in
the selected plain or managed editor. Real plain-file replays exercised
both controls with genuine clangd and basedpyright. Their diagnostic hashes
matched the saved source, and both controls reached offsets 78 and 39
respectively in the same editor. Saved bytes stayed exact. The focused
navigation test also skips unknown, stale and scalar-splitting positions.
See `target/main-problems-clangd-fixed-v1/browser.json` and
`target/main-problems-basedpyright-fixed-v1/browser.json`.

Main Find References keeps the result list tied to the originating editor,
caret, workspace and foreground tab. Selecting a row rechecks that context
through the destination read and compares saved bytes with any retained
cross-file editor. The provider supplies a URI and UTF-16 position. The
target read has no provider-supplied revision to compare. A genuine
basedpyright replay used Shift+F12 from a declaration and selected a use
after an emoji in another file. The caret reached offset 53 at the reported
column 13, with no source writes. See
`target/main-references-genuine-fixed-v1/browser.json`. In a second
replay, moving the declaration caret during the held destination read
kept that editor active after HTTP 200. Dismissing the old list and
issuing a fresh search reached the same exact offset. See
`target/main-references-genuine-held-v1/browser.json`.

Main File Structure uses the existing floating prompt on authored editors.
Ctrl+F12 loads named declarations from the language provider. The filter
searches all returned names, including qualified child symbols. Selecting a
row reveals its start in the same editor. A genuine basedpyright Chromium
replay selected `OwnedStructure.target_method` at UTF-16 offset 84 without
changing the editor frame or source. An earlier reply held across a real tab
switch opened no dialog and left the newer editor active. See
`target/main-document-outline-fixed-held-v1/browser.json`.
With the same installed provider, mixed-case filtering and Enter reached
offset 84 in a plain file and offset 204 in a managed document opened from
Files. The managed result mapped a declaration in the constructed Python
output back to its authored position. Both editors kept their identity,
dimensions and text through the interaction. Neither run wrote source.
See `target/main-document-outline-final-filter-v1/browser.json` and
`target/main-document-outline-hook-fixed-managed-output-v1/browser.json`.
An ordinary Python fence in a managed document also reached its declaration
at authored offset 169 after the provider received its private inspection
document. A cold query before inspection preparation completed returned an
empty list. The source editor stayed available and a fresh query worked after
preparation. See
`target/main-document-outline-hook-fixed-managed-fence-ready-v1/browser.json`.

Main workspace symbol search accepts `@name` in the existing search box
and is also listed in its Commands mode. It uses the active providers'
answers and includes listed workspace locations with their exact position.
Before opening, it checks the reported spelling at that position and
compares a retained cross-file editor with its saved source. The provider
response carries no source hash or complete-index claim. A cold basedpyright
query returned no symbols while indexing, and a later query found the symbol
in an unopened file. Chromium reached that symbol at UTF-16 offset 53.
An older held provider reply left newer results visible. A held target read
left a newly selected peer active, and a same-column fixture rename that kept
the old name as a prefix was refused. The browser made no source writes.
See `target/main-command-symbol-provider-wire-v1/wire.json` and
`target/main-command-symbol-commit-chromium-v2/browser.json`.

Main offers declaration, type definition, implementations, callers,
callees, supertypes and subtypes in Commands when the current editor
has a mapped language position. The floating results use provider selection
ranges. A changed source, caret or foreground retires the query, including
the interval between hierarchy preparation and its follow-up. A result
must still belong to listed workspace source before it can move the caret.
Named identifiers also need to match at the reported column.

Genuine basedpyright found a caller in the current editor and reached a
previously unopened caller and callee at offsets 56 and 28. Releasing a
held incoming reply after switching tabs kept the newer editor active.
Genuine Apple clangd returned supertypes, subtypes, a declaration, an
implementation and a type definition in the same C++ editor. The five
choices reached UTF-16 offsets 25, 85, 145, 359 and 85. Both browser
replays kept the fixture source unchanged. See
`target/main-call-hierarchy-commit-v2/browser.json` and
`target/main-type-hierarchy-clangd-commit-v1/browser.json`.
One concurrent Python replay timed out waiting for the origin to become
editable before it sent a hierarchy request. The isolated replay passed.

Main records completed guarded source jumps in a bounded workspace history.
Navigate Back and Navigate Forward appear in Commands when available.
Their keyboard bindings use the primary modifier with Alt and the left or
right arrow. History compares exact source digests before reopening a file
and advances when the selected editor confirms the caret move. A real
basedpyright definition replay verified both commands and shortcuts.
Back reached the saved origin carets at offsets 41 and 42, and Forward
restored the destination's updated offset 13. Switching tabs during a held
Back read left the newer editor active and the same entry available.
Appending a line to the closed origin blocked Back until the original
bytes were restored. The browser made no source writes. See
`target/main-caret-trail-commit-v1/browser.json`.
The same active-editor guard passed a managed document reveal at offset 31
and a generated disk preview reveal at offset 3017 before provenance became
available. See `target/main-caret-trail-managed-guard-v1/browser.json` and
`target/main-caret-trail-output-provenance-v1/browser.json`.

Main also adds same-editor File Structure, Go to line, Next Problem and
managed Go to Definition visits after the editor applies the move. A real
basedpyright replay restored the plain Structure carets at offsets 32 and
85. A managed Structure replay restored offsets 152 and 205. In that managed
editor, the initial definition jump selected the full identifier at offsets
255 through 271. Back returned to the invocation at 329 and Forward returned
to the selection head at 271. A separate genuine diagnostic replay covered
F8 and Go to line through keyboard Back and Forward. The same editor stayed
mounted throughout each flow. See
`target/main-local-history-plain-fixed-v1/browser.json`,
`target/main-local-history-managed-fixed-v1/browser.json` and
`target/main-problems-local-history-v1/browser.json`.
The cross-file definition history replay also passed on this revision,
including source change refusal and retry. See
`target/main-local-history-cross-file-v1/browser.json`.

Main F2 now opens a guarded rename preview. The workflow synchronizes the
authored inputs the provider discovers, requests a source-bound rename
and verifies the returned hashes before rendering the before and after
text. The preview can include an unopened target. Draft Apply checks the
saved guards, opens any affected closed files in background tabs and waits
for each editor's own source and recovery checks. It rechecks the saved
sources before committing the entire group to drafts. Commands exposes
coordinated Undo and Redo.

A genuine basedpyright replay previewed a closed consumer, explicitly
applied a single-file rename, and applied, undid and redid a two-file rename
with both editors open. Changing the originating caret after preview refused
Apply. The browser retained the original editors and made no source writes.
The provider can return an origin-only result while indexing. After
basedpyright exposed the consumer, a fresh query returned its edits and
hash. See
`target/main-rename-commit-v2/browser.json` and
`target/main-rename-bound-wire-v2/wire.json`.

A newer genuine basedpyright replay applied to an unopened consumer.
It kept the original editor selected at the same dimensions, and
coordinated Undo and Redo restored both exact versions. A held consumer
read retired when the reader selected a peer tab. The consumer stayed
closed and both drafts kept their original text. The replay made no
source writes. See
`target/main-cold-rename-tab-final-v1/browser.json`.
A separate replay changed the saved consumer during the held read. Apply
refused the whole group before opening the consumer. A fresh preview and
Apply succeeded after the original fixture bytes were restored. A window
opened directly to one file also passed F2, Apply, Undo and Redo against
its admitted source with an empty folder tree. See
`target/main-cold-rename-external-final-v1/browser.json` and
`target/main-single-file-rename-final-v1/browser.json`.

Main code actions use the same guarded preview and coordinated draft
history. Selecting a retained quick fix or refactoring prepares a
proposal at the captured source range. If the provider discovers more
sources, Main acquires them and asks again for the uniquely matching
choice. The final edit must carry exact source proofs for all affected
files. Actions that still carry a server command or a resource operation
are refused. Apply requires a separate interaction and stages the
reviewed changes as drafts.

A genuine Apple clangd replay selected the missing-semicolon quick fix,
checked the provider-proof hash and exact Before and After text, and
applied it to a draft. Undo and Redo restored both versions in the same
editor. Moving the caret during another preview blocked Apply. A fresh
query at the original caret returned a new provider token, and the
subsequent Apply and Undo and Redo succeeded. Neither replay sent source
writes. See `target/main-code-action-fixed-v1/browser.json` and
`target/main-code-action-guarded-v1/browser.json`. The shared preview
also passed the existing Python rename replay at
`target/main-rename-shared-preview-v1/browser.json`.

Main's explicit Format document shortcut and Preview formatting in Commands
request source-bound formatting. The formatter's whole proposal uses the
same exact source proofs, Before and After, draft Apply, and coordinated
Undo and Redo as Rename. Save continues to use the existing format-on-save
setting and formatter entry point.

Chromium drove both Main entry points with Apple clangd. It checked nine
provider edits against the original source, applied them to a draft, and
restored both exact versions through Undo and Redo. A caret move before
Apply refused the preview and a fresh request succeeded. The same editor
stayed mounted and the saved source stayed unchanged. See
`target/main-formatting-guarded-v1/browser.json` and
`target/main-formatting-command-v1/browser.json`. A managed Markdown replay
mapped all nine clangd edits to the two authored C++ lines. It preserved the
surrounding prose, tags, and saved files through Apply and Undo and Redo. See
`target/main-formatting-managed-command-v1/browser.json`.

Main Commands offers File-wide diagnostic fixes and Source actions from the
current authored file, including a prose caret. Each choice retains the
whole-file range and action family through resolution. Main resolves the
retained action immediately. If its proof or edit names another source,
Main acquires that source before querying again within the same family.
The original file and every edited source require matching proof hashes.

Chromium used the installed TypeScript provider to remove an unused import
and to fix two unreachable statements in one file. The first Fix All
replay lost the action when redundant same-byte updates restarted
diagnostics. The correction resolved the held action without restarting
diagnostics and previewed both edits together. Draft Apply and coordinated
Undo and Redo retained the same editor and left saved files unchanged.
A controlled-provider replay also checked whole-file UTF-16 ranges,
filtering of contextual quick fixes, and distinct choices within one kind.
See `target/main-bulk-actions-typescript-retained-final-v1/browser.json`,
`target/main-bulk-actions-typescript-fix-all-refusal-v1/browser.json`,
`target/main-bulk-actions-typescript-fix-all-final-v1/browser.json`, and
`target/main-bulk-actions-controlled-v1/browser.json`. In a controlled
managed project, Main started from a prose caret, acquired a closed copy
contributor and queried again with the same scope. Apply opened the
contributor as a background draft. Undo and Redo preserved the focused
project editor and the saved files. See
`target/main-bulk-actions-managed-requery-v1/browser.json`.

Main now opens the complete resource review for provider actions that
create, move, or remove paths. The shared planner observes exact drafts,
saved baselines, and physical guards, then reads construction consequences.
Main displays each reviewed path in the existing floating preview. Save
reviewed source resources submits the exact guarded proposal once. After
the engine confirms the saved result, Main adopts the new baseline in any
affected open editor that keeps its path and identity. Managed editors
temporarily suspend source synchronization until the receipt and room
agree. If a newer draft arrives, Main preserves it and withholds history
adoption. Review-based workspace Undo and Redo use the retained
recovery identity and require a separate Save for each inverse. A move
to an unoccupied editor address retargets the same open plain tab and
replaces its path-specific services. Later saves use the new path and
guard. Deleting an open plain file removes its live address and keeps its
editor readable in the window. Reviewed restoration matches the retained
resource identity and reinstalls its file guard. The window arrangement
does not reopen this session-only tab as a live file after restart.
Commands offers Preview deletion of this file and Preview move of this file
for writable authored files. An open managed document stays readable after
deletion. Reviewed Undo reconnects it after the restored room matches the
saved bytes. A reviewed move between Markdown paths keeps the mounted tab
and editor. The new path supplies the managed document identity and guarded
save authority. The editor joins that room through an empty client replica
after the room confirms the saved bytes. CodeMirror keeps its selection and
scroll position. Collaboration undo history belongs to its original room.

A genuine TypeScript Move to a new file replay previewed the import in
`move.ts` and the proposed `relocate.ts` export. Both saved and draft layers
showed the exact changes, including the absent destination preimage. The
original editor, full selection, and geometry stayed in place through
Close. Then a publication replay used real reviewed Save, Undo, and Redo
to create, remove, and recreate the destination while retaining the
original editor. The engine issued a fresh guard after each publication.
A competing file created after preview caused one guarded Save to refuse;
both saved files and the editor stayed intact. See
`target/main-resource-move-preview-v3/browser.json`,
`target/main-resource-move-publish-history-v1/browser.json`, and
`target/main-resource-move-stale-destination-v1/browser.json`.

With a controlled provider and the real engine, Chromium saved a projected
resource action from an open managed document. Reviewed Undo and Redo
kept the same editor. Independent room reads matched the saved Markdown
at each step, and the client sent no ordinary room writes. See
`target/main-managed-resource-controlled-publication-v2/browser.json`.
A held successful receipt let the user type after the server committed.
The newer text stayed in the same editor and in private recovery. The
room and disk retained the reviewed bytes, and Main withheld its Undo
and Redo entry. See `target/main-managed-resource-late-local-v2/browser.json`.
TypeScript 6.0.3 through TypeScript Language Server 6.1.0 proposed
a new sibling beside the staged output. The bridge now reads the woven
code without the opening Markdown newline, shifts its position map, and
maps the new sibling to the corresponding workspace path. Chromium
reviewed both sources, then used explicit Save, Undo, and Redo against
the real Engine. The import stayed inside the authored file block, and
the surrounding prose and physical generated file stayed byte-exact.
The mounted editor was retained, the managed room agreed with the
saved document at each step, and the browser sent no ordinary writes.
The authored code and new file passed strict TypeScript compilation.
See `target/main-managed-resource-genuine-fixed-v2/browser.json` and
`target/main-managed-resource-genuine-fixed-v2/typescript-check.json`.

Creating the destination independently after preview caused one refused
Save. The competitor, original authored file, managed room, editor text,
and selection remained intact. Review history did not advance. See
`target/main-managed-resource-genuine-stale-v2/browser.json`.

The same managed move passed in mounted macOS and the Android controller
with a dirty draft. Both replayed guarded Save, reviewed Undo, and
reviewed Redo. The macOS test preserved the editor and viewport geometry
and its final independent room read matched the saved document. The
Android controller preserved the authored block and dirty suffix, and
the resulting TypeScript passed strict compilation. See
`target/appkit-staged-refactor-managed-v2-2026-11-19/receipt.json` and
`target/android-managed-move-rebuilt-2shfvobf/junit.xml`.

Native editors now apply a localized changed span and map selection
through it. Mounted macOS and iOS replayed the genuine managed Move
after that change. Save and Redo selected the import and its inserted
newline. Undo restored the exact declaration selection. The mounted
editor, viewport geometry, surrounding prose, and generated output
were retained. The iOS case also checked the same native undo manager,
the original dirty draft on Undo, and the exact saved source before
and after publication. See
`target/appkit-managed-resource-selection-2026-11-19/receipt.json` and
`target/apple-managed-typescript-move-uikit-2026-11-19/typescript-managed-move-receipt.json`.
The Android controller additionally verified a reversed selection
through each inverse and confirmed draft byte identity. See
`target/native-reviewed-selection-v1/android-focused.xml`.

Rendered Android completed the genuine managed Move with the current APK.
Save, reviewed Undo, and reviewed Redo each received a successful
publication receipt. The complete history cycled between the original
saved source with its retained dirty draft and the reviewed import plus
new module. The physical generated file stayed byte-exact, and strict
TypeScript validation passed. The editor top and horizontal bounds stayed
aligned as the keyboard inset changed. Android accessibility returned
no numeric selection offsets. The provider trace binds the original
declaration selection, and the focused controller supplies exact
post-publication range evidence. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-as2rvfq1/managed-ts-move/rendered-result.json`.

Simple Browser replayed the same move in the rendered editor. Its first
publication expanded the selection to the entire buffer. Resource adoption
now applies a display diff with the exact authored source in the same
transaction. Save selected the replacement import, reviewed Undo recovered
the original declaration selection, and Redo selected the import again
inside the authored block. The mounted editor, viewport dimensions, and
scroll position stayed in place. The resulting code passed strict TypeScript
compilation. See `target/simple-managed-resource-genuine-move-v1/browser.json`
and `target/simple-managed-resource-genuine-move-v2/browser.json`.

After Redo, Main used Go to Definition on the moved call and opened the
saved declaration in workspace `relocate.ts`. Back restored the authored
caret in the same Markdown editor. Forward restored the updated caret
in the destination. Find References listed the exact import and call in
`notes.md` and the declaration in `relocate.ts`. Selecting that row
opened the saved declaration at the same coordinate.

The staged analysis now includes bounded saved copies of literal relative
imports, including supported transitive imports and re-exports. Location
mapping checks the originating revision plus the exact saved and staged
bytes. These copies confer no edit authority. Removing `relocate.ts`
after staging left a stale child result. Main suppressed that result
for both Definition and Find References, retained valid local references,
and sent no writes. See
`target/main-managed-resource-genuine-navigation-fixed-engine-v2/browser.json`,
`target/main-managed-resource-genuine-navigation-removed-engine-v2/browser.json`,
`target/main-managed-resource-genuine-navigation-references-engine-v2/browser.json`,
and `target/main-managed-resource-genuine-navigation-references-removed-engine-v2/browser.json`.

Simple Browser repeated the move through reviewed Save, Undo, and Redo.
Once the provider had prepared the updated source, F12 opened the saved
declaration at UTF-16 offset 16. Back recovered the authored caret, and
Forward recovered the updated destination caret. See
`target/simple-managed-resource-genuine-move-navigation-v2/browser.json`.

A mounted AppKit editor repeated Move, Save, Definition, Back, and
Forward through the native command controller and result opener.
The original Markdown editor was retained and both native carets matched
their source coordinates. See
`target/appkit-managed-definition-v2-bundled-2026-11-19/receipt.json`.

A separate mounted UIKit test loaded the exact saved files from the
earlier Move and Redo fixture. The native command controller followed
the genuine TypeScript result to `relocate.ts` at offset 16. Back
restored the authored call at offset 179. Forward restored the updated
destination caret at offset 41. The saved Markdown, module, and
generated output stayed byte-identical. See
`target/apple-managed-typescript-definition-fresh-uikit-2026-11-19/typescript-managed-move-definition-receipt.json`.
Two attempts to combine the Move history and navigation in one mounted
test aborted inside XCTest's Swift error observer before Definition.
Both reached Save and the first also reached Undo. See the respective
`target/apple-managed-typescript-move-definition-uikit-2026-11-19/concurrency-crash-stack.json`
and
`target/apple-managed-typescript-move-definition-uikit-retry-2026-11-19/concurrency-crash-stack.json`.

Rendered Android completed Move, reviewed Save, Definition, Back, and
Forward in a single session. The genuine provider resolved the moved
symbol, and the native result row opened saved `relocate.ts:1:17`.
Screenshots show Definition at the declaration, Back inside the authored
call, and Forward at the caret moved before `return`. The editor exposed
no numeric selection offsets through Android accessibility, so the caret
positions are screen observations. Navigation sent no saves and kept the
published source bytes unchanged. See the result and caret observation
in `target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-zgzov7vp/managed-ts-definition/`.

Staged imports refresh when the authored document is prepared. Before
revealing a staged location, the engine checks the full bounded import
projection. A changed or missing intermediary can retire a location in
an unchanged leaf. Chromium verified this with genuine TypeScript after
rewriting and deleting a saved re-export in separate runs. In both cases
the provider still returned the old leaf, Definition left the caller and
its caret in place, and Find References retained the authored locations.
The replay sent no source writes or caller updates. Closing and reopening
the unchanged caller rebuilt the projection. Its first Definition then
opened the new saved `alternate.ts` declaration. That replay used the
same child and private root, with no source writes or text updates. See
`target/main-staged-barrel-navigation-fixed-v4-rewritten/browser.json`,
`target/main-staged-barrel-navigation-fixed-v4-deleted/browser.json`,
and `target/main-staged-barrel-navigation-v4-reopen/browser.json`.

Location commands now recheck the staged imports while the caller stays
open. Changes to the bounded saved projection replace the ECMAScript
provider for that private root. Retirement and startup have deadlines and
release the dispatcher lock during provider I/O. A replacement of a recognized
TypeScript Language Server starts in semantic mode so its first Definition
can see the staged modules. The engine still checks source revisions and
saved and staged bytes before revealing a result.

Rendered Main with genuine TypeScript opened the new declaration on the
first command after rewriting, restoring, or creating a saved import.
Deleting the import exposed only authored results. Moving the declaration
down three lines within an otherwise isolated saved leaf opened its new
coordinate on the first lookup. The tests kept the same caller editor,
restored its exact caret with Back, and sent no source writes or room
updates. See
`target/main-staged-barrel-navigation-v8-live-existing/browser.json`,
`target/main-staged-barrel-navigation-v8-live-unresolved/browser.json`,
and `target/main-staged-barrel-navigation-v8-live-shifted/browser.json`.

A separate replay withheld the old provider's shutdown response. The first
lookup still opened the shifted declaration. Before the browser closed,
the old provider process group had exited and an unrelated provider and
the replacement were still running. See
`target/main-staged-barrel-navigation-v8-live-shifted-timeout/browser.json`.

Mounted AppKit followed the first Definition to the shifted saved leaf and
restored the exact authored caret with Back in the same Markdown buffer and
native text view. The private provider restarted without a caller update or
fixture write. See
`target/appkit-saved-import-refresh-v8-settled-2026-11-19/receipt.json`.

Rendered Simple Browser passed rewrite, deletion, restoration, and initial
creation with the caller still open. In a separate replay, closing the
clean leaf initially left its old source registered, so the engine refused
the shifted destination. Closing a clean saved plain tab now releases that
registration. Reopening equal saved bytes retains the editor and its history.
Reopening changed bytes requires a fresh scoped read. The rendered replay
opened the shifted leaf on the first Definition and Back restored the same
caller and caret. Navigation sent no source writes or room updates. See
`target/simple-staged-barrel-navigation-v8-live-existing-post-close-fix/browser.json`,
`target/simple-staged-barrel-navigation-v8-live-unresolved-post-close-fix/browser.json`, and
`target/simple-staged-barrel-navigation-v8-live-shifted-fixed/browser.json`.

Rendered Android loaded the exact saved post-Move fixture in a separate
session. An independent edit shifted the saved declaration down three
lines. The first native Definition opened the exact changed source at the
new result row, and Back restored the unchanged caller bytes and frame.
The provider restarted with the same staged call location. The caller
sent no open, close, or change message during refresh, and navigation
sent no source writes. Numeric caret offsets were unavailable in Android
accessibility. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-39rxc3ez/managed-ts-live-refresh/live-refresh-result.json`.

Mounted UIKit opened the shifted saved leaf on its first Definition.
Back returned to UTF-16 offset 178 in the same Markdown buffer and
native text view. The new provider used the same staged call coordinate,
and navigation preserved the fixture files and sent no caller update.
A background completion request displayed an error from the retired
provider during that run. See
`target/ios-saved-import-refresh-v8-2026-11-19/receipt.json` and
`target/ios-saved-import-refresh-v8-2026-11-19/mounted-refresh.log`.

An obsolete completion transport failure now returns an empty response
when both its source capture and producing connection have been retired.
A mounted UIKit replay withheld the old provider's completion until
replacement. The actual client received JSON-RPC `result: null`, the
first Definition opened the shifted leaf, and a new completion produced
two visible suggestions. Back preserved the original editor and caret.
The replay used the prior native bundle with the new engine. The fresh
Swift test separately verifies the neutral completion status, retained
rows, discarded obsolete errors, and visible current transport failures.
See `target/ios-retired-completion-v9-2026-11-19/receipt.json` and
`target/apple-retired-completion-focused.log`.

Go to Declaration tries the provider's dedicated method. When it
returns method-not-found, the engine asks for Definition at the same
captured position and applies the same revision and saved-target checks.
An empty response from a supported method does not retry. A genuine
Chromium replay opened the saved `relocate.ts` through this fallback.
After removal of the saved file, it showed an empty result and kept
the original caret and navigation history. See
`target/main-managed-resource-genuine-navigation-declaration-engine-v3/browser.json`
and
`target/main-managed-resource-genuine-navigation-declaration-removed-engine-v3-retry/browser.json`.

A focused Main replay with clangd also passed managed formatting after
the position shift. See
`target/main-managed-formatting-provider-framing-v2/browser.json`.

A controlled provider's RenameFile moved an open plain file through
Save, Undo, and Redo in Chromium. The tab, mounted editor, selection,
viewport, and source bytes stayed in place. The Files tree reflected each
rename. Typing after Redo sent one guarded write to the new path and
synchronized only the new language-server address. See
`target/main-resource-renamefile-fixed-held-writes-v2/browser.json`.
A held successful receipt let the reader type in the original editor
after the disk move. The new text stayed in the editor and in private
draft recovery with retention enabled. Main sent no old-path writes
and withheld its workspace history entry. See
`target/main-resource-renamefile-late-local-v2/browser.json`.

A controlled provider's DeleteFile passed through Save, reviewed Undo, and
reviewed Redo in Chromium. All three publications retained the original
editor, text, selection, and viewport. Delete and Redo removed the path
from Files and disk and blocked ordinary typing in the retained tab.
Undo restored the exact saved bytes and editability in the same editor.
The browser made no standalone source writes. An independent file
created at the deleted path after Undo review made the engine refuse
restoration. Files refreshed automatically and opened the new occupant
in a separate editor. Returning to the old tab preserved its read-only
text, selection, and viewport. See
`target/main-resource-deletefile-final-v1/browser.json` and
`target/main-resource-deletefile-stale-undo-refresh-v4/browser.json`.

Chromium used Preview deletion of this file on an open managed document
with the real engine. Delete, Undo, and Redo kept the same editor,
nonempty selection, and viewport. The retained document stayed read-only
while absent. On Undo, an independent room read matched the restored disk
bytes, and the mounted client sent no text updates during publication.
See `target/main-managed-resource-deletefile-direct-v1/browser.json`.
A separate Delete and Undo replay typed in that same editor after
reconnection. One client update reached the restored room. The new text
matched the editor, an independent room read, and the saved file. See
`target/main-managed-resource-deletefile-direct-typing-v2/browser.json`.

Chromium used Preview move of this file to move `notes.md` to
`revised.md` with the real engine. Save, reviewed Undo, and reviewed Redo
preserved the same mounted editor, nonempty selection, and viewport.
Each publication changed the Files tree and saved path, closed the
superseded client sockets, and sent zero client text updates. An
independent read of each destination room matched the saved bytes.
A separate replay typed after Redo. Its single text update reached only
the new room, and its text matched the editor and saved file. See
`target/main-managed-resource-move-command-fixed-strict-v3/browser.json`
and `target/main-managed-resource-move-command-typing-strict-v3/browser.json`.

Focused rebuilt-engine replays also passed for Python definition and
Back/Forward, workspace search, delayed first-open authority and capabilities,
and Debug and guarded patch review on both recorded Agent message sides.
See `target/browser-tab-open-*-safety.log`.

The Simple Browser delayed-capability replay also aborts an inline rendering
stylesheet after the user has typed and selected source. The workspace
status bar reports the optional failure while the same editor keeps its
dimensions, scroll, selection, draft and Undo history. A separate Agent
replay aborts the asset with an open draft and keeps the failure in the
existing heading. See `scripts/test-simple-browser.py` modes
`--delayed-capabilities-only` and `--agent-only`.

## Native source sessions share workspace transport

Apple and Android retain per-buffer language knowledge while borrowing one workspace-owned language connection. The transport assigns unique request IDs, routes each reply to its requester and diagnostics to their source URI, and cancels only a detached source's pending requests. A duplicate URI cannot overwrite the held source or borrow its request authority. Transport failure invalidates source captures before resuming pending reads; a later read can reconnect and replay every held full-text document, including changes retained while offline. Closing the last source releases the idle wire; reconnecting a workspace closes its old transport. Server-initiated applyEdit requests are refused because source changes require a retained preview.

The shared local wire fixture exercises Hickory channel framing with actual native WebSocket clients; it does not simulate a language provider or establish provider semantics. The separate rebuilt-engine/provider witnesses remain the evidence for semantic reads, refactorings and source-bound diagnostics.

## Source-backed test facts and saved-run admission

The shared engine exposes read-only source test candidates for plain files, local literate constructions and an explicitly chosen project closure. Syntax-backed candidates retain separate authored and runtime locations; heuristic and dynamic declarations stay labeled. Candidate identity binds the workspace, input fingerprints, target bytes and provider runner scope. Optional capture registers compiler output in memory without materialization. Source candidates are not confirmed runner case IDs or test results.

The separate versioned saved-run route recomputes those facts and rechecks both target and authored contributor bytes before opening a terminal. Unknown execution modes/fields, stale scans and dynamic name-filter selections refuse admission. Files remain mutable during execution; admission does not claim frozen source. JavaScript literal names are escaped as runner regex filters. C# planning names one explicit project and refuses ambiguous project ownership.

Verification: four focused service witnesses, four source-detector witnesses and the C# scope witness passed. The real HTTP witness verifies read-only inspection, foreign compiler origins, stale contributor refusal, strict protocol selection and an actual dependency-free Rust test run in a disposable workspace. TypeScript checking, API codegen consistency and source reconstruction/audit passed. This establishes the common engine protocol; the simple-host interaction evidence follows below.

## Floating source-test interaction

The shared Edit-menu command opens Source tests on browser, macOS, iOS and Android without resizing the editor. Explicit read-only inspection uses the current source and an optional construction entry; paged candidates retain syntax/heuristic/dynamic confidence and unsupported/partial notices. Name-filter runs and containing runner scopes are separate choices. A model guards relevant draft/workspace identity before execution, while the engine independently guards saved contributors and target bytes. Relevant browser buffer subscriptions also retire observations after further edits to an already-dirty draft.

Closing the floating view retains observed terminal identities and output. A selected run can be refreshed or opened in the existing terminal surface. Each record retains the admission fingerprint, target hash and checked contributor hashes; historical terminal output never becomes current per-case verification. Per-run request tickets reject obsolete output reads. Source navigation verifies the destination's exact hash after loading and carries its matching text through the existing guarded reveal path. Replacing a native workspace replaces its observation model.

Verification: the focused browser flow uses the actual menu and engine to run a dependency-free Rust test, checks real success output, exact source-name caret navigation, admission-history disclosure, retained editor/session identity, stationary editor dimensions, unsaved-draft preservation and stale saved-source refusal. It also checks immediate stale retirement after another edit to an already-dirty buffer. Focused source-model and command checks, TypeScript checking, macOS/Android compilation and the iOS simulator build passed. These checks do not establish native pointer/IME acceptance. Source-adjacent gutter access, structured runner discovery/results and coverage integration are described below. Test debugging remains subsequent work.

## Source-adjacent test inspection

Every simple editor now requests test declarations through a separate debounced workspace index. Each index admits two reads, retains only one latest pending draft per path, avoids overlapping a path, and rejects obsolete primary-source replies. Validated source rows are cached per observation. A failed detector cannot hide source text or prevent unrelated reads. The index never executes a runner.

Compiler-owned primary-source locations expose inspection hints through the existing right gutter. Unmapped constructed runtime locations and foreign owners do not become local line marks. A gutter hint is not execution authority: clicking it activates the existing buffer and refreshes the full floating source-test view with current drafts and the selected construction context before offering a run. Native workspace replacement stops its old index. Browser buffer subscriptions keep indexing independent of editor remounting or dirty-state transitions.

Right-gutter actions use one vertical column at every width. Existing slot reservation keeps each later populated row anchored; overflow remains a floating grid. Line-action menus close before opening another surface. No new permanent editor chrome or width reservation is introduced.

Verification: focused asynchronous index witnesses pass on TypeScript, Swift and Kotlin for the two-read bound, coalescing, duplicate suppression, stale refusal, independent failure, stop, primary/foreign ownership and position bounds. The real browser flow uses adjacent one-line tests with an existing debug action, checks unchanged later-row coordinates while opening the gutter menu, and reaches the fresh source inspection. Its existing real-run, exact-caret, admission-history, retained-draft/editor and stale-save checks also pass. Focused shared rail/slot checks, TypeScript checking, macOS/Android compilation, iOS simulator build and reconstruction/audit pass. Mac and iPhone pointer receipts in [Native editor gutter overflow](#native-editor-gutter-overflow) exercise adjacent geometry with equation actions. The mounted AppKit receipt also exercises combined test and coverage actions in the native grid.

## Saved-revision runner discovery

The shared runner-discovery endpoint now returns a versioned observation: reported test names, normalized workspace-relative source paths when available, the saved hashes captured before the runner starts, a fingerprint and whether those files changed during discovery. A changed snapshot is historical; an empty snapshot is unknown. Files are not frozen, and these bounded workspace snapshots do not claim to include every runtime dependency. Discovery may build/import a project and stays distinct from read-only source inspection and completed test results.

Verification: two focused model/path witnesses pass. A real Cargo HTTP witness creates a fresh dependency-free crate, proves that lockfile creation retires the first observation without replacing its before hashes, obtains a current stable second listing, and checks a later source edit changes the identity. Its test body never runs. TypeScript checking and reconstruction/audit pass. The shared floating coverage/results surface described below consumes these observations.

## Reusable structured test/coverage surface

The existing structured test/coverage view now accepts one replaceable adapter for service calls, editor coverage selection and source navigation. Browser defaults remain available, and native embedding has its own reversible presentation. Runner discovery explicitly uses saved project files and may build/import the project. Unsupported collector languages remain unavailable; JavaScript/TypeScript aliases use their existing collector family. A paste field supports standard-report import without requiring a browser file-picker delegate.

Request tickets retire superseded/closed operation replies and deduplicate pending history reads. A newer discovery observation replaces old selected-run result rows. Navigation distinguishes unbound file opens from positive source positions with a valid captured owner hash and no observed execution-source change; each host must independently compare that hash with its current buffer before revealing a position. A rejected navigation keeps the report readable.

Verification: eight focused adapter/model/view checks and TypeScript checking pass. The view checks control this product's own adapter to prove reply ordering, closed-view refusal, history coalescing, result replacement and retained reporting after a host navigation rejection; they do not certify an external collector. Reconstruction/audit pass. Simple-host integration and actual WebKit loading are verified below; native pointer/IME acceptance remains unverified.

## Source-owned coverage projections

A shared projection contract now has TypeScript, Swift and Kotlin representations. Runtime locations remain distinct from compiler-reported authored locations; every marked owner needs its own captured SHA-256 hash. Mixed hit/miss observations stay mixed. A count is available only when every contributor supplies an exact shared-range quantity; otherwise it is null. Changed or missing execution-source observations, unbound owners and invalid mapped positions provide no editor mark. Hosts still need to bind these projections to current buffer bytes before rendering.

Verification: all three implementations pass the same five fixture cases, including the fixture loaded from actual native resource bundles. TypeScript checking, native compilation and reconstruction/audit pass. These witnesses validate projection semantics; source binding and shared-surface integration are described below. Native pointer/IME acceptance remains unverified.

## Exact native source revisions

Apple source identity compares exact UTF-8 rather than Swift's canonical Unicode equality. Normalization-only edits remain dirty and publish buffer/language observation changes; stale source-derived navigation, diagnostics, test hints, table edits and proposed changes retire on the same byte distinction. Renderer-facing String APIs remain intact. This is a source-revision mechanism, not a filesystem path normalization policy.

Verification: five focused Unicode witnesses first reproduced the dirty-state, observation and change-plan failures, then passed with the mechanism. Eleven focused source-revision, source-test/index and workspace-analysis cases pass, and the iOS simulator build succeeds. Reconstruction and ownership audit pass. These checks do not establish native pointer/IME acceptance.

## Structured runner results and coverage in existing menus

The shared Edit menu exposes Tests and coverage on browser, macOS, iOS and Android. A floating surface presents saved-project discovery, retained runner results, failure output, collector execution, imported reports, manual collection and compatible evidence comparison using the existing coverage service. Both native hosts package the same local renderer and expose only fixed coverage operations; workspace replacement retires old replies. Unbound reports open files without precise-position claims. Reported line navigation independently matches captured source hashes against loaded text; source remains readable and the editor stays mounted.

Verification: a real browser/engine witness discovers a Cargo test whose body must not execute, imports an explicitly identified LCOV fixture, navigates precisely, refuses after an unsaved edit, and retains editor geometry/identity and draft bytes with unchanged disk. A focused actual WebKit witness loads the bundled renderer and rejects obsolete-workspace messages while preserving draft selection and bytes. TypeScript checking, Android compilation, iOS simulator build and reconstruction/audit pass. The import witness does not establish an automatic collector execution, and native pointer/IME acceptance remains unverified. Coverage line-mark binding and richer test debugging remain subsequent work.

## Source-bound coverage in existing gutter columns

Selected coverage records now attach to source lines in every simple UI through the existing right gutter. Covered, uncovered and mixed measurements have distinct glyphs and accessible state descriptions; missing quantities stay unavailable. A background digest must match the captured owner hash before a mark appears. An edit removes that authority synchronously, even while hashing is pending. Multiple actions on one browser line share one anchored group, and native collision calculations include coverage rows. Opening the retained report uses the same floating surface and adds no editor column or toolbar.

These are per-owner observations from a captured run, not current-project verification or proof that execution inputs were frozen. Native selection replies carry a latest-selection ticket, and workspace replacement stops old digest services. Failed hashes leave unknown evidence without hiding readable text.

Verification: native service witnesses bind mixed generated consequences to an authored line, reject normalization-only edits immediately, restore marks only for matching bytes and clear them on deselection. Shared projection/digest checks pass, along with the existing browser rail checks. The real browser witness imports source-bound coverage onto two adjacent Cargo declarations. The first line presents test, coverage and Debug as three icon buttons in the same floating grid. The witness opens the retained report and source-test inspection from that grid in separate clicks, verifies that the grid closes after each click and compares exact coordinates for the following controls while it is open. Editing the source retires the coverage marks and keeps the same editor and saved bytes. The existing real source-test flow also passes. Native model checks and builds do not establish device pointer/IME acceptance.

## Rename targets beyond retained editors

Rename preparation now acquires previously unopened authored reference targets on all simple platforms. The initial reference answer discovers files; it does not authorize edits against text loaded afterward. Up to two background acquisitions retain source without activating tabs, synchronize the target language session through the existing outline reader, then query references again. Newly acquired snapshots join the original immutable inputs before the rename request. Unlisted, generated, unavailable or changed targets refuse explicitly. Four continually expanding query rounds or more than 10,000 source targets also refuse without applying edits.

The reviewed plan and coordinated draft undo remain the application boundary; acquisition and provider queries do not save files. The observed reference set still does not establish exhaustive project analysis. Provider actions and resource operations retain their separately stated limitations. Each native preparation now holds its own request-validity closure, so a late older result cannot borrow the authority of a newer request; byte-distinct Unicode rename names retire the older preview too.

Verification: two acquisition witnesses per platform prove bounded reads, deduplication, fresh queries, unchanged original draft snapshots and stale/expanding refusal. Focused native witnesses complete a newer rename before an older reply and preserve the newer preview. The Apple witness additionally checks Unicode name retirement. TypeScript checking, native compilation, the iOS simulator build and reconstruction/audit pass.

The Simple Browser basedpyright replay starts with two unopened consumers,
including an aliased import. Both tabs stay closed during background
acquisition and three-file review. Editing one consumer after review
causes Apply to refuse all three drafts. A fresh review applies the group;
one workspace Undo and one Redo each restore all three drafts. A subsequent
independent edit also causes workspace Undo to refuse the complete group.
The origin editor, its dimensions and all saved bytes survive.

Mounted AppKit used Search Everywhere and the native name field to prepare
a genuine basedpyright Rename. The unopened consumer joined the review
after fresh references and remained tab-closed through Apply, Undo and
Redo. The exact two-file drafts, the origin's independent native comment,
editor, viewport and Undo manager were retained. Both saved files kept
their bytes, inodes and modification times. The fixture selected each
native preview page and used app-local clicks for the commands. See
`target/appkit-basedpyright-rename._wwz4igb/receipt.json`.

In the iPhone simulator, unshifted lowercase input reached the original
Rename field with an uppercase first letter. A separate run reached the
complete two-file review and then aborted on Apply. UIKit was synchronizing
the insertion point from typing attributes while a text storage batch was
open. The iPhone field now preserves literal identifier spelling, and the
source editor updates typing attributes after closing the storage batch.
The rebuilt app prepared the exact lowercase name, showed both patches
with the consumer still tab-closed, and applied both drafts while keeping
the origin editor and saved files intact. See
`target/ios-basedpyright-rename/iphone-27-rendered/attempt-3/app.stderr`
and `target/ios-basedpyright-rename/iphone-27-exact-fixes/attempt-1/`.

The complete iPhone replay used the same unshifted hardware input, produced
`renewed` exactly, and invoked Apply, Undo workspace change and Redo
workspace change with native taps. Both drafts restored their exact text
at each step. The original unsaved comment, editor identities, dimensions,
scroll and Undo managers survived. Both saved files kept their bytes,
inodes and modification times. See
`target/ios-basedpyright-rename/iphone-27-exact-fixes/attempt-3/receipt.json`.

The stock Android 37 replay inspected the genuine bound basedpyright
review for a declaration and both references, including an offset after
an emoji. Native Apply, Undo and Redo updated both exact drafts while the
consumer tab stayed closed. Production recovery checkpoints confirmed the
same buffer identities, then recorded the consumer tab when it was opened
to read the renamed text. Both saved files kept their bytes, inodes and
modification times, and the replay sent zero source writes. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-5stjo0_w/basedpyright-rename/rendered-android-basedpyright-rename.json`.


## Managed C++ formatting in Simple

Installed Apple clangd 21 formatted C++ owned by a CRLF Markdown document
through the Simple Browser's Preview formatting command. The review left the
active source and its independent unsaved prose edit intact. Apply retained
the prose and used CRLF for the newly inserted C++ lines. Workspace Undo and
Redo restored the exact before and after drafts in the same editor. The saved
entry and owner kept their exact bytes, and no generated file appeared. Run
`python3 scripts/test-simple-browser.py --managed-formatting-only` with
clangd on PATH to replay this case with the LLVM fallback style.

Mounted AppKit used Search Everywhere to open the clangd review, inspected
the mapped C++ lines and applied, undid and redid the draft change. App-local
events drove the mounted view while the console was locked. The same editor,
undo manager and nonzero viewport survived, and the saved Markdown, project
entry and existing generated C++ retained their bytes, inodes and modification
times. The receipt includes the two genuine formatting requests and replies.
See `target/appkit-clangd-format.vkl3zwf9/receipt.json`.

The iPhone simulator replay used an ordinary C++ tab with an unsaved comment
typed through UIKit. Both guarded calls to Apple clangd 21 read that exact
draft and returned six identical edits. The visible review, Apply, workspace
Undo and Redo preserved the comment, editor identity, bounds, scroll and undo
manager. The saved file kept its bytes, inode and modification time. See
`target/ios-clangd-format/iphone-27/attempt-3/receipt.json`.

The iPhone managed-source replay opened the CRLF Markdown owner and typed
an independent prose edit. Apple clangd returned fifteen C++ edits on each
of two requests against the private generated file. The engine projected
each edit into the exact dirty Markdown revision. Native Preview, Apply,
Undo and Redo retained the prose, tags, raw CRLF, editor, scroll, frame and
undo manager. The saved owner and entry kept their bytes and file metadata.
The generated C++ path stayed absent from the workspace. See
`target/ios-clangd-format-managed/iphone-27/attempt-1/receipt.json`.

Stock Android API 37 reviewed and formatted C++ owned by an LF Markdown
document with Apple clangd 21. Both guarded replies contained nine edits
inside the authored C++ body. Native Apply and physical Undo and Redo
produced the exact drafts in the same editor. Production recovery preserved
the authored prose and tags, and saved files kept their bytes, inodes and
modification times. The replay sent zero source writes and produced no
generated C++ file. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-9zaxa_xf/clangd-formatting/rendered-android-clangd-formatting.json`.

## Engine-held provider actions

The engine now supports standard lazy code-action resolution through bounded, expiring opaque artifacts. It retains the original child value and child handle, records actual child capabilities, and ignores caller substitutions apart from the lookup token. Lazy children resolve through their original provider; eager children return their held complete value without an unsupported RPC. Each installed source/ownership construction has a revision, and an order-independent graph fingerprint gates queries, resolution and complete compiler projection. Diagnostic-cache messages alone do not change that identity. The graph describes installed mappings; it does not freeze disk or establish exhaustive workspace dependency coverage.

Resolution remains read-only: provider commands and source application are separate boundaries. Source changes, ownership replacement, eviction, expiration or foreign tokens require a fresh action query. The store retains at most 1,024 actions for two minutes. Complete mapped edits or explicit disabled outcomes return; an unprojectable edit is never silently applied in part.

Verification: three focused source-identity/artifact witnesses pass, as does the existing real-clangd diagnostic/quick-fix witness. A controlled-provider witness uses the real engine and binary LSP WebSocket channel, queries Python and TypeScript, preserves original nested provider data, ignores injected caller commands/titles/edits, checks lazy and eager paths, rejects stale/foreign tokens and rejects a held late answer after another request synchronizes changed source. Its source files remain unchanged. Controlled providers establish protocol behavior, not language analysis. Rust formatting, reconstruction and ownership audit pass. Simple-host action-chooser integration is described below.

## Resolve choices through the existing change preview

Browser, macOS, iOS and Android now resolve held provider choices before admitting their complete source edits to the existing preview. The origin document accompanies resolution so pending source synchronization is not lost on this otherwise URI-less protocol method. Legacy eager edits remain usable without an unsupported resolver. Each chooser retains its exact preparation and latest choice; replaced rows, closed views and older replies cannot borrow a newer request's authority. Resolved disabled actions, missing edits and edit-plus-command results refuse without partial source application.

The normal browser editor shares this resolver so advertising child lazy support does not remove its existing quick fixes. Its current-editor scope still refuses multi-document changes without a workspace preview. Source, selection, attachment, composition and retained choice are rechecked after resolution, and superseded replies do not replace newer status messages. No permanent controls or editor width reservations are added.

Verification: six focused browser resolver/controller checks pass, including the existing retained-choice flow, late replies and controlled composition. Native change-model witnesses check latest-choice ordering, replaced preparation identities, closing and whole edit-plus-command refusal. The real-engine browser flow uses a controlled lazy provider through the existing menu, previews before source changes, applies only on request, undoes the draft batch and retains editor identity/geometry with unchanged disk. These controlled providers establish integration rather than Python semantics. Native pointer/IME acceptance remains unverified.

## Acquire unopened provider-action targets

All simple change previews now discover unknown workspace-local action targets without inventing authored range mappings. The backend returns acquisition identities with no edit or command. Hosts load listed writable source through the existing background buffer path, synchronize its provider, and re-query at the fixed originating source and selection. A unique title and kind must match the chosen intent before fresh resolution; changed or ambiguous choices refuse. Acquired source joins the original immutable snapshot set; no tab is activated, draft applied or file saved by acquisition. The final complete planner still checks source revisions, ownership, scalar positions, overlap and unsupported operations before preview or application. This does not establish exhaustive semantic analysis or authorize resource operations.

Verification: the controlled-child browser witness starts with only the origin open, confirms the second tab stays closed during acquisition, observes two child resolutions and a complete two-file preview, then checks explicit draft application, coordinated undo, retained editor identity/geometry and unchanged disk. The protocol witness also rejects caller substitutions, foreign/stale tokens and child metadata masquerading as acquisition metadata. Focused native witnesses cover acquisition metadata, fresh querying, ambiguity, late choices and changed-target application refusal. Shared planner safety cases, TypeScript checking, native compilation, the iOS build, reconstruction and audit pass. Controlled child responses establish protocol behavior, not language-server semantic completeness; native pointer/IME acceptance remains unverified.

## Read callers and callees beside suspended drafts

Simple Browser opened a genuine basedpyright caller and callee in two
previously unopened Python files. Both carets matched the returned UTF-16
positions, including offsets after an emoji. Back restored the origin caret,
the original editor, and its unsaved draft. A second incoming result named a
caller that the fixture renamed after the query. The baseline jumped into the
longer name at the old column. The rebuilt app refused the precise jump,
kept the origin visible, and asked for a fresh query. The maintained test is
`python3 scripts/test-simple-browser.py --call-hierarchy-only` with
`HICKORY_SIMPLE_TEST_PYTHON_PROVIDER` pointing at the installed server script.
A held-preparation test moves the caret before delivering the provider reply.
The previous browser sent an incoming-call follow-up for that obsolete
preparation. The current browser retires it before issuing the follow-up.

The Simple hosts check an ordinary reported identifier at its exact UTF-16
position in a cross-file destination they just read. An adjacent identifier
character prevents an old prefix from authorizing a jump. Browser refuses
before activating the destination. Apple and Android can open destination
text while withholding the proposed caret and visit history. Existing
captured-source and history paths keep their own checks.

The mounted AppKit replay sent an exact unsaved comment to basedpyright,
then reached the local caller and callee at UTF-16 offsets 112 and 193 with
app-local mouse events. The same editor, viewport and Undo manager stayed in
place. Native Undo restored the original text. See
`target/appkit-basedpyright-call.i1lmufq0/receipt.json`. In a separate
mounted replay, the fixture renamed an unread callee after genuine
basedpyright returned it. The loaded destination showed its new name. The
old offset and history entry were withheld and the dirty origin remained
intact. The focused Swift test also covered exact UTF-16 matching, both
identifier boundaries and captured-source compatibility. See
`target/appkit-named-cross-file.o3fd8tc1/receipt.json`.

The iPhone simulator used native touches to select the caller and callee at
UTF-16 offsets 61 and 4 in its dirty Python editor. It kept the same editor,
viewport, Undo manager and saved bytes. See
`target/ios-basedpyright-calls/iphone-27/attempt-1/receipt.json`.
A separate iPhone replay selected the old cross-file caller after the
fixture renamed it from `outer` to `outerRevised`. The app opened the new
bytes, displayed the stale-name refusal and withheld the reported caret.
The origin and the renamed file kept their saved bytes and file identity.
See `target/ios-basedpyright-stale-call/iphone-27/attempt-1/receipt.json`.

Stock Android selected the genuine cross-file caller and callee at UTF-16
offsets 56 and 28. Production recovery checkpoints confirmed the exact
carets and retained origin draft. No source write occurred, and saved
bytes, inodes and modification times matched. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-d0y_jghg/basedpyright-call-hierarchy/rendered-android-basedpyright-call-hierarchy.json`.
The focused Android workspace test kept a stale destination readable and
withheld its caret and history entry. It covered both identifier boundaries
with supplementary Unicode characters, the UTF-16 offset after an emoji,
and compatibility with qualified labels and explicit source evidence.

## Type relationships through the existing language menu

Browser, macOS, iOS and Android expose Find supertypes and Find subtypes through existing semantic commands and action search. They prepare at the captured source position, preserve opaque provider items through the matching standard follow-up, and reuse the floating source-location report. The backend retains the exact child connection and original item under bounded, expiring source/ownership tokens. Caller substitutions cannot change the provider payload. Name positions use each returned item's own source mapping; a declaration spanning authored owners displays only the exact name range. Querying does not edit or save source.

The pinned server library lacks a standard type-hierarchy capability field. Both stdio and workspace sockets now use explicit standard trait handlers and one service factory for custom capture; initialization marks support through the experimental Hickory capability and simple clients invoke it explicitly. Other clients must opt into these methods until the dependency can advertise the standard field. The pinned typed response schema also cannot represent standard tag arrays; those arrays remain intact in held provider payloads, while type-list deprecated-tag styling is not supplied. A language provider may still return no relationships or an unsupported-method error; no inference fills the gap.

Verification: the actual-engine controlled-child witness prepares Python and TypeScript items before following the older Python result, preserves each original provider payload including tag arrays, rejects caller substitutions and foreign/stale tokens, follows a newly returned item, and refuses a held answer after source changes. The controlled browser menu flow checks both relationships, exact type-name navigation, retained editor/draft/dimensions and unchanged disk. Native shared-connection witnesses check both follow-ups and cursor-change refusal. Focused command and semantic models, TypeScript checking, the iOS simulator build, source reconstruction, ownership audit and Rust formatting pass.

In the Simple Browser replay with installed Apple clangd 21, Find supertypes opened the exact type name in a previously closed C++ header. Back returned to the original C++ editor. Find subtypes selected the derived type name in that editor. The draft, editor identity, dimensions and both saved files were retained. The maintained case is `python3 scripts/test-type-navigation-browser.py --clangd`.

Mounted AppKit used the same genuine provider with a native unsaved comment.
App-local input invoked both Language tools commands and selected their
results at UTF-16 offsets 25 and 85. The provider received the exact draft
before either query. Both jumps kept the same draft, editor, scroll and Undo
manager. Native Undo restored the original clean text and both the source
bytes and file identity stayed intact. The harness scrolled the floating
results into view. See `target/appkit-clangd-type-draft.e_r23gyu/receipt.json`.

The iPhone simulator entered an unsaved space through native hardware key
input. Genuine clangd received the exact draft and both standard hierarchy
follow-ups. Native result taps reached the displayed C++ base and derived
names at UTF-16 offsets 7 and 42. Both reads retained the same editor,
viewport, Undo manager and exact draft, with saved bytes and file identity
unchanged. See
`target/ios-clangd-hierarchy/iphone-27-visible-replay/attempt-1/receipt.json`.

Two stock Android 37 replays chose the native Language tools commands and
their result rows through native accessibility actions. The production
recovery checkpoints recorded an unsaved Delete, the original editor
identity and the two exact carets at UTF-16 offsets 7 and 42. Genuine
clangd received the exact draft for both queries. Both preserved that draft
and sent zero source writes. The subtype replay also verified the saved
bytes, inode and modification time. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-z37vx4jx/clangd-hierarchy/supertypes-verified-evidence.json`
and
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-_u1dxqit/clangd-hierarchy/rendered-android-clangd-hierarchy.json`.

## Retained completion provider context

Completion offers now use the same installed source/ownership revision model as actions and type relationships. Each token retains the actual producing child, original query cursor and standards-expanded provider item. List defaults materialize explicit edits and opaque data before retention; caller mutations cannot select another provider or alter its payload. Resolution handles eager providers without an unsupported RPC, retains complete import edits and refuses stale/foreign artifacts or late source changes. Only known edit fields are projected, so range-shaped opaque data remains untouched. Primary replacements must contain the original cursor on one line; exact source projection refuses edits crossing fragments or authored owners.

The [completion mechanism](../mechanisms/provider-completion-artifacts.md) is independent of popup rendering and never applies, saves or executes a command. This backend slice establishes safe resolution; frontend acceptance is established separately below; snippet field interaction remains required. Three focused model cases and the actual-engine two-provider protocol witness pass for defaults, original opaque Unicode/range-shaped data, complete import results, eager behavior, trigger context, caller substitution refusal, stale/foreign tokens, held-reply refusal and unchanged disk. Focused existing action/type protocol checks, source reconstruction/audit and Rust formatting also pass.

## Whole completion draft model

The shared completion draft models on browser, Apple and Android prepare a primary replacement, complete import edits and the resulting UTF-16 selection without touching UI or disk. Imports inserted at the primary edit's start precede it in the provider's declared order; identical imports collapse, while distinct Unicode byte sequences stay distinct. Other changes pass through the existing source planner. Replacements before the primary edit move its selection consistently. A replaceable snippet expander can supply plain text and an initial selection; absent expansion refuses snippet syntax rather than inserting it literally.

A completion may explicitly allow an empty patch set to move a caret, but retains its exact before snapshot and still requires fresh host source/selection/permission checks. Other workspace proposals continue to refuse no-change results. Commands, malformed edits, overlap, read-only ownership and scalar-splitting selections refuse the whole proposal. The Apple vectors reproduced JSON booleans being accepted as integer positions and format values; exact numeric decoding now rejects those values consistently with browser and Android.

Verification: all three models pass the same 21 source/import/selection/refusal vectors and the existing workspace-edit safety cases; TypeScript checking, native compilation, the iOS simulator build and source reconstruction/audit pass. This establishes draft planning and an expansion interface. Completion-popup acceptance is established separately below; snippet field interaction remains required. No native pointer/IME acceptance is inferred from these model checks.

## Resolve and accept complete source completions

Browser, macOS, iOS and Android use the existing completion popup or language menu to resolve the retained provider choice before applying its primary replacement and imports together. The source draft model supplies the complete change and final selection. Source, selection, attachment, permissions, saving and latest-choice checks reject obsolete replies. Dismissal cancels pending acceptance; a newer choice owns its own status. Source text stays readable while resolution runs. Browser choices share one immutable query snapshot rather than copying the document for each offered item.

Acceptance changes only the draft, with one undo/redo step for a text change and no implicit save or provider command. The native Apple bridge records the complete source and final selection together; stale source or permissions retire incompatible history after the active undo notification, preserving the current draft. Existing status areas display progress and refusal without adding a toolbar or resizing the editor. Cross-owner generated completions still require exact writable projection.

Verification: five focused browser controller/actual-popup agent checks pass, including held replies, replacement choices, composition, cancellation and settled application reporting. The actual-engine browser witness resolves imports, checks one-step undo/redo, retained editor identity/geometry and unchanged disk, then dismisses a held resolution and releases it without a late write. Two AppKit/controller checks pass for compound import edits, final redo selection, undo during saving, stale-history retirement and latest/dismissed replies. Two Android tests pass using the actual text state and compound undo/redo. TypeScript checking, the iOS simulator build, source reconstruction and audit pass. These are not native gesture, IME or accessibility acceptance results. Broad regression runs remain user-owned.

Simple Browser used TypeScript Language Server 6.1.0 with TypeScript 6.0.3
to select an unresolved identifier's completion from an unopened module.
The child received its original resolve data and returned the import edit.
The draft received the exact replacement and import in one ordinary undo
step. Redo restored the completed caret in the same editor, the module tab
stayed closed and both saved files retained their bytes. Run
`python3 scripts/test-source-completion-browser.py --typescript` with
`HICKORY_SIMPLE_TEST_TYPESCRIPT_PROVIDER` set to the installed server entry
point. A separate controlled replay still covers nested fields, linked
typing and cancellation while resolution is held.

Mounted AppKit offered the same real TypeScript completion with its
`./answers` source. The suggestion resolved against an independent unsaved
comment. One Undo text edit removed both changes and one Redo restored both
edits and UTF-16 caret 85 in the same view, frame and undo manager. App-local
events drove the mounted menus with the console locked. The saved origin,
module and TypeScript configuration retained their bytes and file metadata.
See `target/appkit-ts-auto-import.at2vr5o_/receipt.json`.

The mounted iPhone used the controlled lazy Python provider to review a
completion with a Unicode linked placeholder and an additional import edit.
A native tap applied both edits at once and selected the first placeholder
at UTF-16 range `[0, 8)`. One native Undo and Redo restored the exact text
and selection. Hardware typing changed both linked occurrences together and
had its own Undo and Redo. The compact sheet's dismissal swipe cancelled a
held resolution, and the released reply left the restored draft and caret
intact. The mounted editor, frame, scroll, undo manager and saved file
metadata survived. See
`target/ios-source-completion-native/iphone-27/attempt-1/receipt.json`.

Stock Android API 37 offered the genuine TypeScript auto-import from
`./answers` and used one native Undo and Redo for the replacement and
import. The same visible editor retained its frame while its authored
caret moved from UTF-16 offset 34 to 85, back to 34, then to 85. The
exporting file stayed unopened, and the saved files and configuration kept
their bytes and file metadata. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-bic8r2xt/typescript-completion/rendered-android-typescript-completion.json`.

Simple Browser, Apple and Android group ordinary completion for 2–16 captured
carets. Each caret queries and resolves a unique matching offer with its
own token on the exact source revision. The group requires an agreed plain
replacement, the same additional edits and disjoint primary ranges. An
implicit edit also requires matching identifier prefixes at collapsed
carets. The source transaction restores the complete caret roster with one
Undo and Redo.

The Simple hosts also prepare a group of independently resolved snippet
offers. The engine binds each caret context to the captured position and
uses its authored ordinal for cursor variables. A group shares one clock
and random capture. It validates all first replies and planned edit ranges
before a requested clipboard read, then sends the result only to the
tokens that requested it. The engine keeps retry context bound to that
caret and snapshot. Shared additions are applied once before the complete
field roster enters history. Source-equal groups select their fields and
keep the existing text history. Starting a group inside actively linked
template fields remains a guarded refusal.

Focused browser, AppKit and Android checks cover separate caret offers,
shared edits, delayed or invalid context, clipboard admission and linked
field history. AppKit mounted the source-equal path and kept prior Undo
history. Android exercised the exact editor text state and complete roster
through Undo and Redo. These checks used controlled provider replies.

The rebuilt engine rejected a foreign caret and a changed clipboard retry,
then accepted the captured retry without repeating child resolution.
Simple Browser selected a visible completion and restored both ordinal
fields through one Undo and Redo. Mounted AppKit used Add cursor below,
selected the same provider choice, and received two independent resolutions.
Both hosts inserted one shared import, kept the same editor and frame, and
updated both sets of mirrors after Redo. AppKit also kept its scroll offset.
The saved source stayed unchanged.
See `target/grouped-snippet-integration/protocol.log`,
`target/grouped-snippet-integration/browser-grouped.log` and
`target/appkit-live-grouped-snippet.c14lbk30/receipt.json`.

The iPhone simulator applied the exact two provider expansions and one
shared import. Native Undo restored both original carets, and Redo restored
both ordinal fields. A native `r` keystroke updated all four mirrors.
Editor identity, frame, scroll, Undo manager and saved file metadata survived.
The replay also exposed the selection controls covering part of the first
source row. See
`target/ios-grouped-provider-snippets/iphone-27/attempt-1/receipt.json`.

The iPhone now puts selection actions in Edit. The signed rebuild showed
both painted fields on their original rows after Apply and Redo. A native
lowercase `r` in the selection sheet reached all four mirrors exactly.
Undo and Redo restored both selections, and Clear removed the selection
group while preserving the draft. The same editor, Undo manager and frame
survived, with saved bytes and metadata intact. See
`target/ios-complete-file-fix/combined-literal-1/grouped-edit-selections/attempt-1/receipt.json`.

The stock Android rebuild used the Edit menu for selection actions and
painted both fields on the first two source rows after Apply and Redo.
The replay placed the first caret through native accessibility, added the
second through Edit, and resolved two independent offers. One Undo restored
both original carets. The replacement dialog updated all four mirrors
with lowercase `r`. Gboard appeared during that explicit input. The same
editor and source width survived, and the saved file kept its bytes, inode
and modification time. An automatic completion refresh left the applied
status intact. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-ebkrzvwq/grouped-snippets-menu/rendered-android-grouped-snippets-edit-menu.json`.

The main Hickory Docs plain-file editor chose the grouped completion from
its visible popup in a CRLF fixture. Two independent provider resolves
inserted both ordinal fields and one shared import. Native Undo and Redo
restored the exact rosters, and typing updated all mirrors. The same editor
and frame survived. The saved bytes, inode and modification time held past
the ordinary autosave delay. Explicit Save All then published through the
guarded source-history route, preserving the CRLF rows and the provider's
literal import. See `target/main-plain-snippet-acceptance/candidate-1/`.

The main managed-document replay chose one completion for two provider
resolves in an LF `hick:file` body. It inserted the shared addition within
that author, kept the surrounding prose and scaffolds exact, and restored both field
selections with native Undo and Redo. Linked typing updated all mirrors.
The editor and viewport stayed in place, with zero collaboration update
frames or source writes before explicit Save. Save published only the
managed draft through its original source-history guard. The generated
file stayed byte-identical. A separate replay refused the same provider's
virtual EOF insertion because it crossed the authored boundary. See
`target/main-plain-snippet-acceptance/managed-final-3/` and
`target/main-plain-snippet-acceptance/managed-candidate-2/`.

Main's Insert reusable template command opened the actual workspace catalog
for a CRLF plain file and an LF managed code body. Each grouped preview
preserved both carets and saved source, and Apply rechecked the identical
two-caret plan. Native Undo and Redo restored both fields, and typing updated
their mirrors. The managed prose, scaffolds and generated file stayed exact.
The editor stayed mounted without source writes or collaboration updates until
explicit Save. See
`target/main-plain-snippet-acceptance/{plain-template-3,managed-template-3}/`.

Installed TypeScript completed both prefixes and inserted one shared
import in Simple Browser and mounted AppKit. The Browser replay used the
existing popup with two exact editor selections. AppKit used Add cursor
below and Language tools, then native Undo and Redo. Both retained their
editors and saved bytes. Browser focused cases also rejected conflicting
imports and a late reply after a secondary caret round trip. Run
`python3 scripts/test-source-completion-browser.py --typescript-group`
with the installed provider. See
`target/appkit-live-caret-completion.g594rhqw/receipt.json`.

The first iPhone run exposed UIKit changing the import's closing quote
during the guarded edit. Draft-history edits now use literal TextKit
storage. The rebuilt iPhone used native Add cursor below and chose the
visible TypeScript completion. Two genuine queries and two resolutions
produced both replacements and one import with exact ASCII quotes.
Native Undo and Redo restored both UTF-16 carets from offsets 34 and 69
to 84 and 123 and back with the same editor, viewport and undo manager.
The saved files kept their bytes, inodes and modification times. See
`target/ios-caret-completion-native/iphone-27-exact/confirmed-literal/receipt.json`.

Android's focused editor tests apply the two independently resolved edits
and shared import in one native text-state transaction. They restore the
full caret roster through Undo and Redo, and preserve the entire draft when
a selection, provider proof, composition state or edit range invalidates
the group.

Stock Android API 37 used Add cursor below and selected the visible
TypeScript choice from `./answers`. The provider received two queries and
two resolutions. The changes produced both exact replacements and one
shared import. Native Undo and Redo restored both text versions and
both caret rosters, checked through production recovery and the painted
secondary caret. The same editor and source bounds survived. The module
stayed unopened and the saved bytes, inodes and modification times stayed
fixed. See
`target/parity-rendered-android.sgYIt7/continuation-receipts/final-android-4i301xke/typescript-group/rendered-android-typescript-group.json`.

## Shared snippet expansion foundation

The engine now exposes one renderer-independent snippet plan for completion and source-owned live-template services. It retains literal text, every numbered or unknown-variable field occurrence, UTF-16 ranges, choice alternatives and numeric visit order ending at the exit. Forward references share collected defaults, including nested fields; repeated unknown variable names become linked editable positions. Known variable values remain literal, while unset values use defaults. Cyclic/conflicting defaults, malformed or excessive input and unsupported transforms refuse without producing a partial proposal. This common non-transform subset does not establish full snippet syntax or advertise full host support.

Three focused constructed-module and Cargo witnesses pass for forward/nested mirrors, Unicode ranges, choices/escaping, literal/default/unknown variables and whole-proposal refusals. Rust formatting, source reconstruction and ownership audit pass. Host field navigation, editing/mirroring and templates remain required follow-ups; initial completion expansion is established below. This pure model edits no document and adds no UI.

## Expand resolved snippets without coupling the parser to UI

All simple completion hosts now consume a versioned plan produced by the engine's shared snippet model after authored source projection and indentation. Engine metadata replaces any provider-supplied expansion metadata and is bound to the exact resolved insertion syntax. File and line variables use authored identity and coordinates; other recognized variables unavailable to this service use defaults or empty text. Unknown variable names become editable fields. The backend retains original provider context and refuses unsupported transforms rather than applying imports alone.

Browser, Apple and Android validate syntax identity, numeric positions, scalar boundaries, consistent mirror text/choices and a unique exit. Their existing completion planners retain every field position after import edits and select the first numbered field. This integration applies expanded text and imports together to the draft, with no automatic save. The browser witness exposed CodeMirror redo mapping the old caret through the insertion instead of retaining the snippet selection. Completion now records a selection-only history event in the same synchronous view update; one ordinary undo/redo restores both source and the intended field selection. Existing surfaces and editor dimensions are retained.

Verification: the same 18 wire/import/refusal vectors pass on all three hosts. Eight focused browser checks pass, including the reproduced redo-selection case and actual popup agent acceptance. The controlled-provider real-engine browser flow passes for expanded Unicode mirrors, imports, initial field selection, one-step undo/redo, dismissal of a held reply, retained editor geometry and unchanged disk. Three AppKit/controller tests, three Android acceptance tests and existing completion-plan vectors pass; the actual native text editors restore the selected field with imports on redo. Four focused Rust snippet/wire tests, TypeScript checking and the iOS simulator build pass. Source reconstruction/audit and Rust formatting are checked separately. Tab traversal, mirrored typing, choice interaction, live-template commands, unavailable variable contexts and transforms remain required work. Snippet advertisement to providers remains disabled until field editing is integrated; these witnesses do not establish native gesture/IME/accessibility acceptance.

## Retain explicit snippet occurrence ancestry

Snippet fields now carry stable occurrence identities and explicit parents through the engine wire plan and all simple source planners. Adjacent empty fields can occupy the same positions as nested empty fields; editing must use this relationship rather than infer ownership from offsets. Hosts reject partial or duplicate identities, missing or non-containing parents and cycles. Legacy plans without identities retain initial expansion/selection support, but do not establish an editing session. The implicit exit now counts toward the same 4,096-occurrence bound as explicit fields.

Verification: five focused Rust parser/wire tests pass, including nested versus adjacent empty fields and the exit-inclusive limit. All three hosts pass 28 shared wire/import cases, including ancestry preservation after imports and malformed forests. Focused completion acceptance and browser redo tests continue to pass. These establish a field relationship model; Tab traversal and mirrored typing remain required integrations.

## Source-bound linked field editing model

Browser, Apple and Android now share the same snippet-session behavior contract independently of editor rendering. A current active field accepts an entire local replacement and prepares every mirrored replacement before returning a proposal. Explicit ancestry distinguishes child editing from parent replacement; parent edits retire descendants and child edits retain enclosing mirrored fields. A no-op retains nested fields. Ordered coincident empty mirrors insert text for every occurrence and preserve the positions of empty siblings. Numeric traversal follows field indices to the explicit exit, even when that order differs from source order.

The model retains exact source, UTF-16 positions and the originating caret, rejects scalar/CRLF-splitting offsets and mismatched mirror contents, and bounds total inserted mirror text before constructing a result. Stale, outside or unsupported intent returns no linked proposal; it must not block ordinary host input. Nothing in the model applies source, saves or opens UI.

Verification: all three production models pass the same 25 source/field/selection scenarios, including nested and empty fields, ordered coincident mirrors, numeric traversal, no-op preservation, distinct Unicode forms, multiline input and nested exits. Refusal cases cover stale source, edits outside a field, scalar and CRLF splits, incompatible mirrors, fanout limits and invalid session relationships. Browser checks also assert the input session remains immutable. TypeScript checking and native compilation pass. These pure-model results do not establish actual editor interaction; keyboard/menu traversal, mirrored typing, choice interaction and IME acceptance remain required integrations.

## Connect source templates through existing editor controls

Browser, Apple and Android attach optional source-bound template controllers to their existing editors. Linked typing prepares every mirror before applying one native source edit. Numeric next/previous traversal, local field choices and finish actions use the existing Edit/language menus and completion popup; hardware keyboard handling adds no permanent controls. Choice interaction does not require a language provider. Rejected or stale mirroring leaves ordinary input available. Plain completion inside a field updates its mirrors; imports or nested snippet completion currently refuse the entire choice and retain the draft.

Text undo remains owned by each editor. Bounded optional history context restores field relationships and intended selection when the matching source step is available; eviction retires template context without disabling text undo. Native source identity, write permissions, saving and composition checks remain authoritative. Browser and native plans bound expanded UTF-8 text and choice metadata independently, preventing empty mirrors from amplifying large alternative lists.

Focused evidence: 20 browser tests and 13 Apple tests pass, along with the focused Android interaction, history, shared session, completion and command tests. The same 25 session scenarios run on each host. These checks cover linked typing, choice acceptance without a language connection, source undo/redo, equal-position navigation invalidating a held completion and whole-action refusal. Native marked-text handling retains ordinary input; mirrored IME, nested insertion, full transforms/variable contexts, multi-caret templates and the reusable live-template catalog remain required parity work. Native physical input and accessibility acceptance remain unverified. The iOS simulator build passes. The controlled-provider real-engine browser flow passes expanded Unicode fields, linked typing, existing-menu traversal, exact undo/redo selection, held-reply dismissal, retained editor identity/dimensions and unchanged disk. TypeScript checking passes. Broad regression checks remain user-owned.

## Compose completion imports with active field mirrors

Completion drafts retain their primary replacement separately from their import edits. All simple hosts now mirror that primary through the active field session and apply imports outside its complete field forest once. Prefix imports shift every field and the intended caret; suffix imports leave those positions intact. Coincident prefix imports retain provider order. Imports inside the forest, overlapping edits, stale source and excessive import text refuse the entire action. A source-equal completion retains scope and selection without adding a text undo step. Nested snippet insertion remains required work.

The 30 shared session scenarios include prefix/suffix imports, ordered coincident imports and whole-action refusal. Browser acceptance and real native text-state/editor witnesses check imports, mirrored replacement, field ranges and exact undo/redo selection together. This extends the earlier conservative refusal of all imports inside active templates; it does not establish nested scope, native IME or full provider-command support.

## Nest template scopes without changing the editor layout

All simple completion hosts can now insert a snippet inside an active field. The shared session model clones its inner field forest beneath every outer mirror, allocates distinct occurrence identities and linked group indices, and retains local traversal in a bounded scope stack. Inner edits update their mirrors and enclosing fields together. Finishing the inner scope returns to the originating outer occurrence at its explicit exit; finishing the outer scope reaches its original exit. Imports outside the field forest remain single source edits. Expansion limits include cloned fields and choices.

The same 37 scenarios cover nested and twice-nested scopes, empty fields, Unicode boundaries, the second outer mirror, explicit exits and malformed ancestry alongside the prior edit/import cases. Browser and Android editor witnesses pass nested completion, mirrored typing, undo/redo and outer-scope return. The real-engine browser flow passes nested provider resolution with imports, four linked occurrences, nested/outer menu traversal, unchanged editor dimensions and unchanged disk. The Apple editor and completion-history witnesses pass after retaining the scope stack in both history directions and separating consecutive explicit actions while preserving Cocoa’s event frame. These results supersede the earlier refusal of nested insertion; full transforms/variable contexts, multi-caret templates, native mirrored IME and the live-template catalog remain required work.

## Review saved Git changes from the existing View menu

Browser, macOS, iOS and Android expose Repository changes through the existing command descriptors, menus and action search. A replaceable reading model owns captured branch/file state, paging, side selection and request identity. Refresh, file/side changes and close retire older replies; a diff whose returned path differs from the requested byte identity never displays. Swift row identities preserve byte-distinct Unicode names. Floating views show staged and working-tree sides independently, retain selectable bounded diff text and explicitly exclude unsaved editor drafts. Inspection neither saves drafts nor stages files.

The shared backend now reads NUL-delimited porcelain records, preserves literal filenames and rename pairs, and translates repository-root paths into workspace paths. Git pathspec expansion, external diff and text-conversion drivers are disabled for these reads; untracked files have no staged diff. Six focused backend cases pass, including a real repository with newline, quoted, Unicode and magic-looking names. The host models pass held/reversed/closed/foreign reply checks. The controlled real-engine browser flow passes staged rename and working-tree/index reads while preserving an unsaved draft, editor identity/dimensions and disk bytes. The browser witness also reproduces the old false-empty diff for a file removed after listing, then passes against the corrected engine with an explicit read failure. TypeScript checking and the iOS simulator build pass. Native pointer/accessibility acceptance remains unverified. Staging, history/blame and conflict UI remain required follow-ups; a captured reading is not a freshness guarantee for a later mutation.

## Stage saved paths through the shared review model

Every simple platform exposes Stage saved file and Unstage file inside the existing floating repository review. These actions operate on current saved Git paths and never save editor drafts. A rename action includes both workspace-local names; an old name outside the workspace refuses before dispatch. Native review models now belong to the workspace so closing/reopening the view retains the pending-operation guard. Changing workspace retires that model’s presentation context.

One index mutation runs per live review model. A failed or unconfirmed write is never replayed automatically and requires a successful fresh read after settlement before another index action. Reading while a write is pending does not clear this requirement. A newer inspection or closed view retains its own request identity; Git remains authoritative for an already-sent operation. A confirmed operation refreshes saved state without touching buffers, selection or layout. These actions stage current saved paths; they do not promise that externally mutable bytes still equal an older diff.

Three focused browser cases, four Apple cases and three Android cases pass for rename paths, Unicode identities, held/closed replies, no automatic retry and fresh-read admission. The real-engine browser flow stages the saved file while excluding its unsaved draft, unstages it, then unstages both names of a literal-path rename. Its disk files, dirty editor text, editor identity and code dimensions remain unchanged. TypeScript checking and the iOS simulator build pass. Repository history/blame, conflict resolution and the other parity requirements remain outstanding; native physical input and accessibility acceptance remain unverified.

## Read repository and saved-file history without replacing live source

All simple platforms expose Repository history and Saved file history through existing View menus and action search. An independent history model captures the requested scope, pages reported commits and admits immutable readings by full SHA-1 or SHA-256 identity. A returned reading must match the selected full identity. Its exact before/after blobs, binary state and empty sides remain separate from editor drafts. File/side selection is local to the floating reading; no checkout, save or index operation occurs. The current log window starts at 120 commits and can grow to 1,000; this bounded window is not exhaustive history search. Text display limits are explicit.

Distinct requests keep headings and data together. Browser models are keyed by workspace/path; native menu adapters replace the read session before presenting a new request. Closed and obsolete replies retire independently. Git history filters now treat filenames as literal paths, including names resembling Git patterns.

Two focused history cases pass on each host for held/closed/foreign replies and malformed full identities. The real-engine browser flow reads repository and saved-file history, verifies immutable before/after text, and opens an uncommitted pattern-looking filename whose literal history is empty. The witness reproduces the prior unrelated-commit result against the older engine and passes after the shared reader correction. It also preserves the editor instance, caret, dirty draft and undo/redo, while HEAD, index, disk and code dimensions remain unchanged by history inspection. TypeScript checking and the iOS simulator build pass. Source reconstruction/audit and targeted Rust formatting are checked separately. Blame/attribution, conflict resolution, recovery and the broader parity work remain required; native physical input/accessibility acceptance remains unverified.

## Inspect saved-file attribution and read exact reported commits

All simple platforms expose Saved file attribution through the existing View menu and action search. Its independent read model keeps the captured path, saved line numbers/text, author and provider facts together; foreign paths, duplicate line identities and obsolete/closed replies refuse before presentation. Facts remain readable when an older provider supplies no full commit reference. The optional immutable identity is separate from the compact display label, and uncommitted lines never admit commit reading.

Git attribution now retains the full commit identity in the shared engine. A valid committed row constructs a typed immutable-reading request for the existing historical surface. History models distinguish log scopes from exact observed commit requests, so attribution does not fabricate a log choice or resolve a short label. Captured requests and responses must match the full identity. This reads exact before/after blobs without checking out, editing, saving or staging anything. Attribution is a saved-file reading with no source revision stamp: draft line positions can differ, and these unversioned facts are not attached to live editor gutters or admitted as source edits.

A focused real-Git engine case checks full identity on a committed line and absence on a dirty line. Five focused browser cases and five cases on each native host pass across attribution and history for old/closed/foreign replies, byte-distinct paths, duplicate lines, full-identity admission and direct observed commit requests. The real-engine browser flow reports saved attribution while excluding the unsaved draft, opens the stable line’s exact commit, and retains source, caret, undo/redo, staged content, saved file bytes and code dimensions. The caret witness waits for rendered frames after editor reattachment rather than reading transient DOM selection. TypeScript checking and the iOS simulator build pass. Git path fixtures use host-legal literal names; the targeted fixture passes on this host, not a claim of Windows runtime acceptance. Conflict resolution, durable recovery and the other parity requirements remain outstanding; native physical input/accessibility acceptance remains unverified.



## Explicit cleanup through the existing change model

The Edit menu offers Trim Trailing Whitespace on every simple UI platform. It
prepares removal of trailing ASCII spaces and tabs through the shared engine,
without requiring a language provider. Before/after source, guarded draft apply
and coordinated workspace undo/redo use the existing change preview. Nothing runs
automatically on save. Indentation, line endings, other Unicode whitespace and
the presence or absence of a final newline remain intact. A no-op creates no
change. Cleanup can alter significant whitespace, so its explicit preview remains
the review boundary.

Focused evidence (2026-11-15): two shared Rust cleanup cases pass for Unicode
positions, CRLF, bounds, invalid scope and no-op. Two cleanup cases on each native
host pass for provider-independent preview/apply/undo/redo, saved-base retention,
held response ordering and close retirement. The browser real-engine witness
passes the existing menu, Unicode draft preview/apply/workspace undo/redo, no-op,
retained editor instance/dimensions and unchanged saved bytes. TypeScript checking
and the iOS simulator build pass. This establishes this explicit cleanup profile;
semantic cleanup profiles, the template/code-generation catalog and the rest of
the parity requirements remain required. Physical native input/accessibility
acceptance is not established by these model tests and build.

## Inspect immutable conflict stages inside repository review

Every simple UI offers Inspect conflict stages inside the selected file’s
existing repository review. Stage 1 is the base; stages 2 and 3 keep Git’s labels,
including during rebase. Exact immutable object names and file modes accompany
paged text. Missing, binary, oversized and special-file stages are not invented
as empty editable text. Inspection changes neither working source nor index.
These stage observations are separate from dirty editor drafts.

Focused evidence (2026-11-15): two real-Git backend cases pass for literal
pattern-looking paths, exact stages, stable fingerprints, binary refusal and
unchanged index/working text. Two focused cases on each UI host cover held,
closed and byte-distinct foreign replies and malformed stage availability. The
real-engine browser witness displays all three actual merge stages while
retaining dirty source, editor identity/dimensions and undo/redo; HEAD, index and
saved bytes remain unchanged. TypeScript checking and the iOS simulator build
pass. This is conflict inspection: applying a reviewed resolution with guarded
source and index authority remains required work. Native physical input and
accessibility acceptance remain unverified.

## Preview an available conflict text stage as a source change

Every simple UI can offer an available regular Git conflict stage through the
existing source-change preview. Recheck the captured conflict fingerprint, object
identity, mode and exact chosen bytes before preparing that proposal. Retired,
changed, binary and special-file stages cannot offer source replacements. Bind
the preview to the exact authored draft and selection; a later source/selection
change refuses approval. Applying changes the draft only and uses coordinated
workspace undo/redo. The existing Save and Stage actions are separate, explicit
boundaries; choosing a stage never commits or changes the index automatically.

The same reviewed-source proposal boundary is available to other source-aware
capabilities without importing or replacing an editor renderer. Missing target
captures and changed source refuse before becoming previews. This mechanism
currently supports ordinary text changes; resource operations still require their
complete namespace/save model.

Focused evidence (2026-11-15): each UI host passes choice admission cases for
changed fingerprints, mismatching selected bytes, closed replies and binary
stages. Each native host passes reviewed-source preview/apply/workspace undo/redo,
saved-base retention, missing/stale captures and changed-selection refusal. The
real-engine browser flow reads actual conflict stages, reviews a stage against a
dirty draft, applies and exactly undoes/redoes it, then separately saves and stages
the chosen bytes to resolve its temporary conflict. HEAD remains unchanged;
inspection and draft application leave index/saved bytes unchanged until those
explicit actions. Editor identity/dimensions remain stable. This covers choosing
a whole regular text stage and subsequent ordinary editing; assisted merge-hunk
composition and delete/rename conflict resolutions remain required. Native
physical input and accessibility acceptance remain unverified.

## Preserve explicit absence in resource namespace analysis

The shared compiler model can mark source addresses absent independently of saved
bytes or pinned snapshots. A construction that still refers to a removed source
refuses analysis. Updating its entry and supplying the new source address permits
a read-only moved-source analysis without creating files or outputs. Construction
reading accepts the same optional missing-source set; existing UI requests retain
their default empty set.

A separate resource recovery store can retain versioned saved before/after states
and append-only per-path receipts before a later publisher writes source. It
retains exact text, missing states and modeled permission metadata, refuses
pre-existing symlink/non-directory recovery paths and replaced batch directories,
and never automatically replays a receipt. This is a prerequisite for recoverable
namespace saves, not a connected resource Save command or a cross-file transaction.
Candidate saved/draft construction ownership must be rechecked against the final
namespace before a publisher is connected.

Focused evidence (2026-11-15): four namespace cases pass for deleted-source refusal
against disk and pinned fallbacks, moved-source analysis, invalid/contradictory
absence and the construction-reading request. Three existing construction-reading
cases continue to pass. Four recovery-store cases pass for exact states, no-clobber
receipts, symlink/non-directory refusal and replaced batch identity. The actual
engine protocol witness passes deleted/moved construction analysis alongside the
resource acquisition/planning checks, with saved source and destinations unchanged.
Resource UI namespace application, saving and coordinated resource undo remain
required across all platforms.

## Analyze candidate construction ownership before resource publication

The shared resource model revalidates complete acquired source/saved/ownership
facts before evaluating a proposed namespace. Fresh acquisition identities do not
replace the reviewed plan’s identities. Compare source text, version stamps,
capabilities, exact saved states and held root scope; unavailable targets cannot
be republished as nominal no-ops.

Analyze both the candidate saved construction layer and supplied unsaved drafts.
A surviving entry follows its source identity through a move. Newly declared
project roots are analyzed, removed entries stay absent, and a new construction
fragment needs context in at least one candidate layer. The response reports the
exact entries and their compiler output paths, checks saved dependencies again,
and names proposed targets that would become generated. A fingerprint identifies
these facts; it grants no publication permission and is not workspace-wide
project discovery. No source, generated output or index is modified by analysis.
This optional capability runs independently of the filesystem write gate.

Focused evidence (2026-11-15): seven candidate cases cover final generated
destinations, source-only construction changes, moved/removed entries, broken
dependencies, draft-owned fragments, new project roots, fresh capture changes and
unavailable no-op targets. The write-gate observation cases cover this named
route. The real-engine protocol witness detects a newly generated destination in
both candidate layers while retaining all saved files. Resource namespace UI
application, guarded saving/recovery and coordinated resource undo still remain
required across all platforms.


## Retain draft-aware resource recovery and explicit publication outcomes

The recovery record now retains both the reviewed draft transition and exact saved
before/after states. Draft source identities and preparation versions survive
serialization separately from the saved baseline. The pure record constructor
follows source identity permissions through moves and gives newly created source
private permissions. Records validate complete reviewed paths, final saved text
and saved fingerprints before recovery IO.

A separate publication core checks current source paths and root identity, retains
the record before writing, installs files before deletions and reports confirmed,
refused or uncertain outcomes. An independent edit to a later path stops that path
without erasing earlier confirmed saves. A caller must still hold mutation ordering
and supply current editor, live-room and construction authority. This core exposes
no resource-save HTTP route. It is not an external-filesystem transaction, frontend
resource namespace application or coordinated saved/draft undo.

Focused evidence (2026-11-15): six recovery cases and five publication cases pass,
including retained dirty drafts and original saved baselines, inherited permissions,
create/delete, a new destination racing preflight, stale initial source and partial
publication that preserves an independent later edit. Frontend resource application,
recovery UI and saved/draft undo remain required on every simple platform.

## Publish reviewed resource sources through the engine

The explicit engine resource-save endpoint repeats candidate consequences and
fresh acquisition, refuses changed reviews and generated targets, and uses the
shared checked publisher under process-lifetime mutation ordering. Markdown
replacements compare live room text; deletion retires the old room. Sources
addressed after startup are registered in both the workspace index and shared
store before room access. The save request identity binds its entire typed review;
repeated identical requests return the retained outcome, and a read-only status
query never repeats publication. Missing or uncertain results require inspecting
recovery. Retention and dependency IO have explicit bounds.

The rebuilt real-engine witness passes dirty-draft move/save, exact saved/draft
recovery, duplicate/status responses, changed-request refusal, stale saved-source
and candidate-generated destination refusal, and a newly discovered Markdown
source move. Five publication safety cases continue to pass after introducing the
live-document adapter seam. This establishes an engine save path, not frontend
resource namespace adoption, recovery UI, or coordinated saved/draft undo. Those
remain required on browser, macOS, iOS and Android.

## Analyze saved restoration separately from its unsaved draft

An internal consequence-analysis seam can project complete saved restoration
states independently of the reviewed draft transition. Saved fingerprints and
namespace presence must be valid. Compile the layers independently, bind the
saved projection into its consequence fingerprint and protect both output sets.
Construction consumption requires the actual candidate text hash, not merely the
same source address. The public ordinary-save API still saves its reviewed draft
and accepts no arbitrary saved override.

Focused evidence: the restoration projection case passes for distinct saved/draft
outputs, protected-target union, changed fingerprint, incomplete projections,
contradictory absence, false hashes, and an unconsumed draft at an address where
older saved text was consumed. Seven existing candidate cases pass. Historical
invalid draft restoration, recovery admission, UI namespace adoption and
coordinated saved/draft undo remain outstanding.

## Inspect retained resource recovery without replay

The internal recovery reader validates a complete manifest and bounded receipts
without creating missing cache directories or changing source. Exact batch IDs,
ordinary-file checks, held directory identities and Unix no-follow file opens
protect evidence access. It reports unrecognized files and fingerprints the
manifest and observed receipts. Recorded applied outcomes are historical evidence;
they do not claim current disk equality or authorize replay.

Nine focused recovery-store cases pass, including three reader cases for partial
and complete receipt sets, retained dirty drafts, independent current source,
unknown/oversized records, changed fingerprints and Unix symlink refusal. A
recovery endpoint, review UI and coordinated saved/draft restore remain required.

## Keep canonical entry ownership stable during resource saves

The default main construction remains address-bound to `project.md`. Until a
workspace-entry transition is implemented, resource save refuses to move/delete
that entry or a physical alias when it is the protected canonical construction.
Read-only consequences may describe a hypothetical entry move; they do not grant
save permission. The rebuilt real-engine witness confirms this refusal leaves
both source and the recovery record inventory unchanged, while ordinary dirty
source and authored-document moves continue to pass. Explicit workspace-entry
changes remain outstanding.

## Prepare a resource restoration from current authority

The pure restoration model compares current saved states against the retained
result, normalizes historical identity labels and validates the complete inverse
against a fresh current capture. Its plan retains current preparation stamps
rather than borrowing historical counters as authority. Saved before/after states
remain separate from dirty draft states and flow through the shared candidate
construction analysis. Internal normalized inputs are not serialized as authority.

Both focused model cases pass: separate saved/draft restoration and construction
analysis leave source/recovery records unchanged; independent draft text, saved
metadata, permissions, versions and root scopes refuse. Restoration-aware durable
publication, recovery endpoints and UI namespace/undo adoption are still required.

## Retain restoration-aware saved/draft publication

Version-3 recovery records retain the old saved baseline separately from the
restored dirty draft and reference the parent evidence identity/fingerprint.
Version-2 ordinary saves still require their saved after text to equal the
reviewed draft. Both validate complete paths, saved hashes and namespace kinds.
Parent evidence is not write permission; current admission remains required.

Four focused restoration cases and nine recovery-store cases pass. A temporary
root performs save, saved-baseline restore and redo through the shared publisher;
its restored editor snapshot remains dirty, parent records remain unchanged, and
invalid versions/origins/hashes refuse before new recovery IO. Recovery HTTP
admission, live-document restoration integration and frontend namespace/undo
adoption are still required.

## Admit recovery through the shared engine publisher

The engine now exposes read-only recovery metadata and explicitly selected draft
text, read-only restoration preparation, and guarded restoration publication.
Both parent evidence and candidate consequences are checked again after review.
Restore uses the same current source, root, dependency, static-output and
live-document publication guards as ordinary resource save, retaining one outcome
per request identity. Complete results expose confirmed saved baselines; partial
results expose intended states with a null confirmed map. Historical draft plans
remain separate from physical saved baselines.

The rebuilt actual-engine witness passes inspection, independent saved-edit
refusal, changed evidence/consequence refusal, saved-baseline restore with a
separate dirty draft, redo, repeated-request identity and status. A second real
engine witness supports undoing canonical entry creation to absence and refuses a
historical canonical move without changing files or recovery inventory. Two
outcome/attempt, four write-gate and four startup cases pass. Invalid historical
draft layers, partial-state recovery, persistent interrupted-request discovery and
frontend namespace/undo adoption remain outstanding across the simple platforms.

## Read retained resource recovery through existing menus

All simple hosts expose **View → Source recovery** as a floating reading over the
existing editor. The shared engine lists bounded, sorted recovery identities
without claiming time order or successful publication. Selecting one validates
the retained evidence; snapshots distinguish saved baselines from dirty drafts,
before from after, and empty text from absence. Readers reject retired requests,
changed workspace sessions and mixed evidence fingerprints. Discovery/inspection
remain independent of unrelated workspace preparation and do not authorize writes.

The real browser witness passes exact named snapshot selection and preservation
of editor identity, dimensions, newer dirty text and undo/redo. Two focused model
cases pass per frontend, two shared-store discovery cases pass, and TypeScript,
Apple/Android compilation and the iOS simulator build pass. The service witness
also reads discovery and separate saved/draft text through the actual engine.
These checks establish a recovery reading, not frontend restoration, ordinary-save
local history, crash/session recovery or native physical-input acceptance.

## Route resource identities before frontend namespace adoption

The [resource editor routing model](resource-editors.md) computes the complete
before/after address transition on browser, macOS/iOS and Android. Moves and swaps
return the original editor handles. Deletions return handles for retention;
restoration accepts an explicitly supplied retained handle, and path reuse does
not infer identity. Duplicate paths/identities, missing or stale captures and one
handle assigned to two sources refuse before any host mutation. Empty new source
remains distinct from absence. Apple compares address bytes without Unicode
canonical folding.

Focused model evidence (2026-11-15): three browser cases and two cases per native
implementation cover routing, explicit restoration, retained selection/scroll/undo
state and complete refusal. The browser also retains an actual CodeMirror state
and its undo depth through move and inverse routing. Apple and Kotlin cases run
as isolated source-model targets; these are not full app/device builds. TypeScript
checking, reconstruction and ownership audit pass. The model performs no IO or
editor mutation. Confirmed-receipt adoption into live buffer/tab namespaces,
renderer retention during deletion, language-session rebinding, menu-driven
resource apply/restoration and coordinated saved/draft undo remain required on
all hosts. This evidence does not establish frontend resource apply/undo parity.

## Confirm resource receipts before adopting live buffers

Browser, Apple and Android now provide live buffer adoption seams over the routing
model. Complete request/workspace/context identity, every applied path, the exact
reviewed saved states and surviving file metadata are checked before mutation.
Moves retain the buffer and cached renderer identity; deletion holds the buffer
outside the writable namespace, and restoration requires an explicitly retained
handle. Saved baseline text and draft text remain independent. Local draft or
baseline changes refuse adoption. Namespace changes retire pending opens, native
language sessions rebind to current addresses, and deleted dirty buffers still
participate in unsaved-change guards. Android viewport state lives in its buffer.

Focused evidence: five browser cases include live CodeMirror/buffer adoption with
retained selection, scroll and undo depth, separate restored baseline/draft text,
and partial/stale receipt refusal. Three native cases per implementation cover
receipt admission and opaque handle routing. TypeScript checking, Xcode-backed
macOS compilation/selected XCTest, Android compilation/selected JUnit, and the iOS
simulator build pass. Native live renderer adoption is not established by those
model checks. Resource publication callers, tab-address adoption, existing-menu
Save/restoration and coordinated saved/draft undo remain outstanding on all hosts;
the new seams alone do not complete the requested IDE parity contract.

## Retain sealed proposal readings across engine restarts

Proposal retention now writes a bounded, authenticated private catalogue tied to
the physical workspace and held storage identities. Reopening retains the exact
original material, fingerprint, scope and explicitly unverified host report. A
historical reading carries a separate current reading scope; it cannot populate
live admission state, acquire editor authority, Apply or report a new application.
Missing, substituted, oversized or damaged storage refuses with an explicit
retention notice while a separately prepared live review remains usable. Private
prediction requests never create this store.

Focused evidence (2026-11-17): fourteen Rust proposal/storage cases pass, alongside
the existing HTTP flow and genuine restart/tamper protocol. Browser proposal
models pass23 focused cases with TypeScript checking. Native model groups pass
with their explicitly environment-dependent live cases executed separately. The
actual browser, macOS and Android restart flows each read the original sealed
proposal beside a current dirty editor, disable Apply, retain editor identity and
geometry or native state, preserve Undo and leave saved source unchanged. The
macOS and iOS simulator product builds pass. These receipts do not establish native physical input,
assistive technology or device interaction acceptance. Other open parity rows
remain required.

## Prepare common snippet transformation and computed-field models

The shared engine and three host models now have bounded transformation programs,
Unicode16 case/word/fold data and version2 computed-field descriptors. Field edits
recompute dependent text as one proposal; computed text is excluded from field
traversal, and direct edits inside it retire linked intent while ordinary host
input remains available. Native and browser undo retain the linked session with
its exact text history. Version1 clients refuse computed descriptors, including
ones whose initial text happens to equal a normal field. Cycles, malformed
programs and shared work budgets refuse without partial edits.

This is infrastructure and editor integration evidence, not completion of general
snippet syntax. The production parser still refuses transforms; source-bound
variable contexts, resolver/template integration, multi-caret templates and
mirrored native IME remain required. Frozen Unicode data avoids host locale and
OS version differences without making a claim of language-provider completeness.

Executed focused checks: browser83 case/transform/derived/compatibility checks and
16 interaction checks; Rust5 transformation cases covering32 shared vectors and
one case-table test covering13 vectors; macOS21 tests across the new models and
existing expansion/session/completion/editor interaction paths. The computed-field
AppKit interaction test uses actual native edit/history. Android20 focused tests
pass with no skips, including computed input and Compose controller history. The iOS simulator product
build passes. Physical input, assistive technology and device IME acceptance remain
unverified.

## Read semantic refactoring edits before applying them

A held TypeScript refactor action can now use the provider's advertised
`typescript.tsserverRequest` interface for `getEditsForRefactor`. The engine never
dispatches `_typescript.applyRefactoring`, which can create files before client
review. Original provider arguments stay bound to the queried virtual source and
held child. Graph/source proofs are checked before and after the read; resulting
edits rejoin complete source acquisition, authored projection and guarded review.
Unknown commands and edit-plus-command combinations retain explicit refusal.
New files decode only as ordered proposals; this alone does not verify a move.

TypeScript applies ordered changes in reverse. Genuine accessor generation
exposed coincident getter/setter insertions that the shared overlap policy rightly
refused. The adapter now combines only exactly adjacent ordered changes into one
unambiguous edit. Unrelated overlaps and malformed later members still refuse the
whole reading; the general projection overlap guard is unchanged.

Executed evidence (2026-11-17): four focused Rust policy/decoding cases pass,
including the reproduced boundary insertion and malformed later member. The
existing real opaque-action protocol continues to pass original routing, caller
substitution refusal and stale/foreign token refusal. Genuine TypeScript Language
Server6.1.0 with TypeScript6.0.3 passes extraction and getter/setter generation
through browser, mounted macOS and Android state/controller previews. Each host
retains an earlier unsaved edit, original editor identity and geometry or native
state, and coordinated Undo/Redo; saved source stays unchanged. The resulting
drafts separately pass the installed TypeScript compiler. The iOS simulator product
build passes. These receipts do not certify other command/provider families,
native physical input, assistive technology or device interaction.

## Reach and manage every retained private editor checkpoint

Browser, Apple and Android now page through the bounded32-session store and
select each session's three retained revision identities. Reads remain exact to
revision, request and snapshot hash. Explicit two-step Forget removes only the
selected whole private session, freeing capacity without saving or changing source.
Its current-runtime/root/window/directory/stat/hash guard refuses changed or
replaced records. A compare-after-rename check preserves an injected replacement;
exact lost-response replay returns the old receipt without deleting a new record.
Bounded corrupt ordinary records may be explicitly forgotten without importing
untrusted snapshots. Symlinked or oversized records remain explicit refusals.

The private directory resolver creates nothing by itself. Held no-follow directory
operations protect checkpoint/removal paths, and publication keeps its separate
source authority. Pending removal pauses checkpointing and retains its exact retry
intention. A verified own-session removal retires the old writer. Paging failures
preserve the previously loaded catalogue; optional recovery failure leaves current
code and drafts available.

Executed evidence (2026-11-17): thirteen backend safety cases and fourteen browser
model cases pass; web type checking passes. Focused native groups pass with the
live-fixture skips recorded separately. The actual service and rendered browser
flows pass32-session capacity, corrupt-slot removal, resumed automatic checkpoint,
all three exact readings and guarded recovery. The mounted macOS flow executes
without skips through a real WorkspaceView/popover and NSTextView; viewport is
checked across readings, while restore/Forget retain the original editor, draft,
selection and native Undo. Android's integrated Engine/Workspace/TextFieldState
flow also executes without skips. The iOS simulator product build passes. Saved
source stays unchanged. Physical input,
assistive technology, Android rendered acceptance and iOS runtime acceptance remain
unverified; restoration and private Forget confer no source-save permission.

## Complete source actions and original editor intentions

The shared action menu now reaches source cleanup and generation through the
provider's standard `source` action family on browser, Apple and Android. Requests
cover the complete authored file. Family checks survive lazy resolution and fresh
acquisition; missing provider support produces an explicit unavailable result.
The narrower fix-all command retains its own family restriction. Proposals retain
the existing source proofs, preview, guarded application and coordinated history.

Completion and template offers also retain their original text and editor selection
intention. Moving the cursor or replacing source invalidates an old offer even if
later text happens to match. Native production offer construction supplies these
stamps; optional stamps remain confined to explicit test seams.

Executed evidence (2026-11-17): the focused browser groups pass47 cases, and the
backend source-action group passes12. Focused Apple and Android model groups pass
with live-fixture skips recorded separately. Genuine TypeScript extraction,
getter/setter generation and unused-import cleanup each pass through browser,
mounted macOS and Android controller/state previews, preserving an earlier draft
and coordinated Undo/Redo. The resulting drafts separately pass the TypeScript
compiler; the iOS simulator product builds. These receipts do not establish
physical input, assistive-technology, Android rendering or iOS runtime acceptance.
Full snippet context and production transform activation remain separate open work.

## Protect unproduced names inside declared output folders

Resource review now treats literal declared volume folders as reserved output
scopes, including filenames not yet produced. Saved, dirty and candidate source
readings contribute these scopes while compiler output inventories continue to
report only produced files. Exact folder membership preserves similarly prefixed
ordinary siblings. Symlinked or aliased physical scopes and unresolved declarations
refuse guarded publication without blocking readable source.

Executed evidence (2026-11-17):23 focused resource-input cases pass, including five
new saved/draft/candidate scope and physical-path checks. The actual HTTP witness
refuses forged creation and generated-folder publication without source or recovery
writes. The maintained resource service flow also passes explicit loaded-source
guards, stale inverse refusal after an equal-byte external write, a freshly guarded
positive inverse/redo pair and a genuine engine restart. These checks establish the
resource-publication path; they do not assert an audit of every independent writer.

## Preserve exact native test selection and visible terminal results

JavaScript discovery and execution share one producer-native driver. Exact source
file/name pairs prevent crossed failures from widening into unrelated bodies.
Native hierarchy names, literal separators, whitespace, metacharacters and empty
names remain distinct. Source-adjacent Run shares the exact file selector and saved
source preflight, while an empty name cannot silently mean a containing-file run.

Failed-only reruns reacquire no-body native discovery under the original physical
root. A real code fix can start a new run without changing the historical result;
missing or ambiguous identities refuse before bodies. C# filtered discovery must
match the exact requested set despite native case-insensitive matching. Coverlet
report correlation follows the producer's TRX deployment relationship, never an
equal-byte guess. Partial results remain attributable to their original root.

GET and subsequent rerun now select the same retained-first result. A live Vitest
whitespace case exposed terminal results becoming visible before the durable
running record was replaced; the shared reader closes that race.

Executed evidence (2026-11-17):31 focused collector cases pass, with seven explicitly
opt-in live cases ignored in that model run. Eleven separate fresh-engine Vitest
flows then pass, including fixed-code three-session histories, literal hierarchy,
empty/whitespace cases, missing/ambiguous identities, source preflight and root
retirement. Earlier executed receipts include ten Jest flows, two installed Rust
flows, the unchanged Python fix/rerun flow and two C# fixture flows; those preceded
the final shared retained-reader-only correction. Six coverage UI cases and web
type checking pass. These receipts do not certify arbitrary projects, other
runner versions, native physical input or device acceptance.

## Keep authored bytes authoritative beside a normalized browser view

The browser editor now keeps exact authored separators independently of normalized
CodeMirror display text. Shared UTF-16 conversions map selection, language queries,
diagnostics, snippets and provider edits across that boundary. Exact raw edits and
inverse history effects preserve mixed CRLF/LF text through typing, paste and
Undo/Redo, including changes that affect separators alone. Revision-local indexes
bound repeated attached offset lookups; split-surrogate and CRLF-interior authored
positions refuse rather than rounding an edit into another source location.

Review, resource adoption, recovery, source history, coverage/test guards and agent
application consume the same authored source. Rich inline parsing uses normalized
display text without gaining source-write authority. Completion and choice intent
also advances for raw-only or field-only transitions, so Undo cannot revive an old
offer. Explicit completion can query blank, member and Unicode positions; implicit
triggers must belong to the captured producing child. Unsupported implicit and
invalid requests do not reach that child.

Executed evidence (2026-11-17):115 focused browser cases and web type checking pass;
focused positions and common-query groups also pass. The rendered mixed-line-ending
flow passes original bytes, typing, local Enter, mixed paste, exact selection,
genuine TypeScript extraction/proofs and coordinated history, preserving the editor
and geometry while disk stays unchanged. Actual normal and nested completion-popup
flows pass primary/import edits, mirrors, caret/history and held-reply cancellation.
The fresh-engine provider protocol passes explicit/declaration-bound triggers and
unsupported/invalid child-RPC absence. This is browser adapter acceptance, not
physical-input, IME, assistive-technology or device acceptance.

## Serialize Android language transport retirement

Language replies, cancellation cleanup, source attachment and connection shutdown
now share short state admission. Pending reply completions, subscriber callbacks
and socket retirement run after the monitor is released; no coroutine waits under
it. Failure callbacks invalidate every subscriber before pending callers resume.
A retired opening epoch cannot publish its returned socket, and one throwing
subscriber does not prevent remaining invalidation or reconnect-gate release.

Executed evidence (2026-11-17): six focused concurrency cases and the existing real
WebSocket replay/detach flow pass without skips. The new cases include24 rounds of
64 pending requests racing close, replies and cancellation,128 concurrent source
owners, reentrant opening retirement, old-epoch isolation and invalidation order.
The genuine five-case TypeScript Android host flow then passes, including shutdown,
resource publication/inverse/redo, CRLF and supplementary Unicode, retained editor
state and separate resulting-draft compilation. This is controller/transport
acceptance; rendered Android, physical input and assistive technology remain
unverified.

## Publish and reverse a genuine provider move into a new file

The TypeScript read-only adapter now recognizes the native singleton zero-position
new-file sentinel without an `isNewFile` flag. It produces an ordered creation and
text edit for shared acquisition; it never creates a file while resolving. Mixed
or repeated sentinel edits and an explicit contradictory existing-file marker
refuse the complete reading. Current missing-destination, ownership and permission
guards still authorize publication separately.

Executed evidence (2026-11-17): six focused decoder cases pass. Five genuine
provider cases pass through browser, mounted macOS and Android controller/state:
CRLF/supplementary-Unicode extraction, ordinary extraction, accessor generation,
move to a new file and source cleanup. Preview leaves saved source unchanged.
The move explicitly saves the reviewed namespace; reviewed Undo restores the
original saved source and dirty draft while removing the destination, and reviewed
Redo republishes it. All three flows preserve the original editor identity/state
and coordinated history. Resulting drafts separately pass the TypeScript compiler;
three retained namespace journals account for Save/inverse/redo. Android's shutdown
race exposed by this flow is covered by the preceding transport receipt. Other
provider move variants and physical/device/accessibility acceptance are not
established by these cases.

## Share explicit variable snapshots and bounded demand mechanisms

The common snippet mechanisms now bind variables to exact authored source,
selection, path and workspace. A host captures its clock/offset and random seed
once per intention; the engine never substitutes its own local clock. Presence
analysis follows reachable defaults and rendered field dependencies before value
materialization, so an unused oversized value or clipboard default cannot force a
read or reject an unrelated expansion. Unsupported syntax and value/work limits
refuse the complete expansion.

Browser, Apple and Android share the same source-proof and two-step clipboard
handshake policy. Clipboard is read only after an exact supported demand, at most
once, while the original lease remains current. Foreign proof, repeated demand,
source/selection retirement and denied reads retain explicit outcomes.

Executed evidence (2026-11-17): eleven parser and five context cases pass through
the generated Rust modules. Browser handshake cases and both native handshake
classes pass; web type checking and the iOS simulator product build pass. These
are shared mechanisms and protocol models. Production completion/template
integration and usable provider flows remain a separate acceptance step; these
receipts alone do not establish complete template parity.

## Apply source-bound variable and computed-field expansions through each host

Completion and reusable templates now use the captured source/selection context
and bounded transform programs through their production adapters. Preview and Apply
retain one clock/seed and clipboard outcome. An exact demand may read clipboard
once; denial retains defaults and a notice, an unused default performs no read,
and an obsolete lease refuses before transmitting or applying context. Hosts reject
foreign proof, repeated requests, extra effects and changed template/context data.
Clipboard admission rejects oversized values before unbounded scans or encodings.

The real macOS flow exposed a planner dropping computed-field programs while
rebasing fields. Preserving that metadata now keeps ordinary mirrors and derived
Unicode fields live after insertion. Connected planning/session/input cases cover
this handoff on all three hosts.

Executed evidence (2026-11-17): generated Rust parser11/context5/trigger1 and engine
template-service8 cases pass; browser focused36 and type checking pass. Swift12
and Android13 focused checks pass, with Android's one opt-in model-run skip
executed separately in its owned fixture. Fresh-engine protocol acceptance proves
one child resolve across a demanded clipboard retry, denial/default behavior,
unused-default privacy, exact physical producer identity and foreign-proof refusal;
the legacy protocol remains compatible. Rendered browser provider blank/selected
and template Preview/Apply flows pass variables, computed Unicode mirrors,
Undo/Redo, editor identity/geometry and unchanged CRLF source bytes. Four native
real-engine flows pass exact1/zero skips each: templates and provider completion
on mounted macOS and Android controller/state. Android retains producer XML;
macOS's SDK emits console results, retained as explicitly labelled console JSON
receipts with verified log hashes. The final iOS simulator product build passes.

These receipts establish the supported bounded language. Full ECMAScript regex,
embedded-language template routing, multi-caret templates and mirrored native IME
remain required open work. Physical input, OS clipboard permission prompts,
assistive technology, Android rendering and iOS device/runtime acceptance remain
unverified; they are not inferred from controller or build results.

</ui:doc>
