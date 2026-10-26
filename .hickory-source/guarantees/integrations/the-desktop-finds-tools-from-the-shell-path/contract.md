<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Find installed tools when the desktop starts outside a terminal

For: Engineers connecting repository issues, reviews and locally installed tools.

Launch from Finder after configuring a language tool in your login shell. Tool discovery should use the resolved search path without waiting for shell logout hooks.

## What must happen

Desktop launch resolves an absolute executable search path through the supported login/interactive shell setup within its bound. Discovery replaces the shell with the path-reporting command after startup profiles, avoiding logout hooks. A new window reuses a parent path only when its marked resolved value is unchanged; altered or unmarked values are rediscovered. Discovery failure retains explicit fallback behavior rather than blocking editor startup indefinitely.

## Limits of the evidence

Shell profile side effects remain those of the user's configured shell. Native startup and each platform need their own acceptance evidence.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/the-desktop-finds-tools-from-the-shell-path.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Include the October 17 shell replacement and parent-path reuse amendment; keep startup responsiveness separate from later workspace preparation.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/launch_path.rs](../../../mechanisms/apps/desktop/src-tauri/src/launch_path.rs.md#output-9697ca6e6f7a)
- [apps/desktop/src-tauri/src/main.rs](../../../mechanisms/apps/desktop/src-tauri/src/main.rs.md#output-981e94b5aacd)
- [crates/hick-term/src/session.rs](../../../mechanisms/crates/hick-term/src/session.rs.md#output-5705d2b8acc0)
- [crates/hick-term/src/shell_integration.rs](../../../mechanisms/crates/hick-term/src/shell_integration.rs.md#output-3465af886d7a)
- [crates/hickory-cli/src/engine/client.rs](../../../mechanisms/crates/hickory-cli/src/engine/client.rs.md#output-5c9c7cadce49)
- [crates/hickory-cli/src/serve/github.rs](../../../mechanisms/crates/hickory-cli/src/serve/github.rs.md#output-1de81b976483)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
