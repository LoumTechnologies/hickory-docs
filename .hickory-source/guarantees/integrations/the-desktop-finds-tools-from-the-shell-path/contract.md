<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Find installed tools when the desktop starts outside a terminal

For: Engineers connecting repository issues, reviews and locally installed tools.

Launch from Finder after configuring a language tool in your login shell. Tool discovery should use the resolved search path without waiting for shell logout hooks.

## What must happen

Desktop launch resolves an absolute executable search path through the supported login/interactive shell setup within its bound. Discovery replaces the shell with the path-reporting command after startup profiles, avoiding logout hooks. A new window reuses a parent path only when its marked resolved value is unchanged; altered or unmarked values are rediscovered. Discovery failure retains explicit fallback behavior rather than blocking editor startup indefinitely.

## Limits of the evidence

Shell profile side effects remain those of the user's configured shell. Native startup and each platform need their own acceptance evidence.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/the-desktop-finds-tools-from-the-shell-path.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Include the October 17 shell replacement and parent-path reuse amendment; keep startup responsiveness separate from later workspace preparation.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
