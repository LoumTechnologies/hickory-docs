<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Debugger That Lives In A Language Server Is Asked For

For: Authors debugging generated code while navigating its document source.

Given Java, when a debug session starts, then hick starts **eclipse.jdt.ls**
with the `java-debug` plugin named in its `initializationOptions.bundles`,
waits for the server to report `ServiceReady`, sends
`workspace/executeCommand` for `vscode.java.startDebugSession`, and connects
to the **port** in the reply. From there it is ordinary DAP: breakpoints in
document coordinates, the stack, values, stepping.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-debugger-that-lives-in-a-language-server-is-asked-for.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
