<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Install a supported missing debugger and resume the requested launch

For: Authors debugging generated code while navigating its document source.

Choose Debug on a supported program with no adapter installed. Setup should continue that launch without requiring another Debug click, and dismissal must block a late launch.

## What must happen

A typed catalogue-supported missing-adapter result starts the existing confined installer and then retries the originating program. Setup exposes actual progress and diagnostics. Failure offers an explicit retry; repeated missing-adapter results do not cause an automatic installation loop. Dismissed or replaced setup cannot start a late session. Requests for the same adapter share installation. Unsupported adapters retain their ecosystem-specific guidance. Installer paths are shell-quoted and redirected home applies to the complete compound install.

## Limits of the evidence

The UI witness uses a controlled installer response. It does not download an adapter, establish platform confinement or certify a packaged debugger.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-missing-debugger-is-a-button.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The current Debug action authorizes prerequisite setup and supersedes the former separate-install-button decision. Keep retry, cancellation and typed-error boundaries in the current statement.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
