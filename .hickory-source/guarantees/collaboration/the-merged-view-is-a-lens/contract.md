<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Merged View Is A Lens Over Several Worktrees, Never A Document

For: Engineers using local sessions or pairing their own machines.

Given a file that exists in several worktrees of one repository, when a merged
view is opened over it, then regions **every** source agrees on appear once,
regions that differ appear as variants naming each source, and the view says
plainly that it is read-only by default and exists on disk nowhere. Explicit
region edits name the targets opened for writing and use the captured source
preconditions; they do not give the lens a save path.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-merged-view-is-a-lens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="f-87756f3e699e-git"></a>
### `git`

The merged view over two real local worktrees, read-only.
Protects `docs/guarantees/collaboration/the-merged-view-is-a-lens.md`.
This is where the alignment risk lives: a bad alignment routes an edit
silently into the wrong file, so the first step is deliberately read-only
and deliberately conservative — fewer regions declared shared, more shown
as variants, because the failure of over-sharing is an edit in the wrong
place and the failure of under-sharing is a little redundant typing.

<storycode:copy id="f-87756f3e699e-git">
//! The merged view over two real local worktrees, read-only.
//!
//! Protects `docs/guarantees/collaboration/the-merged-view-is-a-lens.md`.
//!
//! This is where the alignment risk lives: a bad alignment routes an edit
//! silently into the wrong file, so the first step is deliberately read-only
//! and deliberately conservative — fewer regions declared shared, more shown
//! as variants, because the failure of over-sharing is an edit in the wrong
//! place and the failure of under-sharing is a little redundant typing.

use std::path::{Path, PathBuf};

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

</storycode:copy>

</storycode:doc>
