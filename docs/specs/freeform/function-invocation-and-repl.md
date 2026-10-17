# Invoke a function in a live REPL

**Status: proposed design, not implemented.** Requested 2026-10-17. This document
is for maintainers designing Hickory's function invocation, launch discovery and
runtime integrations. It specifies intended behavior, not a shipped guarantee.
No runtime, provider, launch-profile format or UI implementation is added here.

A developer should be able to explore `welcome("Hickory")`, adjust its arguments,
and stop inside it without writing a throwaway application. The interaction must
use a real language session and must never insert rows into the source editor.

## The first interaction

Beside `def welcome(name):`, a gutter action offers **Invoke function…**.
It opens a small anchored popover with the function name, its arguments, the
selected session, and **Run** and **Debug**. For this function the only argument
field is `name`, initially empty; an example value is `"Hickory"`. Opening the
popover discovers information but does not import modules or execute code.

Run submits `welcome("Hickory")` to the selected REPL. Debug submits the same
invocation with the debugger connected and applicable breakpoints installed
before execution. A fresh session loads the selected project first. The result,
exception and stdout/stderr appear in the REPL pane, associated with this call.
The source editor stays at its current position.

The developer can change the argument and repeat, or type the equivalent call
in the REPL. Both routes use the same runtime, bindings and history. The argument
form is another way to compose a call, not a second execution environment.

Invoking `welcome` directly does not run `app.py`. **Run application** and
**Run test** remain distinct targets. A function declaration that merely defines
code must not be presented as an application whose execution reaches its body.

## Controls occupy the margin and overlays

The existing right action rail is the entry point. A function adds no inline
form, result card, transcript, toolbar or extra source line. Functions outside
the supported callable surface have no misleading runnable action; an explicit
Invoke command can explain what is missing.

Use words in the action's accessible name and tooltip. An existing application
or test play button keeps its meaning. Where actions share an anchor, one compact
popover lists **Run application**, **Run test**, or **Invoke function…** as
applicable. Do not add a row of similar play icons and make the person infer
which one calls a function.

The argument popover floats over the workspace. It stays within the viewport,
scrolls internally if necessary, supports keyboard operation, and closes with
Escape without executing. Dismissing it preserves the draft arguments for that
symbol and project state. Run/Debug never trigger merely because focus changes.
A large object editor opens in the REPL pane instead of growing the popover.

The REPL is an ordinary workspace surface, reused for its session. By default it
opens in a side overlay/drawer so the source layout and scroll position do not
change. Pinning it beside the source is an explicit workspace layout action.
On narrow windows it can temporarily occupy the center and return to the saved
source position. It never appears as a block between source lines.

Results and history stay in that surface. While paused, existing gutter stepping
controls and source highlights are reused; stack and object inspection belong
in the REPL/debug surfaces. No permanent REPL toolbar is added to every editor.

## A call belongs to a project state and a session

A launch profile describes what to start and how: an application, test runner,
or interactive runtime. A resolved project state identifies the exact files
and environment inputs it starts against. A session is the live process created
from that selection. An invocation addresses a callable within that process.
These identities must not collapse into a single filename or version preference.

A new session uses the latest compatible **atomic project state** containing the
viewed callable's source. It excludes halfway states inside an explicit atomic
scope. Initially compatibility uses the entire displayed file; finer function
compatibility would need a separate, demonstrated source-mapping contract.
Captured project files and their source maps remain fixed for that session.
For ordinary workspace projects, the provider must detect source changes and
report whether its loaded-code/source correspondence remains valid.

The popover and REPL header identify the active session and its project reading,
such as **Greeting · before greeting change**. They indicate when a compatible
newer reading exists. There is no global project-version selector.

An existing session is reused only when its loaded project and requested code
match. It must never silently execute an old `welcome` definition beneath new
source. When code changes, present **Restart with current code** and, only if a
provider supports it, **Reload changed code**. The old session and its bindings
are not silently destroyed. Choosing to retain it opens its captured reading for
source navigation and debugging.

The initial implementation should support restart rather than promise live
redefinition. Reload is a capability with language-specific semantics: it must
report which definitions and existing values remain valid. A successful module
reload alone does not establish that old objects, closures or active frames now
use new code. If exact correspondence cannot be established, source debugging
against the new reading is unavailable until restart.

## Arguments and objects

Parameter names, defaults, types and overloads come from the language provider's
semantic information. LSP signatures and the existing code-model servers are
possible inputs. Neither is assumed to exist for every supported language.

Required arguments have no invented values. Defaults can be omitted. Null,
omission and an empty string remain different values. Overload selection is
explicit when it changes the call. Expressions are interpreted using the chosen
language and session; they are never passed through an unrelated shell.

Start with literal values and expressions. Every field has a visible interpretation
so a string is not confused with an expression naming a session variable. Typed
controls may construct literals, while an explicit expression mode permits
`customer`, `make_customer()` or another binding. The submitted history shows the
actual call, including receiver and argument expressions, in evaluation order.

A method can require a receiver. A closure can require a live closure instance.
Dependency injection or framework initialization can require an existing session,
factory or fixture. Let the developer select/provide those values; do not invent
an object graph, invoke arbitrary constructors during discovery, or substitute
nulls until something runs. Unsupported callable shapes get a specific explanation
and a route to invoke through a supported application, test or fixture.

