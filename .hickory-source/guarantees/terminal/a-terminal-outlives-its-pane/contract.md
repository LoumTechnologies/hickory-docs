<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Terminal Outlives Its Pane

For: Authors running interactive tools alongside documents without losing session state.

Given a terminal session started through `POST /api/terminals`, when its tab
is closed — or the window is rearranged, or nobody is attached at all — then
the process keeps running, its output keeps accumulating server-side, and a
client that attaches later is sent everything the session has already said
before it is sent anything new.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-terminal-outlives-its-pane.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="f-31c9ccf64945-installed"></a>
### `installed`

Real full-screen programs, in a real PTY.
The tests above this one prove that bytes move. These prove the harder
thing: that a program which takes over the whole screen — an editor, a
process monitor, a coding agent's UI — gets a terminal it believes in.
Those are the programs a half-built PTY breaks: they ask the terminal its
size, switch to the alternate screen, position the cursor absolutely, and
stop drawing entirely if any of it is missing.
What is asserted is the server's own screen model (`vt100`) after feeding
it exactly the bytes the client would receive. If the model shows the
program's interface, the client's emulator has everything it needs to draw
it — the two are parsing the same stream.
Every test skips when its program is not installed, because these run on
whatever machine happens to have the checkout.
Protects docs/guarantees/terminal/a-terminal-outlives-its-pane.md

<storycode:copy id="f-31c9ccf64945-installed">
//! Real full-screen programs, in a real PTY.
//!
//! The tests above this one prove that bytes move. These prove the harder
//! thing: that a program which takes over the whole screen — an editor, a
//! process monitor, a coding agent's UI — gets a terminal it believes in.
//! Those are the programs a half-built PTY breaks: they ask the terminal its
//! size, switch to the alternate screen, position the cursor absolutely, and
//! stop drawing entirely if any of it is missing.
//!
//! What is asserted is the server's own screen model (`vt100`) after feeding
//! it exactly the bytes the client would receive. If the model shows the
//! program's interface, the client's emulator has everything it needs to draw
//! it — the two are parsing the same stream.
//!
//! Every test skips when its program is not installed, because these run on
//! whatever machine happens to have the checkout.
//!
//! Protects docs/guarantees/terminal/a-terminal-outlives-its-pane.md

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_term::session::Session;
use hick_term::{SessionSpec, TermConfig, Terminals};

/// Is `program` on this machine's PATH?
///
/// Walked directly rather than asked of a shell. `sh -c "command -v x"` was
/// the old spelling and it fails to SPAWN on a machine with no `sh` — which
/// `unwrap_or(false)` then read as "not installed", so every test in this file
/// skipped on Windows without printing a word. A silent skip is the one
/// outcome a suite must never have: it is indistinguishable from a pass, and
/// this file would have stayed green with the PTY layer entirely broken.
fn installed(program: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    // `PATHEXT` is how Windows decides what "executable" means; a bare name on
    // PATH there is `vim.exe` or `less.bat`, never `vim`.
    let suffixes: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(str::to_string)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&paths).any(|dir| {
        suffixes
            .iter()
            .any(|suffix| dir.join(format!("{program}{suffix}")).is_file())
    })
}

</storycode:copy>

</storycode:doc>
