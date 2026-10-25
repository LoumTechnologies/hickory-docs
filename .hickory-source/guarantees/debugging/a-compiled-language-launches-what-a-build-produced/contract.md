<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Launch the artifact built from the selected compiled project

For: Authors debugging generated code while navigating its document source.

Debug a Rust or C# program described by the current project. The debugger must receive the selected build artifact rather than a guessed source-file stem.

## What must happen

Compiled-language launch uses the language's declared build step, artifact location and executable selection. Rust builds in its manifest directory, uses the package manifest to identify the program, and excludes dependency-note files from executable selection. Adapter discovery prefers a supported installed adapter; catalogue installation can supply supported archive-based C# and Rust adapters. Build failure retains actual diagnostics rather than starting a different program.

## Limits of the evidence

A successful build is not behavioral verification. Platform adapter availability and live stop/source mapping need independent witnesses.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-compiled-language-launches-what-a-build-produced.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold Rust/codelldb and the available C# archive installer into the earlier compiled-language contract. Retire the obsolete claim that C# has no installer.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