Returned objects stay runtime objects. Inspection uses bounded previews and
session-local handles, rather than converting everything to JSON. Handles become
invalid when the process ends and do not silently bind to a new process. Invoking
an async callable, consuming an iterator or awaiting a task requires an explicit
provider-supported operation; inspecting a value must not silently consume it.

## The language runtime seam

A provider contributes callable discovery, signature/argument representation,
runtime preparation, invocation, value inspection, interruption, teardown and
capabilities. Language-specific schemas remain language-specific, following the
existing code-model design. The shared seam concerns sessions and invocations,
not a universal representation of every language's types.

Requests and responses carry session, project and invocation identities. Events
associate input, output, result, exception, interruption and debugger stops with
the owning invocation where the runtime can establish attribution. Background
output without that association is shown as session output. Late events from an
ended session cannot replace a newer result.

A provider must expose what it can actually do: direct invocation, persistent
bindings, async evaluation, object inspection, debugger attachment, interruption
and optional reload. Discovering a compiler, LSP or debug adapter does not certify
these capabilities. Capability discovery itself must not execute project code.

For Debug, the adapter controls the **same process** that owns the REPL values.
DAP evaluation in an existing paused frame is useful, but does not by itself
provide an idle REPL with persistent bindings. If the runtime cannot support
Debug in this session, say so and offer a separately identified fresh debug
session when supported. Do not secretly switch to an isolated harness and imply
that the existing REPL objects were used.

Runtime loading, imports, factories and invocations may have effects. They occur
only as part of an explicit execution action, with visible preparation failures.
A scratch project protects source files from debugger preparation; it does not
make arbitrary user code side-effect-free. Cancellation reports whether the call
was interrupted and whether the session remains usable; otherwise offer restart.
Concurrent calls in one mutable session are serialized initially. Repeated Run
clicks must not accidentally enqueue duplicate executions.

## Keep an exploration when it becomes useful

A scratch REPL writes no document by default. **Keep invocation…** can create an
ordinary `.md` example containing reconstructible setup, the call and an optional
expected result. This is an explicit authoring action and creates an unverified
example until run through the document's normal execution/verification path.

An in-memory object handle or the printed representation of an object is not
reconstructible setup. Ask for a factory/fixture or retain the invocation only
as a session record with its limitation stated. Editing history composes a new
invocation; it does not rewrite what happened in the running process or prove
that replay from a clean process has the same result.

The existing anchored-terminal mechanism records shell submissions and suspends
receiving them inside a child REPL. Language-level inputs must therefore come
from the runtime provider. Do not infer them from terminal escape sequences or
claim that shell recording already captures the language session.

## Relationship to the existing designs

[A terminal that writes the document](a-terminal-that-writes-the-document.md)
provides the earlier comint/SLIME analogy, shared session state and explicit
recording boundary. This design adds structured language invocation and a
space-preserving workspace interaction. It does not replace anchored shell cells.

The SLIME/Swank-inspired direction here is an editor connected to a live language
process, with source navigation, evaluation, inspection and debugging cooperating.
This is not a researched feature-by-feature comparison, a Swank protocol proposal,
or a claim that Hickory already has Lisp-style redefinition/restarts. A dedicated
comparison can inform provider capabilities later.

[Debug controls preserve editor space](../../guarantees/debugging/debug-controls-preserve-editor-space.md)
supplies the layout constraint. Its historical global-selector and replacement-code
wording is superseded by the newer exact-source design; this proposal adds no
shipped guarantees.

[Launching what a document builds](launching-what-a-document-builds.md) supplies
the build-before-launch boundary. Its prohibition on required launch configuration
must not force discovery to guess a receiver, fixture or runtime startup. Optional
manual launch profiles are a separate design, to be discussed alongside discovery.
This document does not choose their filename or schema.

## Acceptance cases for a future implementation

| Situation | Required observation |
|---|---|
| `welcome("Hickory")` from its declaration | Form and typed REPL call use one session and return the same value |
| Breakpoint on the return | Debug invokes the body and stops on the displayed source |
| Two calls share an object binding | The same runtime object is available to both calls |
| Arguments need a receiver or fixture | Setup is explicit; no fabricated instance is used |
| New code is viewed while an old session lives | No silent old-definition invocation; restart/reload choice is visible |
| Old/new greetings inside one atomic scope | Each new session uses a compatible atomic state, never a half-updated caller/library pair |
| Popover, result, exception or debugger appears | Source line positions and editor text layout do not change |
| Invocation completes after a session was replaced | Its events cannot overwrite the new session's state |
| Invocation requires a non-reconstructible object | Keeping it does not claim a reproducible executable example |

Before advertising a provider, demonstrate these cases with its actual runtime
and debugger. Python is a useful first candidate because the walkthrough is
small; Python support for this feature is not established by existing debugpy
support, and the current code-model servers cover C#, TypeScript and Go.

## Decisions left for the next design discussion

Resolve launch discovery, optional manual profiles and project-state selection
before implementation. In particular: how a target declares its build/setup;
how call-graph evidence ranks applications and tests; how atomic states are
chosen; and how source breakpoints are mapped into every compatible launch.

The walkthrough exposed existing gaps in these areas: library debug actions run
a definition-only module, older-side breakpoints belong to a different captured
project than the old caller, and the current latest-file resolver admits an
intermediate tree inside an atomic scope. This proposal does not fix those bugs
or reinterpret their current behavior as correct.
