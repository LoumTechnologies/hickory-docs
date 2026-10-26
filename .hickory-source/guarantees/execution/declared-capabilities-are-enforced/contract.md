<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Declared Capabilities Are Enforced

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document that declares container capabilities with `hick:allow` /
`hick:deny`, when the pipeline runs it, then those declarations decide what
each container actually gets:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/declared-capabilities-are-enforced.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-a58d8477e5c2"></a>
## `crates/hick-exec/src/volume.rs`

Related contracts: [Declared Capabilities Are Enforced](contract.md).

<a id="f-a58d8477e5c2-VolumeKind"></a>
### `VolumeKind`

Volume system domain types.
Defines volume declarations, access rules, and volume kinds used by
the DAG builder and pipeline executor to manage inter-container data flow.
The kind of volume: input, output, both, or ephemeral.

<storycode:copy id="f-a58d8477e5c2-VolumeKind">
//! Volume system domain types.
//!
//! Defines volume declarations, access rules, and volume kinds used by
//! the DAG builder and pipeline executor to manage inter-container data flow.

/// The kind of volume: input, output, both, or ephemeral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeKind {
    /// Snapshot of an existing host directory (read-only source).
    Input { path: String },
    /// Contents become pipeline file outputs at the given path.
    Output { path: String },
    /// Reads existing state AND writes updates.
    InputOutput { input: String, output: String },
    /// Ephemeral inter-container scratch (no host mapping).
    Ephemeral,
}

</storycode:copy>
<a id="f-a58d8477e5c2-VolumeAccess"></a>
### `VolumeAccess`

<storycode:copy id="f-a58d8477e5c2-VolumeAccess">
/// Access level for a container on a volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeAccess {
    /// Read-only access matching a glob pattern.
    Read(String),
    /// Write access matching a glob pattern (implies read).
    Write(String),
}

</storycode:copy>
<a id="f-a58d8477e5c2-impl-VolumeAccess"></a>
### `impl-VolumeAccess`

<storycode:copy id="f-a58d8477e5c2-impl-VolumeAccess">
impl VolumeAccess {
    /// Check if this access rule permits reading the given path.
    pub fn permits_read(&self, path: &str) -> bool {
        match self {
            VolumeAccess::Read(pattern) | VolumeAccess::Write(pattern) => {
                glob_matches(path, pattern)
            }
        }
    }

    /// Check if this access rule permits writing the given path.
    pub fn permits_write(&self, path: &str) -> bool {
        match self {
            VolumeAccess::Write(pattern) => glob_matches(path, pattern),
            VolumeAccess::Read(_) => false,
        }
    }
}

</storycode:copy>
<a id="f-a58d8477e5c2-VolumeAccessRule"></a>
### `VolumeAccessRule`

An access rule binding a container to a volume with specific permissions.

<storycode:copy id="f-a58d8477e5c2-VolumeAccessRule">
/// An access rule binding a container to a volume with specific permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeAccessRule {
    /// The container this rule applies to.
    pub container: String,
    /// The access level granted.
    pub access: VolumeAccess,
}

</storycode:copy>
<a id="f-a58d8477e5c2-AccessScope"></a>
### `AccessScope`

How much of a volume one container may touch, in one direction.
`check_read`/`check_write` answer "may this container touch this path";
enforcement also needs the question the other way round — "what may this
container touch at all" — because the pipeline moves a whole volume in and
out of a container as a tar archive, and has to decide whether to hand
over everything, some of it, or nothing.

<storycode:copy id="f-a58d8477e5c2-AccessScope">
/// How much of a volume one container may touch, in one direction.
///
/// `check_read`/`check_write` answer "may this container touch this path";
/// enforcement also needs the question the other way round — "what may this
/// container touch at all" — because the pipeline moves a whole volume in and
/// out of a container as a tar archive, and has to decide whether to hand
/// over everything, some of it, or nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessScope {
    /// Nothing: the container has no rule granting this direction.
    None,
    /// Every path in the volume — either an explicit `**` rule, or the
    /// backward-compatible case of a volume that declares no rules at all.
    All,
    /// Only paths matching at least one of these glob patterns.
    Patterns(Vec<String>),
}

</storycode:copy>
<a id="f-a58d8477e5c2-impl-AccessScope"></a>
### `impl-AccessScope`

<storycode:copy id="f-a58d8477e5c2-impl-AccessScope">
impl AccessScope {
    /// Whether this scope permits touching `path`.
    pub fn permits(&self, path: &str) -> bool {
        match self {
            AccessScope::None => false,
            AccessScope::All => true,
            AccessScope::Patterns(patterns) => patterns.iter().any(|p| glob_matches(path, p)),
        }
    }

    /// Whether this scope permits nothing at all.
    pub fn is_none(&self) -> bool {
        matches!(self, AccessScope::None)
    }
}

</storycode:copy>
<a id="f-a58d8477e5c2-VolumeDeclaration"></a>
### `VolumeDeclaration`

A volume declaration parsed from `<hick:volume>`.

<storycode:copy id="f-a58d8477e5c2-VolumeDeclaration">
/// A volume declaration parsed from `<hick:volume>`.
#[derive(Debug, Clone)]
pub struct VolumeDeclaration {
    /// Volume name (unique within a document).
    pub name: String,
    /// What kind of volume this is.
    pub kind: VolumeKind,
    /// Explicit access rules from `<hick:allow>` children.
    pub access_rules: Vec<VolumeAccessRule>,
}

</storycode:copy>
<a id="f-a58d8477e5c2-impl-VolumeDeclaration"></a>
### `impl-VolumeDeclaration`

<storycode:copy id="f-a58d8477e5c2-impl-VolumeDeclaration">
impl VolumeDeclaration {
    /// Check if a container has read access to a path on this volume.
    ///
    /// If no access rules are defined, access is unrestricted (backward compat).
    pub fn check_read(&self, container: &str, path: &str) -> bool {
        if self.access_rules.is_empty() {
            return true;
        }
        self.access_rules
            .iter()
            .filter(|r| r.container == container)
            .any(|r| r.access.permits_read(path))
    }

    /// Check if a container has write access to a path on this volume.
    ///
    /// If no access rules are defined, access is unrestricted (backward compat).
    pub fn check_write(&self, container: &str, path: &str) -> bool {
        if self.access_rules.is_empty() {
            return true;
        }
        self.access_rules
            .iter()
            .filter(|r| r.container == container)
            .any(|r| r.access.permits_write(path))
    }

    /// What `container` may read from this volume.
    ///
    /// A volume with no `<hick:allow>` children is unrestricted, matching
    /// `check_read`: documents written before access rules existed keep
    /// working. A volume that does declare rules grants nothing to a
    /// container it does not name.
    pub fn read_scope(&self, container: &str) -> AccessScope {
        // Write implies read, so both rule kinds contribute patterns here.
        self.scope_for(container, |access| match access {
            VolumeAccess::Read(pattern) | VolumeAccess::Write(pattern) => Some(pattern.as_str()),
        })
    }

    /// What `container` may write to this volume.
    pub fn write_scope(&self, container: &str) -> AccessScope {
        self.scope_for(container, |access| match access {
            VolumeAccess::Write(pattern) => Some(pattern.as_str()),
            VolumeAccess::Read(_) => None,
        })
    }

    fn scope_for(
        &self,
        container: &str,
        pattern_of: impl Fn(&VolumeAccess) -> Option<&str>,
    ) -> AccessScope {
        if self.access_rules.is_empty() {
            return AccessScope::All;
        }
        let mut patterns = Vec::new();
        for rule in self
            .access_rules
            .iter()
            .filter(|r| r.container == container)
        {
            if let Some(pattern) = pattern_of(&rule.access) {
                if pattern == "**" {
                    return AccessScope::All;
                }
                if !patterns.iter().any(|p: &String| p == pattern) {
                    patterns.push(pattern.to_string());
                }
            }
        }
        if patterns.is_empty() {
            AccessScope::None
        } else {
            AccessScope::Patterns(patterns)
        }
    }

    /// Get the set of containers that have any write access on this volume.
    pub fn writers(&self) -> Vec<&str> {
        self.access_rules
            .iter()
            .filter(|r| matches!(r.access, VolumeAccess::Write(_)))
            .map(|r| r.container.as_str())
            .collect()
    }

    /// Get the set of containers that have read-only access on this volume.
    pub fn readers(&self) -> Vec<&str> {
        self.access_rules
            .iter()
            .filter(|r| matches!(r.access, VolumeAccess::Read(_)))
            .map(|r| r.container.as_str())
            .collect()
    }
}

</storycode:copy>
<a id="f-a58d8477e5c2-glob_matches"></a>
### `glob_matches`

Simple glob-style path matching.
Supports:
- `**` matches everything
- Trailing `*` matches any suffix
- `Dir/**` matches anything under Dir/
- Exact match otherwise

<storycode:copy id="f-a58d8477e5c2-glob_matches">
/// Simple glob-style path matching.
///
/// Supports:
/// - `**` matches everything
/// - Trailing `*` matches any suffix
/// - `Dir/**` matches anything under Dir/
/// - Exact match otherwise
fn glob_matches(path: &str, pattern: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("**") {
        return path.starts_with(prefix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return path.starts_with(prefix);
    }
    path == pattern
}

</storycode:copy>
<a id="f-a58d8477e5c2-tests"></a>
### `tests`

<storycode:copy id="f-a58d8477e5c2-tests">
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_double_star() {
        assert!(glob_matches("foo/bar/baz.txt", "**"));
        assert!(glob_matches("anything", "**"));
    }

    #[test]
    fn glob_matches_prefix_double_star() {
        assert!(glob_matches(
            "Controllers/HomeController.cs",
            "Controllers/**"
        ));
        assert!(!glob_matches("Models/User.cs", "Controllers/**"));
    }

    #[test]
    fn glob_matches_trailing_star() {
        assert!(glob_matches("src/main.rs", "src/*"));
        assert!(!glob_matches("tests/main.rs", "src/*"));
    }

    #[test]
    fn glob_matches_exact() {
        assert!(glob_matches("README.md", "README.md"));
        assert!(!glob_matches("README.txt", "README.md"));
    }

    #[test]
    fn volume_access_read_permits() {
        let access = VolumeAccess::Read("src/**".to_string());
        assert!(access.permits_read("src/main.rs"));
        assert!(!access.permits_write("src/main.rs"));
    }

    #[test]
    fn volume_access_write_implies_read() {
        let access = VolumeAccess::Write("output/**".to_string());
        assert!(access.permits_read("output/file.txt"));
        assert!(access.permits_write("output/file.txt"));
    }

    #[test]
    fn volume_declaration_no_rules_allows_all() {
        let vol = VolumeDeclaration {
            name: "shared".to_string(),
            kind: VolumeKind::Ephemeral,
            access_rules: vec![],
        };
        assert!(vol.check_read("any-container", "any/path"));
        assert!(vol.check_write("any-container", "any/path"));
    }

    #[test]
    fn volume_declaration_with_rules() {
        let vol = VolumeDeclaration {
            name: "project".to_string(),
            kind: VolumeKind::Output {
                path: "src/MyApi/".to_string(),
            },
            access_rules: vec![
                VolumeAccessRule {
                    container: "scaffolder".to_string(),
                    access: VolumeAccess::Write("**".to_string()),
                },
                VolumeAccessRule {
                    container: "linter".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "patcher".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "patcher".to_string(),
                    access: VolumeAccess::Write("Controllers/**".to_string()),
                },
            ],
        };

        // scaffolder can write anything
        assert!(vol.check_write("scaffolder", "Models/User.cs"));
        assert!(vol.check_read("scaffolder", "Models/User.cs"));

        // linter can only read
        assert!(vol.check_read("linter", "Models/User.cs"));
        assert!(!vol.check_write("linter", "Models/User.cs"));

        // patcher can read anything, write only Controllers/
        assert!(vol.check_read("patcher", "Models/User.cs"));
        assert!(!vol.check_write("patcher", "Models/User.cs"));
        assert!(vol.check_write("patcher", "Controllers/HomeController.cs"));

        // unknown container has no access
        assert!(!vol.check_read("unknown", "anything"));
    }

    /// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
    #[test]
    fn access_scopes_say_how_much_of_a_volume_a_container_may_touch() {
        let vol = VolumeDeclaration {
            name: "project".to_string(),
            kind: VolumeKind::Ephemeral,
            access_rules: vec![
                VolumeAccessRule {
                    container: "scaffolder".to_string(),
                    access: VolumeAccess::Write("**".to_string()),
                },
                VolumeAccessRule {
                    container: "linter".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "patcher".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "patcher".to_string(),
                    access: VolumeAccess::Write("Controllers/**".to_string()),
                },
                VolumeAccessRule {
                    container: "docs".to_string(),
                    access: VolumeAccess::Read("public/**".to_string()),
                },
            ],
        };

        // Write implies read, so a full writer reads everything.
        assert_eq!(vol.read_scope("scaffolder"), AccessScope::All);
        assert_eq!(vol.write_scope("scaffolder"), AccessScope::All);

        // A reader may not write anything at all.
        assert_eq!(vol.read_scope("linter"), AccessScope::All);
        assert_eq!(vol.write_scope("linter"), AccessScope::None);

        // A partial writer: everything readable, one subtree writable.
        assert_eq!(vol.read_scope("patcher"), AccessScope::All);
        let write = vol.write_scope("patcher");
        assert!(write.permits("Controllers/Home.cs"));
        assert!(!write.permits("Models/User.cs"));

        // A partial reader.
        let read = vol.read_scope("docs");
        assert!(read.permits("public/index.md"));
        assert!(!read.permits("private/keys.txt"));

        // A container the rules never name gets nothing in either direction.
        assert_eq!(vol.read_scope("stranger"), AccessScope::None);
        assert_eq!(vol.write_scope("stranger"), AccessScope::None);
        assert!(vol.read_scope("stranger").is_none());
    }

    /// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
    #[test]
    fn a_volume_with_no_rules_grants_everything_to_everyone() {
        // Backward compatibility: rules bind only once someone is named.
        let vol = VolumeDeclaration {
            name: "shared".to_string(),
            kind: VolumeKind::Ephemeral,
            access_rules: vec![],
        };
        assert_eq!(vol.read_scope("anyone"), AccessScope::All);
        assert_eq!(vol.write_scope("anyone"), AccessScope::All);
    }

    #[test]
    fn writers_and_readers() {
        let vol = VolumeDeclaration {
            name: "shared".to_string(),
            kind: VolumeKind::Ephemeral,
            access_rules: vec![
                VolumeAccessRule {
                    container: "writer1".to_string(),
                    access: VolumeAccess::Write("**".to_string()),
                },
                VolumeAccessRule {
                    container: "reader1".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "both".to_string(),
                    access: VolumeAccess::Write("output/**".to_string()),
                },
                VolumeAccessRule {
                    container: "both".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
            ],
        };

        let writers = vol.writers();
        assert!(writers.contains(&"writer1"));
        assert!(writers.contains(&"both"));
        assert!(!writers.contains(&"reader1"));

        let readers = vol.readers();
        assert!(readers.contains(&"reader1"));
        assert!(readers.contains(&"both"));
        assert!(!readers.contains(&"writer1"));
    }
}
</storycode:copy>

### Assemble `crates/hick-exec/src/volume.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/src/volume.rs"><storycode:paste select="#f-a58d8477e5c2-VolumeKind"/><storycode:paste select="#f-a58d8477e5c2-VolumeAccess"/><storycode:paste select="#f-a58d8477e5c2-impl-VolumeAccess"/><storycode:paste select="#f-a58d8477e5c2-VolumeAccessRule"/><storycode:paste select="#f-a58d8477e5c2-AccessScope"/><storycode:paste select="#f-a58d8477e5c2-impl-AccessScope"/><storycode:paste select="#f-a58d8477e5c2-VolumeDeclaration"/><storycode:paste select="#f-a58d8477e5c2-impl-VolumeDeclaration"/><storycode:paste select="#f-a58d8477e5c2-glob_matches"/><storycode:paste select="#f-a58d8477e5c2-tests"/></storycode:file>

<a id="output-8034529313d8"></a>
## `crates/hick-literate/tests/capability_enforcement_tests.rs`

Related contracts: [Declared Capabilities Are Enforced](contract.md).

<a id="f-8034529313d8-common"></a>
### `common`

What a document declares is what a container gets.
These protect docs/guarantees/execution/declared-capabilities-are-enforced.md.
They run against `LocalExecutor`, which is deliberately unsandboxed — and
that is the point for volumes: volume access is mediated by the *pipeline*
(it tars data into and out of a container), so it is enforced identically
on every backend. Network confinement is the executor's job, so the
network test here checks that the declaration reaches the executor, and
`hickory-executor-docker` checks what a backend does with it.

<storycode:copy id="f-8034529313d8-common">
//! What a document declares is what a container gets.
//!
//! These protect docs/guarantees/execution/declared-capabilities-are-enforced.md.
//!
//! They run against `LocalExecutor`, which is deliberately unsandboxed — and
//! that is the point for volumes: volume access is mediated by the *pipeline*
//! (it tars data into and out of a container), so it is enforced identically
//! on every backend. Network confinement is the executor's job, so the
//! network test here checks that the declaration reaches the executor, and
//! `hickory-executor-docker` checks what a backend does with it.

use std::sync::Arc;

use hickory_executor::{Executor, LocalExecutor};

// Contents are compared TRIMMED. What these guarantee is which bytes land in
// which file and which writes are refused; the line ending is the shell's, not
// the document's, and `echo` on cmd adds CRLF where `echo` on sh adds LF. The
// alternative in cmd -- `<nul set /p=` -- exits 1 and leaves a trailing space,
// so it breaks the `&&` chains AND the byte count. See tests/common/mod.rs.

mod common;
</storycode:copy>
<a id="f-8034529313d8-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-8034529313d8-use_declaration">
use common::{and, make_dir, require_absent, show, write};

</storycode:copy>
<a id="f-8034529313d8-hick_doc"></a>
### `hick_doc`

<storycode:copy id="f-8034529313d8-hick_doc">
fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

</storycode:copy>
<a id="f-8034529313d8-run"></a>
### `run`

<storycode:copy id="f-8034529313d8-run">
async fn run(src: &str) -> anyhow::Result<hick_literate::PipelineResult> {
    let executor = Arc::new(LocalExecutor::new().unwrap());
    hick_literate::run_pipeline_live(
        &[("test.md", src)],
        &hick_literate::PipelineConfig::default(),
        &[],
        None,
        executor,
    )
    .await
}

</storycode:copy>
<a id="f-8034529313d8-a_container_the_volume_does_not_name_gets_nothing"></a>
### `a_container_the_volume_does_not_name_gets_nothing`

<storycode:copy id="f-8034529313d8-a_container_the_volume_does_not_name_gets_nothing">
// ---------------------------------------------------------------------------
// Volumes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_container_the_volume_does_not_name_gets_nothing() {
    // The denied direction. `intruder` mounts a volume whose access rules
    // never mention it; the run stops rather than handing over the data.
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="intruder" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="intruder" mount="shared:/in">{reader}</hick:exec>"#,
        writer = write("secret", "out/file.txt"),
        reader = show("in/file.txt"),
    ));

    let Err(err) = run(&src).await else {
        panic!("the intruder was handed the volume");
    };
    let msg = err.to_string();
    assert!(
        msg.contains("intruder") && msg.contains("grants it no access"),
        "expected a refusal naming the container, got: {msg}"
    );
    // And it must say how to fix it — this surfaces on the CLI.
    assert!(msg.contains("hick:allow"), "no next step offered: {msg}");
}

</storycode:copy>
<a id="f-8034529313d8-a_named_container_gets_what_it_was_granted"></a>
### `a_named_container_gets_what_it_was_granted`

<storycode:copy id="f-8034529313d8-a_named_container_gets_what_it_was_granted">
#[tokio::test]
async fn a_named_container_gets_what_it_was_granted() {
    // The granted direction, same document shape.
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="reader" mount="shared:/in">{reader}</hick:exec>"#,
        writer = write("secret", "out/file.txt"),
        reader = show("in/file.txt"),
    ));

    let result = run(&src).await.expect("the reader was granted read access");
    assert_eq!(
        result.files.get("result/file.txt").unwrap().trim(),
        "secret"
    );
}

</storycode:copy>
<a id="f-8034529313d8-a_partial_read_grant_hands_over_only_that_part"></a>
### `a_partial_read_grant_hands_over_only_that_part`

<storycode:copy id="f-8034529313d8-a_partial_read_grant_hands_over_only_that_part">
#[tokio::test]
async fn a_partial_read_grant_hands_over_only_that_part() {
    // `read="public/**"` must mean the container never sees the rest, not
    // that it is trusted to ignore it. The reader asserts the absence
    // itself, so a leak fails the exec.
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="public/**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="reader" mount="shared:/in">{reader}</hick:exec>"#,
        writer = and(&[
            make_dir("out/public"),
            write("open", "out/public/ok.txt"),
            write("closed", "out/private.txt"),
        ]),
        reader = and(&[show("in/public/ok.txt"), require_absent("in/private.txt")]),
    ));

    let result = run(&src)
        .await
        .expect("the reader saw a file outside its read grant");
    // The volume itself still has both: the grant narrowed the copy handed
    // to the reader, it did not delete anything.
    assert_eq!(
        result.files.get("result/private.txt").unwrap().trim(),
        "closed"
    );
    assert_eq!(
        result.files.get("result/public/ok.txt").unwrap().trim(),
        "open"
    );
}

</storycode:copy>
<a id="f-8034529313d8-a_read_only_container_cannot_change_the_volume"></a>
### `a_read_only_container_cannot_change_the_volume`

<storycode:copy id="f-8034529313d8-a_read_only_container_cannot_change_the_volume">
#[tokio::test]
async fn a_read_only_container_cannot_change_the_volume() {
    // Write access is not "whoever mounted it last wins".
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="reader" mount="shared:/in">{reader}</hick:exec>"#,
        writer = write("original", "out/file.txt"),
        reader = write("tampered", "in/file.txt"),
    ));

    let result = run(&src).await.expect("pipeline should complete");
    assert_eq!(
        result.files.get("result/file.txt").unwrap().trim(),
        "original",
        "a read-only container's writes escaped its container"
    );
}

</storycode:copy>
<a id="f-8034529313d8-a_partial_writer_changes_only_the_paths_it_was_granted"></a>
### `a_partial_writer_changes_only_the_paths_it_was_granted`

<storycode:copy id="f-8034529313d8-a_partial_writer_changes_only_the_paths_it_was_granted">
#[tokio::test]
async fn a_partial_writer_changes_only_the_paths_it_was_granted() {
    // The case that used to be enforced as all-or-nothing: `patcher` may
    // rewrite Controllers/ and nothing else. Its other write must not land,
    // and its permitted write must.
    let src = hick_doc(&format!(
        r#"<hick:container name="scaffolder" image="alpine" />
<hick:container name="patcher" image="alpine" />
<hick:volume name="project" output="result">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="patcher" read="**" />
  <hick:allow container="patcher" write="Controllers/**" />
</hick:volume>
<hick:exec container="scaffolder" mount="project:/p">{scaffolder}</hick:exec>
<hick:exec container="patcher" mount="project:/p">{patcher}</hick:exec>"#,
        scaffolder = and(&[
            make_dir("p/Controllers"),
            write("base", "p/Models.cs"),
            write("scaffolded", "p/Controllers/Home.cs"),
        ]),
        patcher = and(&[
            write("patched", "p/Controllers/Home.cs"),
            write("hacked", "p/Models.cs"),
        ]),
    ));

    let result = run(&src).await.expect("pipeline should complete");
    assert_eq!(
        result
            .files
            .get("result/Controllers/Home.cs")
            .unwrap()
            .trim(),
        "patched",
        "the granted write did not land"
    );
    assert_eq!(
        result.files.get("result/Models.cs").unwrap().trim(),
        "base",
        "the ungranted write landed anyway"
    );
}

</storycode:copy>
<a id="f-8034529313d8-a_volume_with_no_allow_children_stays_unrestricted"></a>
### `a_volume_with_no_allow_children_stays_unrestricted`

<storycode:copy id="f-8034529313d8-a_volume_with_no_allow_children_stays_unrestricted">
#[tokio::test]
async fn a_volume_with_no_allow_children_stays_unrestricted() {
    // Documents written before access rules existed must keep working:
    // rules apply to everyone only once someone is named.
    let src = hick_doc(&format!(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:volume name="shared" output="result" />
<hick:exec container="a" mount="shared:/out">{a}</hick:exec>
<hick:exec container="b" mount="shared:/in">{b}</hick:exec>"#,
        a = write("from-a", "out/file.txt"),
        b = and(&[show("in/file.txt"), write("from-b", "in/file.txt")]),
    ));

    let result = run(&src).await.expect("unrestricted volume should work");
    assert_eq!(
        result.files.get("result/file.txt").unwrap().trim(),
        "from-b"
    );
}

</storycode:copy>
<a id="f-8034529313d8-the_declared_network_capability_reaches_the_executor"></a>
### `the_declared_network_capability_reaches_the_executor`

<storycode:copy id="f-8034529313d8-the_declared_network_capability_reaches_the_executor">
// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_declared_network_capability_reaches_the_executor() {
    // The plumbing this whole change is about: a container's declaration is
    // handed to the executor before it starts, so a backend that can confine
    // it has something to confine it with. `LocalExecutor` records and does
    // not impose — see the crate docs there.
    let src = hick_doc(
        r#"<hick:container name="online" image="alpine">
  <hick:allow network="github.com:443" />
</hick:container>
<hick:container name="offline" image="alpine" />
<hick:exec container="online">echo hi</hick:exec>
<hick:exec container="offline">echo hi</hick:exec>"#,
    );

    let executor = Arc::new(LocalExecutor::new().unwrap());
    let recorder: Arc<dyn Executor> = executor.clone();
    hick_literate::run_pipeline_live(
        &[("test.md", &src)],
        &hick_literate::PipelineConfig::default(),
        &[],
        None,
        recorder,
    )
    .await
    .unwrap();

    assert!(
        executor
            .declared_capabilities("online")
            .expect("no capabilities declared for 'online'")
            .allows_network(),
        "the document's network grant never reached the executor"
    );
    assert!(
        !executor
            .declared_capabilities("offline")
            .expect("no capabilities declared for 'offline'")
            .allows_network(),
        "a container that declared nothing was given network access"
    );
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/capability_enforcement_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/capability_enforcement_tests.rs"><storycode:paste select="#f-8034529313d8-common"/><storycode:paste select="#f-8034529313d8-use_declaration"/><storycode:paste select="#f-8034529313d8-hick_doc"/><storycode:paste select="#f-8034529313d8-run"/><storycode:paste select="#f-8034529313d8-a_container_the_volume_does_not_name_gets_nothing"/><storycode:paste select="#f-8034529313d8-a_named_container_gets_what_it_was_granted"/><storycode:paste select="#f-8034529313d8-a_partial_read_grant_hands_over_only_that_part"/><storycode:paste select="#f-8034529313d8-a_read_only_container_cannot_change_the_volume"/><storycode:paste select="#f-8034529313d8-a_partial_writer_changes_only_the_paths_it_was_granted"/><storycode:paste select="#f-8034529313d8-a_volume_with_no_allow_children_stays_unrestricted"/><storycode:paste select="#f-8034529313d8-the_declared_network_capability_reaches_the_executor"/></storycode:file>

<a id="output-77c16f1d2966"></a>
## `crates/hick-token/src/lib.rs`

Related contracts: [Declared Capabilities Are Enforced](contract.md).

<a id="f-77c16f1d2966-NetworkRule"></a>
### `NetworkRule`

Macaroon-based capability token system for hick containers.
Each container receives a macaroon token that encodes its capabilities
(network access, file access, secrets). Tokens can be attenuated (add
restrictions) but never escalated.

<storycode:copy id="f-77c16f1d2966-NetworkRule">
//! Macaroon-based capability token system for hick containers.
//!
//! Each container receives a macaroon token that encodes its capabilities
//! (network access, file access, secrets). Tokens can be attenuated (add
//! restrictions) but never escalated.

use std::fmt;

use macaroon::{ByteString, Macaroon, MacaroonKey, Verifier};

// ---------------------------------------------------------------------------
// Capability types
// ---------------------------------------------------------------------------

/// Network access rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkRule {
    /// Allow traffic to a specific host:port (port can be `"*"` for wildcard).
    Allow { host: String, port: String },
    /// Deny all network traffic.
    DenyAll,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-NetworkRule"></a>
### `impl-NetworkRule`

<storycode:copy id="f-77c16f1d2966-impl-NetworkRule">
impl NetworkRule {
    pub fn allow(host: &str, port: &str) -> Self {
        NetworkRule::Allow {
            host: host.to_string(),
            port: port.to_string(),
        }
    }

    pub fn deny_all() -> Self {
        NetworkRule::DenyAll
    }

    /// Convert to a macaroon caveat string.
    pub fn to_caveat(&self) -> String {
        match self {
            NetworkRule::Allow { host, port } => format!("network = allow {host}:{port}"),
            NetworkRule::DenyAll => "network = deny *".to_string(),
        }
    }

    /// Parse from a macaroon caveat string.
    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let caveat = caveat.strip_prefix("network = ")?;
        if caveat == "deny *" {
            return Some(NetworkRule::DenyAll);
        }
        let rest = caveat.strip_prefix("allow ")?;
        let (host, port) = rest.rsplit_once(':')?;
        Some(NetworkRule::Allow {
            host: host.to_string(),
            port: port.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-FileRule"></a>
### `FileRule`

<storycode:copy id="f-77c16f1d2966-FileRule">
/// File access rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRule {
    Read(String),
    Write(String),
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-FileRule"></a>
### `impl-FileRule`

<storycode:copy id="f-77c16f1d2966-impl-FileRule">
impl FileRule {
    pub fn to_caveat(&self) -> String {
        match self {
            FileRule::Read(path) => format!("file-read = {path}"),
            FileRule::Write(path) => format!("file-write = {path}"),
        }
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        if let Some(path) = caveat.strip_prefix("file-read = ") {
            return Some(FileRule::Read(path.to_string()));
        }
        if let Some(path) = caveat.strip_prefix("file-write = ") {
            return Some(FileRule::Write(path.to_string()));
        }
        None
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-VolumeAccess"></a>
### `VolumeAccess`

Volume access level for capability tokens.

<storycode:copy id="f-77c16f1d2966-VolumeAccess">
/// Volume access level for capability tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeAccess {
    /// Read-only access matching a glob pattern.
    Read(String),
    /// Write access matching a glob pattern (implies read).
    Write(String),
}

</storycode:copy>
<a id="f-77c16f1d2966-VolumeRule"></a>
### `VolumeRule`

Volume access rule for a container capability token.

<storycode:copy id="f-77c16f1d2966-VolumeRule">
/// Volume access rule for a container capability token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeRule {
    /// Volume name.
    pub volume: String,
    /// Access level.
    pub access: VolumeAccess,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-VolumeRule"></a>
### `impl-VolumeRule`

<storycode:copy id="f-77c16f1d2966-impl-VolumeRule">
impl VolumeRule {
    pub fn read(volume: &str, pattern: &str) -> Self {
        VolumeRule {
            volume: volume.to_string(),
            access: VolumeAccess::Read(pattern.to_string()),
        }
    }

    pub fn write(volume: &str, pattern: &str) -> Self {
        VolumeRule {
            volume: volume.to_string(),
            access: VolumeAccess::Write(pattern.to_string()),
        }
    }

    /// Convert to a macaroon caveat string.
    pub fn to_caveat(&self) -> String {
        match &self.access {
            VolumeAccess::Read(pattern) => {
                format!("volume-read = {}:{}", self.volume, pattern)
            }
            VolumeAccess::Write(pattern) => {
                format!("volume-write = {}:{}", self.volume, pattern)
            }
        }
    }

    /// Parse from a macaroon caveat string.
    pub fn from_caveat(caveat: &str) -> Option<Self> {
        if let Some(rest) = caveat.strip_prefix("volume-read = ") {
            let (volume, pattern) = rest.split_once(':')?;
            return Some(VolumeRule {
                volume: volume.to_string(),
                access: VolumeAccess::Read(pattern.to_string()),
            });
        }
        if let Some(rest) = caveat.strip_prefix("volume-write = ") {
            let (volume, pattern) = rest.split_once(':')?;
            return Some(VolumeRule {
                volume: volume.to_string(),
                access: VolumeAccess::Write(pattern.to_string()),
            });
        }
        None
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-SecretRule"></a>
### `SecretRule`

<storycode:copy id="f-77c16f1d2966-SecretRule">
/// Secret injection rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRule {
    pub env_var: String,
    pub secret_name: String,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-SecretRule"></a>
### `impl-SecretRule`

<storycode:copy id="f-77c16f1d2966-impl-SecretRule">
impl SecretRule {
    pub fn to_caveat(&self) -> String {
        format!("secret = {}:{}", self.env_var, self.secret_name)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("secret = ")?;
        let (env_var, secret_name) = rest.split_once(':')?;
        Some(SecretRule {
            env_var: env_var.to_string(),
            secret_name: secret_name.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-DataAccessRule"></a>
### `DataAccessRule`

Extended rule types
Data access rule: controls which data classification patterns are accessible.

<storycode:copy id="f-77c16f1d2966-DataAccessRule">
// ---------------------------------------------------------------------------
// Extended rule types
// ---------------------------------------------------------------------------

/// Data access rule: controls which data classification patterns are accessible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataAccessRule {
    pub pattern: String,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-DataAccessRule"></a>
### `impl-DataAccessRule`

<storycode:copy id="f-77c16f1d2966-impl-DataAccessRule">
impl DataAccessRule {
    pub fn to_caveat(&self) -> String {
        format!("data-access = {}", self.pattern)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let pattern = caveat.strip_prefix("data-access = ")?;
        Some(Self {
            pattern: pattern.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-ApiSinkRule"></a>
### `ApiSinkRule`

API sink rule: controls which sink types are accessible.

<storycode:copy id="f-77c16f1d2966-ApiSinkRule">
/// API sink rule: controls which sink types are accessible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSinkRule {
    pub sink_type: String,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-ApiSinkRule"></a>
### `impl-ApiSinkRule`

<storycode:copy id="f-77c16f1d2966-impl-ApiSinkRule">
impl ApiSinkRule {
    pub fn to_caveat(&self) -> String {
        format!("api-sink = {}", self.sink_type)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let sink_type = caveat.strip_prefix("api-sink = ")?;
        Some(Self {
            sink_type: sink_type.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-PurposeRule"></a>
### `PurposeRule`

Purpose rule: constrains data usage to a specific context and purpose.

<storycode:copy id="f-77c16f1d2966-PurposeRule">
/// Purpose rule: constrains data usage to a specific context and purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurposeRule {
    pub context: String,
    pub purpose: String,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-PurposeRule"></a>
### `impl-PurposeRule`

<storycode:copy id="f-77c16f1d2966-impl-PurposeRule">
impl PurposeRule {
    pub fn to_caveat(&self) -> String {
        format!("purpose = {}:{}", self.context, self.purpose)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("purpose = ")?;
        let (context, purpose) = rest.split_once(':')?;
        Some(Self {
            context: context.to_string(),
            purpose: purpose.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-ExpiresRule"></a>
### `ExpiresRule`

Expiration rule: token is invalid after this timestamp.

<storycode:copy id="f-77c16f1d2966-ExpiresRule">
/// Expiration rule: token is invalid after this timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiresRule {
    pub expires_at: String,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-ExpiresRule"></a>
### `impl-ExpiresRule`

<storycode:copy id="f-77c16f1d2966-impl-ExpiresRule">
impl ExpiresRule {
    pub fn to_caveat(&self) -> String {
        format!("expires-at = {}", self.expires_at)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let ts = caveat.strip_prefix("expires-at = ")?;
        Some(Self {
            expires_at: ts.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-MaxCallsRule"></a>
### `MaxCallsRule`

Max calls rule: limits the number of calls to a specific sink.

<storycode:copy id="f-77c16f1d2966-MaxCallsRule">
/// Max calls rule: limits the number of calls to a specific sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaxCallsRule {
    pub sink_type: String,
    pub max_calls: u64,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-MaxCallsRule"></a>
### `impl-MaxCallsRule`

<storycode:copy id="f-77c16f1d2966-impl-MaxCallsRule">
impl MaxCallsRule {
    pub fn to_caveat(&self) -> String {
        format!("max-calls = {}:{}", self.sink_type, self.max_calls)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("max-calls = ")?;
        let (sink_type, n) = rest.rsplit_once(':')?;
        let max_calls = n.parse().ok()?;
        Some(Self {
            sink_type: sink_type.to_string(),
            max_calls,
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-RecipientRule"></a>
### `RecipientRule`

Recipient rule: constrains the recipient for a sink.

<storycode:copy id="f-77c16f1d2966-RecipientRule">
/// Recipient rule: constrains the recipient for a sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipientRule {
    pub sink_type: String,
    pub recipient: String,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-RecipientRule"></a>
### `impl-RecipientRule`

<storycode:copy id="f-77c16f1d2966-impl-RecipientRule">
impl RecipientRule {
    pub fn to_caveat(&self) -> String {
        format!("recipient = {}:{}", self.sink_type, self.recipient)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("recipient = ")?;
        let (sink_type, recipient) = rest.split_once(':')?;
        Some(Self {
            sink_type: sink_type.to_string(),
            recipient: recipient.to_string(),
        })
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-ContainerCapabilities"></a>
### `ContainerCapabilities`

ContainerCapabilities
The complete set of capabilities for a container.

<storycode:copy id="f-77c16f1d2966-ContainerCapabilities">
// ---------------------------------------------------------------------------
// ContainerCapabilities
// ---------------------------------------------------------------------------

/// The complete set of capabilities for a container.
#[derive(Debug, Clone, Default)]
pub struct ContainerCapabilities {
    pub network_rules: Vec<NetworkRule>,
    pub file_rules: Vec<FileRule>,
    pub secret_rules: Vec<SecretRule>,
    pub volume_rules: Vec<VolumeRule>,
    pub data_access_rules: Vec<DataAccessRule>,
    pub api_sink_rules: Vec<ApiSinkRule>,
    pub purpose_rules: Vec<PurposeRule>,
    pub expires_rules: Vec<ExpiresRule>,
    pub max_calls_rules: Vec<MaxCallsRule>,
    pub recipient_rules: Vec<RecipientRule>,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-ContainerCapabilities"></a>
### `impl-ContainerCapabilities`

<storycode:copy id="f-77c16f1d2966-impl-ContainerCapabilities">
impl ContainerCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_network(mut self, host: &str, port: &str) -> Self {
        self.network_rules.push(NetworkRule::allow(host, port));
        self
    }

    pub fn deny_all_network(mut self) -> Self {
        self.network_rules.push(NetworkRule::deny_all());
        self
    }

    pub fn allow_file_read(mut self, path: &str) -> Self {
        self.file_rules.push(FileRule::Read(path.to_string()));
        self
    }

    pub fn allow_file_write(mut self, path: &str) -> Self {
        self.file_rules.push(FileRule::Write(path.to_string()));
        self
    }

    pub fn with_secret(mut self, env_var: &str, secret_name: &str) -> Self {
        self.secret_rules.push(SecretRule {
            env_var: env_var.to_string(),
            secret_name: secret_name.to_string(),
        });
        self
    }

    pub fn allow_volume_read(mut self, volume: &str, pattern: &str) -> Self {
        self.volume_rules.push(VolumeRule::read(volume, pattern));
        self
    }

    pub fn allow_volume_write(mut self, volume: &str, pattern: &str) -> Self {
        self.volume_rules.push(VolumeRule::write(volume, pattern));
        self
    }

    pub fn allow_data_access(mut self, pattern: &str) -> Self {
        self.data_access_rules.push(DataAccessRule {
            pattern: pattern.to_string(),
        });
        self
    }

    pub fn allow_api_sink(mut self, sink_type: &str) -> Self {
        self.api_sink_rules.push(ApiSinkRule {
            sink_type: sink_type.to_string(),
        });
        self
    }

    pub fn with_purpose(mut self, context: &str, purpose: &str) -> Self {
        self.purpose_rules.push(PurposeRule {
            context: context.to_string(),
            purpose: purpose.to_string(),
        });
        self
    }

    pub fn with_expires(mut self, expires_at: &str) -> Self {
        self.expires_rules.push(ExpiresRule {
            expires_at: expires_at.to_string(),
        });
        self
    }

    pub fn with_max_calls(mut self, sink_type: &str, max_calls: u64) -> Self {
        self.max_calls_rules.push(MaxCallsRule {
            sink_type: sink_type.to_string(),
            max_calls,
        });
        self
    }

    pub fn with_recipient(mut self, sink_type: &str, recipient: &str) -> Self {
        self.recipient_rules.push(RecipientRule {
            sink_type: sink_type.to_string(),
            recipient: recipient.to_string(),
        });
        self
    }

    /// Whether these capabilities ask for **any** outbound network access:
    /// true exactly when at least one `Allow` rule is present.
    ///
    /// This is the question a sandbox can actually answer. A container is
    /// confined by a switch — the docker executor's `--network` — so the
    /// choice is connectivity or none, and only an explicit
    /// `<hick:allow network="host:port">` turns it on. `DenyAll` alone, or
    /// no rules at all, means no network.
    ///
    /// **This is deliberately stricter than [`Self::check_network`], which
    /// answers a different question.** `check_network` is the policy
    /// predicate: with no `DenyAll` present it treats the container as
    /// unconstrained, because a document that says nothing about the network
    /// has stated no policy. At the enforcement boundary silence must mean
    /// *no*, or every legacy document would be handed connectivity the
    /// moment capabilities started driving the sandbox.
    ///
    /// Note the coarseness in the other direction: the host and port in an
    /// `Allow` rule are **not** enforced by a switch, so a container granted
    /// `github.com:443` can reach whatever the chosen network reaches.
    /// Narrowing that needs an egress proxy.
    pub fn allows_network(&self) -> bool {
        self.network_rules
            .iter()
            .any(|r| matches!(r, NetworkRule::Allow { .. }))
    }

    /// Intersect two capability sets, producing the most restrictive combination.
    ///
    /// For attenuation: the result only contains capabilities present in both sets.
    /// - Network: keep only Allow rules present in both; preserve DenyAll from either
    /// - File/Volume: keep only rules present in both (intersection of allowed paths)
    /// - Secret: keep only secrets present in both
    pub fn intersect(&self, restrictor: &ContainerCapabilities) -> ContainerCapabilities {
        let network_rules = {
            let mut rules = Vec::new();
            // Preserve DenyAll from either side
            let has_deny_self = self
                .network_rules
                .iter()
                .any(|r| matches!(r, NetworkRule::DenyAll));
            let has_deny_other = restrictor
                .network_rules
                .iter()
                .any(|r| matches!(r, NetworkRule::DenyAll));
            if has_deny_self || has_deny_other {
                rules.push(NetworkRule::DenyAll);
            }
            // Keep only Allow rules present in both
            for rule in &self.network_rules {
                if let NetworkRule::Allow { .. } = rule
                    && restrictor.network_rules.contains(rule)
                    && !rules.contains(rule)
                {
                    rules.push(rule.clone());
                }
            }
            rules
        };

        let file_rules = self
            .file_rules
            .iter()
            .filter(|r| restrictor.file_rules.contains(r))
            .cloned()
            .collect();

        let secret_rules = self
            .secret_rules
            .iter()
            .filter(|r| restrictor.secret_rules.contains(r))
            .cloned()
            .collect();

        let volume_rules = self
            .volume_rules
            .iter()
            .filter(|r| restrictor.volume_rules.contains(r))
            .cloned()
            .collect();

        let data_access_rules = self
            .data_access_rules
            .iter()
            .filter(|r| restrictor.data_access_rules.contains(r))
            .cloned()
            .collect();

        let api_sink_rules = self
            .api_sink_rules
            .iter()
            .filter(|r| restrictor.api_sink_rules.contains(r))
            .cloned()
            .collect();

        let purpose_rules = self
            .purpose_rules
            .iter()
            .filter(|r| restrictor.purpose_rules.contains(r))
            .cloned()
            .collect();

        // Expires: keep the most restrictive (earliest) from both
        let mut expires_rules = self.expires_rules.clone();
        expires_rules.extend(restrictor.expires_rules.clone());

        // MaxCalls: keep all from both (each is additive restriction)
        let mut max_calls_rules = self.max_calls_rules.clone();
        for r in &restrictor.max_calls_rules {
            if !max_calls_rules.contains(r) {
                max_calls_rules.push(r.clone());
            }
        }

        // Recipient: keep all from both
        let mut recipient_rules = self.recipient_rules.clone();
        for r in &restrictor.recipient_rules {
            if !recipient_rules.contains(r) {
                recipient_rules.push(r.clone());
            }
        }

        ContainerCapabilities {
            network_rules,
            file_rules,
            secret_rules,
            volume_rules,
            data_access_rules,
            api_sink_rules,
            purpose_rules,
            expires_rules,
            max_calls_rules,
            recipient_rules,
        }
    }

    /// Merge additional caveat-derived rules onto these capabilities (union of rules).
    ///
    /// This is used internally by `CapabilityToken::attenuate()` to track the
    /// union of all caveats in the in-memory capabilities struct. Each caveat is
    /// an *additional restriction* — the macaroon verifier checks all caveats pass.
    /// The in-memory capabilities struct is a convenience cache; the macaroon is
    /// the source of truth for enforcement.
    ///
    /// **Do not use this for capability widening** — it must only be called
    /// alongside macaroon attenuation, which adds corresponding caveats.
    pub fn merge_caveats(&self, additional: &ContainerCapabilities) -> ContainerCapabilities {
        let mut caps = self.clone();

        for r in &additional.network_rules {
            if !caps.network_rules.contains(r) {
                caps.network_rules.push(r.clone());
            }
        }
        for r in &additional.file_rules {
            if !caps.file_rules.contains(r) {
                caps.file_rules.push(r.clone());
            }
        }
        for r in &additional.secret_rules {
            if !caps.secret_rules.contains(r) {
                caps.secret_rules.push(r.clone());
            }
        }
        for r in &additional.volume_rules {
            if !caps.volume_rules.contains(r) {
                caps.volume_rules.push(r.clone());
            }
        }
        for r in &additional.data_access_rules {
            if !caps.data_access_rules.contains(r) {
                caps.data_access_rules.push(r.clone());
            }
        }
        for r in &additional.api_sink_rules {
            if !caps.api_sink_rules.contains(r) {
                caps.api_sink_rules.push(r.clone());
            }
        }
        for r in &additional.purpose_rules {
            if !caps.purpose_rules.contains(r) {
                caps.purpose_rules.push(r.clone());
            }
        }
        for r in &additional.expires_rules {
            if !caps.expires_rules.contains(r) {
                caps.expires_rules.push(r.clone());
            }
        }
        for r in &additional.max_calls_rules {
            if !caps.max_calls_rules.contains(r) {
                caps.max_calls_rules.push(r.clone());
            }
        }
        for r in &additional.recipient_rules {
            if !caps.recipient_rules.contains(r) {
                caps.recipient_rules.push(r.clone());
            }
        }

        caps
    }

    /// Build capabilities from a list of caveat strings.
    pub fn from_caveats(caveats: &[String]) -> Self {
        let mut caps = Self::new();
        for caveat in caveats {
            if let Some(r) = NetworkRule::from_caveat(caveat) {
                caps.network_rules.push(r);
            } else if let Some(r) = FileRule::from_caveat(caveat) {
                caps.file_rules.push(r);
            } else if let Some(r) = SecretRule::from_caveat(caveat) {
                caps.secret_rules.push(r);
            } else if let Some(r) = VolumeRule::from_caveat(caveat) {
                caps.volume_rules.push(r);
            } else if let Some(r) = DataAccessRule::from_caveat(caveat) {
                caps.data_access_rules.push(r);
            } else if let Some(r) = ApiSinkRule::from_caveat(caveat) {
                caps.api_sink_rules.push(r);
            } else if let Some(r) = PurposeRule::from_caveat(caveat) {
                caps.purpose_rules.push(r);
            } else if let Some(r) = ExpiresRule::from_caveat(caveat) {
                caps.expires_rules.push(r);
            } else if let Some(r) = MaxCallsRule::from_caveat(caveat) {
                caps.max_calls_rules.push(r);
            } else if let Some(r) = RecipientRule::from_caveat(caveat) {
                caps.recipient_rules.push(r);
            }
        }
        caps
    }

    /// Convert all capabilities to caveat strings.
    pub fn to_caveats(&self) -> Vec<String> {
        let mut caveats = Vec::new();
        for rule in &self.network_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.file_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.secret_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.volume_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.data_access_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.api_sink_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.purpose_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.expires_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.max_calls_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.recipient_rules {
            caveats.push(rule.to_caveat());
        }
        caveats
    }

    /// Check if network access to `host:port` is allowed.
    ///
    /// Logic: if any `DenyAll` rule exists, only explicit `Allow` rules permit
    /// traffic. If no `DenyAll`, all traffic is allowed by default.
    pub fn check_network(&self, host: &str, port: u16) -> bool {
        let has_deny_all = self
            .network_rules
            .iter()
            .any(|r| matches!(r, NetworkRule::DenyAll));
        if !has_deny_all {
            return true;
        }

        let port_str = port.to_string();
        self.network_rules.iter().any(|r| match r {
            NetworkRule::Allow { host: h, port: p } => {
                let host_match = h == host || h == "*";
                let port_match = p == "*" || p == &port_str;
                host_match && port_match
            }
            NetworkRule::DenyAll => false,
        })
    }

    /// Check if file read is allowed at the given path.
    /// Denies access when no file rules are defined (closed-by-default).
    pub fn check_file_read(&self, path: &str) -> bool {
        if self.file_rules.is_empty() {
            return false;
        }
        self.file_rules.iter().any(|r| match r {
            FileRule::Read(pattern) => path_matches(path, pattern),
            FileRule::Write(pattern) => path_matches(path, pattern), // write implies read
            #[allow(unreachable_patterns)]
            _ => false,
        })
    }

    /// Check if file write is allowed at the given path.
    /// Denies access when no file rules are defined (closed-by-default).
    pub fn check_file_write(&self, path: &str) -> bool {
        if self.file_rules.is_empty() {
            return false;
        }
        self.file_rules.iter().any(|r| match r {
            FileRule::Write(pattern) => path_matches(path, pattern),
            _ => false,
        })
    }

    /// Check if volume read access is allowed.
    /// Denies access when no volume rules are defined for the given volume (closed-by-default).
    pub fn check_volume_read(&self, volume: &str, path: &str) -> bool {
        let relevant: Vec<_> = self
            .volume_rules
            .iter()
            .filter(|r| r.volume == volume)
            .collect();
        if relevant.is_empty() {
            return false;
        }
        relevant.iter().any(|r| match &r.access {
            VolumeAccess::Read(pattern) | VolumeAccess::Write(pattern) => {
                path_matches(path, pattern)
            }
        })
    }

    /// Check if volume write access is allowed.
    /// Denies access when no volume rules are defined for the given volume (closed-by-default).
    pub fn check_volume_write(&self, volume: &str, path: &str) -> bool {
        let relevant: Vec<_> = self
            .volume_rules
            .iter()
            .filter(|r| r.volume == volume)
            .collect();
        if relevant.is_empty() {
            return false;
        }
        relevant.iter().any(|r| match &r.access {
            VolumeAccess::Write(pattern) => path_matches(path, pattern),
            _ => false,
        })
    }

    /// Check if data access to the given classification pattern is allowed.
    /// Denies access when no data access rules are defined (closed-by-default).
    pub fn check_data_access(&self, pattern: &str) -> bool {
        if self.data_access_rules.is_empty() {
            return false;
        }
        self.data_access_rules.iter().any(|r| r.pattern == pattern)
    }

    /// Check if the given API sink type is allowed.
    /// If no API sink rules are defined, no sinks are allowed.
    pub fn check_api_sink(&self, sink_type: &str) -> bool {
        if self.api_sink_rules.is_empty() {
            return false;
        }
        self.api_sink_rules.iter().any(|r| r.sink_type == sink_type)
    }

    /// Check if the token has expired given the current time (ISO 8601).
    pub fn check_expires(&self, now: &str) -> bool {
        if self.expires_rules.is_empty() {
            return true;
        }
        let now_dt = match chrono::DateTime::parse_from_rfc3339(now) {
            Ok(dt) => dt,
            Err(_) => return false,
        };
        self.expires_rules.iter().all(|r| {
            match chrono::DateTime::parse_from_rfc3339(&r.expires_at) {
                Ok(exp) => now_dt < exp,
                Err(_) => false,
            }
        })
    }

    /// Check if the max calls limit for a sink has been reached.
    pub fn check_max_calls(&self, sink_type: &str, current_count: u64) -> bool {
        for rule in &self.max_calls_rules {
            if rule.sink_type == sink_type && current_count >= rule.max_calls {
                return false;
            }
        }
        true
    }

    /// Check if the recipient matches any recipient constraints for the sink.
    pub fn check_recipient(&self, sink_type: &str, recipient: &str) -> bool {
        let relevant: Vec<_> = self
            .recipient_rules
            .iter()
            .filter(|r| r.sink_type == sink_type)
            .collect();
        if relevant.is_empty() {
            return true;
        }
        relevant.iter().any(|r| r.recipient == recipient)
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-path_matches"></a>
### `path_matches`

Simple glob-style path matching.
Supports:
- `**` matches everything
- `prefix/**` matches anything under `prefix/`
- Trailing `*` matches any suffix after prefix
- Exact match otherwise

<storycode:copy id="f-77c16f1d2966-path_matches">
/// Simple glob-style path matching.
///
/// Supports:
/// - `**` matches everything
/// - `prefix/**` matches anything under `prefix/`
/// - Trailing `*` matches any suffix after prefix
/// - Exact match otherwise
fn path_matches(path: &str, pattern: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("**") {
        return path.starts_with(prefix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return path.starts_with(prefix);
    }
    path == pattern
}

</storycode:copy>
<a id="f-77c16f1d2966-TokenAuthority"></a>
### `TokenAuthority`

TokenAuthority
The authority that mints and verifies capability tokens.

<storycode:copy id="f-77c16f1d2966-TokenAuthority">
// ---------------------------------------------------------------------------
// TokenAuthority
// ---------------------------------------------------------------------------

/// The authority that mints and verifies capability tokens.
pub struct TokenAuthority {
    root_key: MacaroonKey,
}

</storycode:copy>
<a id="f-77c16f1d2966-TokenError"></a>
### `TokenError`

<storycode:copy id="f-77c16f1d2966-TokenError">
/// Errors from token operations.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("macaroon error: {0}")]
    Macaroon(String),

    #[error("verification failed: {0}")]
    Verification(String),

    #[error("invalid caveat: {0}")]
    InvalidCaveat(String),
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-TokenAuthority"></a>
### `impl-TokenAuthority`

<storycode:copy id="f-77c16f1d2966-impl-TokenAuthority">
impl TokenAuthority {
    /// Create a new authority with the given root key bytes.
    pub fn new(key_bytes: &[u8]) -> Self {
        let root_key = MacaroonKey::generate(key_bytes);
        Self { root_key }
    }

    /// Mint a new capability token for a container.
    pub fn mint(
        &self,
        container_name: &str,
        capabilities: &ContainerCapabilities,
    ) -> Result<CapabilityToken, TokenError> {
        let identifier = format!("container-{container_name}");
        let location = "hick://gateway:8080".to_string();

        let mut macaroon = Macaroon::create(Some(location), &self.root_key, identifier.into())
            .map_err(|e| TokenError::Macaroon(format!("{e:?}")))?;

        for caveat in capabilities.to_caveats() {
            macaroon.add_first_party_caveat(caveat.into());
        }

        Ok(CapabilityToken {
            macaroon,
            capabilities: capabilities.clone(),
        })
    }

    /// Verify a token and extract its capabilities.
    pub fn verify(&self, token: &CapabilityToken) -> Result<(), TokenError> {
        let mut verifier = Verifier::default();

        // Satisfy all known caveat patterns
        verifier.satisfy_general(verify_hick_caveat);

        verifier
            .verify(&token.macaroon, &self.root_key, Default::default())
            .map_err(|e| TokenError::Verification(format!("{e:?}")))
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-verify_hick_caveat"></a>
### `verify_hick_caveat`

Verifier callback for hick capability caveats.

<storycode:copy id="f-77c16f1d2966-verify_hick_caveat">
/// Verifier callback for hick capability caveats.
fn verify_hick_caveat(caveat: &ByteString) -> bool {
    let s = match std::str::from_utf8(caveat.as_ref()) {
        Ok(s) => s,
        Err(_) => return false,
    };
    NetworkRule::from_caveat(s).is_some()
        || FileRule::from_caveat(s).is_some()
        || SecretRule::from_caveat(s).is_some()
        || VolumeRule::from_caveat(s).is_some()
        || DataAccessRule::from_caveat(s).is_some()
        || ApiSinkRule::from_caveat(s).is_some()
        || PurposeRule::from_caveat(s).is_some()
        || ExpiresRule::from_caveat(s).is_some()
        || MaxCallsRule::from_caveat(s).is_some()
        || RecipientRule::from_caveat(s).is_some()
}

</storycode:copy>
<a id="f-77c16f1d2966-CapabilityToken"></a>
### `CapabilityToken`

CapabilityToken
A capability token (macaroon) with parsed capabilities.

<storycode:copy id="f-77c16f1d2966-CapabilityToken">
// ---------------------------------------------------------------------------
// CapabilityToken
// ---------------------------------------------------------------------------

/// A capability token (macaroon) with parsed capabilities.
pub struct CapabilityToken {
    macaroon: Macaroon,
    capabilities: ContainerCapabilities,
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-CapabilityToken"></a>
### `impl-CapabilityToken`

<storycode:copy id="f-77c16f1d2966-impl-CapabilityToken">
impl CapabilityToken {
    /// Get the parsed capabilities.
    pub fn capabilities(&self) -> &ContainerCapabilities {
        &self.capabilities
    }

    /// Attenuate this token by adding more restrictions.
    /// Returns a new token with the additional caveats. The original is unchanged.
    pub fn attenuate(
        &self,
        additional: &ContainerCapabilities,
    ) -> Result<CapabilityToken, TokenError> {
        let mut new_macaroon = self.macaroon.clone();

        for caveat in additional.to_caveats() {
            new_macaroon.add_first_party_caveat(caveat.into());
        }

        // Track the union of caveats in the in-memory capabilities struct.
        // Actual enforcement is via the macaroon verifier checking all caveats.
        let merged = self.capabilities.merge_caveats(additional);

        Ok(CapabilityToken {
            macaroon: new_macaroon,
            capabilities: merged,
        })
    }

    /// Serialize the token to a string (V2 format).
    pub fn serialize(&self) -> Result<String, TokenError> {
        self.macaroon
            .serialize(macaroon::Format::V2)
            .map_err(|e| TokenError::Macaroon(format!("{e:?}")))
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-impl-fmt-Debug-for-CapabilityToken"></a>
### `impl-fmt::Debug for CapabilityToken`

<storycode:copy id="f-77c16f1d2966-impl-fmt-Debug-for-CapabilityToken">
impl fmt::Debug for CapabilityToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapabilityToken")
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

</storycode:copy>
<a id="f-77c16f1d2966-tests"></a>
### `tests`

<storycode:copy id="f-77c16f1d2966-tests">
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_authority() -> TokenAuthority {
        TokenAuthority::new(b"test-root-key-32bytes-long-xxxxx")
    }

    #[test]
    fn mint_and_verify() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let token = authority.mint("reporter", &caps).unwrap();
        authority.verify(&token).unwrap();
    }

    #[test]
    fn attenuate_adds_restrictions() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network();

        let token = authority.mint("sandbox", &caps).unwrap();

        // Attenuate: also deny all network
        let extra = ContainerCapabilities::new().deny_all_network();
        let attenuated = token.attenuate(&extra).unwrap();

        // Original still verifies
        authority.verify(&token).unwrap();
        // Attenuated also verifies
        authority.verify(&attenuated).unwrap();
    }

    /// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
    #[test]
    fn allows_network_is_deny_by_default() {
        // Silence is not consent: a document that never mentions the network
        // gets none, which is the opposite of `check_network`'s reading and
        // the reason the two are separate methods.
        assert!(!ContainerCapabilities::new().allows_network());
        assert!(
            !ContainerCapabilities::new()
                .deny_all_network()
                .allows_network()
        );
        assert!(
            !ContainerCapabilities::new()
                .allow_file_read("/input/*")
                .allows_network()
        );

        // An explicit grant, with or without a surrounding default-deny.
        assert!(
            ContainerCapabilities::new()
                .allow_network("github.com", "443")
                .allows_network()
        );
        assert!(
            ContainerCapabilities::new()
                .deny_all_network()
                .allow_network("pypi.org", "443")
                .allows_network()
        );
    }

    #[test]
    fn network_check_deny_all_blocks() {
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        assert!(caps.check_network("github.com", 443));
        assert!(!caps.check_network("evil.com", 80));
    }

    #[test]
    fn network_check_wildcard_port() {
        let caps = ContainerCapabilities::new()
            .allow_network("example.com", "*")
            .deny_all_network();

        assert!(caps.check_network("example.com", 80));
        assert!(caps.check_network("example.com", 443));
        assert!(!caps.check_network("other.com", 80));
    }

    #[test]
    fn network_check_no_deny_allows_all() {
        let caps = ContainerCapabilities::new().allow_network("github.com", "443");

        assert!(caps.check_network("github.com", 443));
        assert!(caps.check_network("anything.com", 9999));
    }

    #[test]
    fn file_rules_check() {
        let caps = ContainerCapabilities::new()
            .allow_file_read("/input/*")
            .allow_file_write("/output/*");

        assert!(caps.check_file_read("/input/data.txt"));
        assert!(!caps.check_file_read("/secret/key"));
        assert!(caps.check_file_write("/output/report.html"));
        assert!(!caps.check_file_write("/input/data.txt"));
        // Write implies read
        assert!(caps.check_file_read("/output/report.html"));
    }

    #[test]
    fn serialize_token() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new().deny_all_network();
        let token = authority.mint("test", &caps).unwrap();
        let serialized = token.serialize().unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn caveat_roundtrip() {
        let rule = NetworkRule::allow("github.com", "443");
        let caveat = rule.to_caveat();
        let parsed = NetworkRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, rule);

        let deny = NetworkRule::deny_all();
        let caveat = deny.to_caveat();
        let parsed = NetworkRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, deny);

        let file = FileRule::Write("/output/*".to_string());
        let caveat = file.to_caveat();
        let parsed = FileRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, file);

        let secret = SecretRule {
            env_var: "TOKEN".to_string(),
            secret_name: "my-token".to_string(),
        };
        let caveat = secret.to_caveat();
        let parsed = SecretRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, secret);
    }

    #[test]
    fn capabilities_from_hick_tag() {
        // Simulate building capabilities from parsed hick tags
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network()
            .allow_file_write("/output/*")
            .with_secret("PULUMI_TOKEN", "pulumi-token");

        let caveats = caps.to_caveats();
        assert_eq!(caveats.len(), 5);
        assert!(caveats.contains(&"network = allow github.com:443".to_string()));
        assert!(caveats.contains(&"network = deny *".to_string()));
        assert!(caveats.contains(&"file-write = /output/*".to_string()));
    }

    #[test]
    fn volume_rule_caveat_roundtrip() {
        let read_rule = VolumeRule::read("project", "**");
        let caveat = read_rule.to_caveat();
        assert_eq!(caveat, "volume-read = project:**");
        let parsed = VolumeRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, read_rule);

        let write_rule = VolumeRule::write("output", "Controllers/**");
        let caveat = write_rule.to_caveat();
        assert_eq!(caveat, "volume-write = output:Controllers/**");
        let parsed = VolumeRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, write_rule);
    }

    #[test]
    fn volume_check_no_rules_denies_all() {
        let caps = ContainerCapabilities::new();
        assert!(!caps.check_volume_read("any-vol", "any/path"));
        assert!(!caps.check_volume_write("any-vol", "any/path"));
    }

    #[test]
    fn volume_check_with_rules() {
        let caps = ContainerCapabilities::new()
            .allow_volume_read("project", "**")
            .allow_volume_write("project", "Controllers/*");

        assert!(caps.check_volume_read("project", "Models/User.cs"));
        assert!(caps.check_volume_write("project", "Controllers/Home.cs"));
        assert!(!caps.check_volume_write("project", "Models/User.cs"));

        // Different volume has no rules → denied (closed-by-default)
        assert!(!caps.check_volume_read("other", "any/path"));
    }

    #[test]
    fn volume_token_mint_and_verify() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_volume_read("shared", "**")
            .allow_volume_write("shared", "output/**");

        let token = authority.mint("worker", &caps).unwrap();
        authority.verify(&token).unwrap();

        let caveats = caps.to_caveats();
        assert!(caveats.contains(&"volume-read = shared:**".to_string()));
        assert!(caveats.contains(&"volume-write = shared:output/**".to_string()));
    }

    #[test]
    fn intersect_network_rules() {
        let a = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network();
        let b = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let result = a.intersect(&b);
        // Only github.com:443 is in both; DenyAll preserved
        assert!(result.check_network("github.com", 443));
        assert!(!result.check_network("pypi.org", 443));
    }

    #[test]
    fn intersect_file_rules() {
        let a = ContainerCapabilities::new()
            .allow_file_read("/input/*")
            .allow_file_write("/output/*");
        let b = ContainerCapabilities::new().allow_file_read("/input/*");

        let result = a.intersect(&b);
        assert!(result.check_file_read("/input/data.txt"));
        // /output/* write only in a, not b → removed
        assert!(!result.check_file_write("/output/report.html"));
    }

    #[test]
    fn intersect_secrets() {
        let a = ContainerCapabilities::new()
            .with_secret("TOKEN", "my-token")
            .with_secret("KEY", "my-key");
        let b = ContainerCapabilities::new().with_secret("TOKEN", "my-token");

        let result = a.intersect(&b);
        assert_eq!(result.secret_rules.len(), 1);
        assert_eq!(result.secret_rules[0].env_var, "TOKEN");
    }

    #[test]
    fn attenuate_merges_restrictions() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .allow_file_read("/input/*");

        let token = authority.mint("sandbox", &caps).unwrap();

        // Attenuate: add deny-all (original allows still in effect)
        let restrictor = ContainerCapabilities::new().deny_all_network();

        let attenuated = token.attenuate(&restrictor).unwrap();
        authority.verify(&attenuated).unwrap();

        // Original allows preserved, but non-whitelisted blocked by deny-all
        assert!(attenuated.capabilities().check_network("github.com", 443));
        assert!(attenuated.capabilities().check_network("pypi.org", 443));
        assert!(!attenuated.capabilities().check_network("evil.com", 80));
    }

    #[test]
    fn intersect_produces_common_subset() {
        let source = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network()
            .allow_file_write("/output/*");

        let restrictor = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let result = source.intersect(&restrictor);

        assert!(result.check_network("github.com", 443));
        assert!(!result.check_network("pypi.org", 443));
        // file_write not in restrictor → removed
        assert!(result.file_rules.is_empty());
    }

    #[test]
    fn extended_caveat_roundtrip() {
        let da = DataAccessRule {
            pattern: "secret:ssn".into(),
        };
        let caveat = da.to_caveat();
        assert_eq!(caveat, "data-access = secret:ssn");
        assert_eq!(DataAccessRule::from_caveat(&caveat).unwrap(), da);

        let sink = ApiSinkRule {
            sink_type: "email".into(),
        };
        let caveat = sink.to_caveat();
        assert_eq!(caveat, "api-sink = email");
        assert_eq!(ApiSinkRule::from_caveat(&caveat).unwrap(), sink);

        let purpose = PurposeRule {
            context: "email".into(),
            purpose: "recipient".into(),
        };
        let caveat = purpose.to_caveat();
        assert_eq!(caveat, "purpose = email:recipient");
        assert_eq!(PurposeRule::from_caveat(&caveat).unwrap(), purpose);

        let expires = ExpiresRule {
            expires_at: "2026-02-17T15:00:00Z".into(),
        };
        let caveat = expires.to_caveat();
        assert_eq!(caveat, "expires-at = 2026-02-17T15:00:00Z");
        assert_eq!(ExpiresRule::from_caveat(&caveat).unwrap(), expires);

        let max = MaxCallsRule {
            sink_type: "email".into(),
            max_calls: 1,
        };
        let caveat = max.to_caveat();
        assert_eq!(caveat, "max-calls = email:1");
        assert_eq!(MaxCallsRule::from_caveat(&caveat).unwrap(), max);

        let recip = RecipientRule {
            sink_type: "email".into(),
            recipient: "alice@co.com".into(),
        };
        let caveat = recip.to_caveat();
        assert_eq!(caveat, "recipient = email:alice@co.com");
        assert_eq!(RecipientRule::from_caveat(&caveat).unwrap(), recip);
    }

    #[test]
    fn extended_caps_check_methods() {
        let caps = ContainerCapabilities::new()
            .allow_data_access("public:*")
            .allow_api_sink("email")
            .with_expires("2026-12-31T23:59:59Z")
            .with_max_calls("email", 2)
            .with_recipient("email", "alice@co.com");

        assert!(caps.check_data_access("public:*"));
        assert!(!caps.check_data_access("secret:ssn"));

        assert!(caps.check_api_sink("email"));
        assert!(!caps.check_api_sink("slack"));

        // Before expiry
        assert!(caps.check_expires("2026-06-01T00:00:00Z"));
        // After expiry
        assert!(!caps.check_expires("2027-01-01T00:00:00Z"));

        // Max calls: 0 < 2 ok, 1 < 2 ok, 2 >= 2 fail
        assert!(caps.check_max_calls("email", 0));
        assert!(caps.check_max_calls("email", 1));
        assert!(!caps.check_max_calls("email", 2));

        assert!(caps.check_recipient("email", "alice@co.com"));
        assert!(!caps.check_recipient("email", "bob@co.com"));
        // No recipient rules for slack → allowed
        assert!(caps.check_recipient("slack", "anyone"));
    }

    #[test]
    fn extended_token_mint_and_verify() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .allow_data_access("public:*")
            .with_expires("2026-12-31T23:59:59Z")
            .with_max_calls("email", 5)
            .with_recipient("email", "alice@co.com");

        let token = authority.mint("worker", &caps).unwrap();
        authority.verify(&token).unwrap();

        let caveats = caps.to_caveats();
        assert!(caveats.contains(&"api-sink = email".to_string()));
        assert!(caveats.contains(&"data-access = public:*".to_string()));
        assert!(caveats.contains(&"expires-at = 2026-12-31T23:59:59Z".to_string()));
        assert!(caveats.contains(&"max-calls = email:5".to_string()));
        assert!(caveats.contains(&"recipient = email:alice@co.com".to_string()));
    }
}
</storycode:copy>

### Assemble `crates/hick-token/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-token/src/lib.rs"><storycode:paste select="#f-77c16f1d2966-NetworkRule"/><storycode:paste select="#f-77c16f1d2966-impl-NetworkRule"/><storycode:paste select="#f-77c16f1d2966-FileRule"/><storycode:paste select="#f-77c16f1d2966-impl-FileRule"/><storycode:paste select="#f-77c16f1d2966-VolumeAccess"/><storycode:paste select="#f-77c16f1d2966-VolumeRule"/><storycode:paste select="#f-77c16f1d2966-impl-VolumeRule"/><storycode:paste select="#f-77c16f1d2966-SecretRule"/><storycode:paste select="#f-77c16f1d2966-impl-SecretRule"/><storycode:paste select="#f-77c16f1d2966-DataAccessRule"/><storycode:paste select="#f-77c16f1d2966-impl-DataAccessRule"/><storycode:paste select="#f-77c16f1d2966-ApiSinkRule"/><storycode:paste select="#f-77c16f1d2966-impl-ApiSinkRule"/><storycode:paste select="#f-77c16f1d2966-PurposeRule"/><storycode:paste select="#f-77c16f1d2966-impl-PurposeRule"/><storycode:paste select="#f-77c16f1d2966-ExpiresRule"/><storycode:paste select="#f-77c16f1d2966-impl-ExpiresRule"/><storycode:paste select="#f-77c16f1d2966-MaxCallsRule"/><storycode:paste select="#f-77c16f1d2966-impl-MaxCallsRule"/><storycode:paste select="#f-77c16f1d2966-RecipientRule"/><storycode:paste select="#f-77c16f1d2966-impl-RecipientRule"/><storycode:paste select="#f-77c16f1d2966-ContainerCapabilities"/><storycode:paste select="#f-77c16f1d2966-impl-ContainerCapabilities"/><storycode:paste select="#f-77c16f1d2966-path_matches"/><storycode:paste select="#f-77c16f1d2966-TokenAuthority"/><storycode:paste select="#f-77c16f1d2966-TokenError"/><storycode:paste select="#f-77c16f1d2966-impl-TokenAuthority"/><storycode:paste select="#f-77c16f1d2966-verify_hick_caveat"/><storycode:paste select="#f-77c16f1d2966-CapabilityToken"/><storycode:paste select="#f-77c16f1d2966-impl-CapabilityToken"/><storycode:paste select="#f-77c16f1d2966-impl-fmt-Debug-for-CapabilityToken"/><storycode:paste select="#f-77c16f1d2966-tests"/></storycode:file>

## Test the behavior: `a_containers_network_is_whatever_its_document_declared`

This is the maintained test fragment from `crates/hickory-executor-docker/tests/contract.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-99afca03241b-a_containers_network_is_whatever_its_document_declared"></a>
### `a_containers_network_is_whatever_its_document_declared`

Protects docs/guarantees/execution/declared-capabilities-are-enforced.md

<storycode:copy id="f-99afca03241b-a_containers_network_is_whatever_its_document_declared">
/// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
#[tokio::test(flavor = "multi_thread")]
async fn a_containers_network_is_whatever_its_document_declared() {
    // Both directions against a real daemon: the document — not the
    // deployment — decides which of these two containers can talk out.
    let Ok(ex) = DockerExecutor::new().await else {
        eprintln!("skipping: docker unavailable");
        return;
    };
    if ex.limits_network() != "none" {
        eprintln!("skipping: HICKORY_DOCKER_NETWORK overridden in this environment");
        return;
    }

    ex.declare_capabilities(
        "granted",
        ContainerCapabilities::new().allow_network("github.com", "443"),
    )
    .await
    .unwrap();
    ex.declare_capabilities("denied", ContainerCapabilities::new())
        .await
        .unwrap();

    // The image must already be present: the denied container has no network
    // to pull with.
    if ex.ensure_started("granted", IMAGE).await.is_err() {
        eprintln!("skipping: {IMAGE} not cached locally");
        return;
    }
    ex.ensure_started("denied", IMAGE).await.unwrap();

    let granted = network_mode_of(&ex, "granted");
    let denied = network_mode_of(&ex, "denied");
    ex.shutdown().await.ok();

    assert_eq!(granted, "bridge", "a declared network grant was ignored");
    assert_eq!(
        denied, "none",
        "a container that declared nothing was given a network"
    );
}

</storycode:copy>

## Test the behavior: `a_running_container_cannot_be_re_declared_into_a_different_confinement`

This is the maintained test fragment from `crates/hickory-executor-docker/tests/contract.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-99afca03241b-a_running_container_cannot_be_re_declared_into_a_different_confinement"></a>
### `a_running_container_cannot_be_re_declared_into_a_different_confinement`

Protects docs/guarantees/execution/declared-capabilities-are-enforced.md

<storycode:copy id="f-99afca03241b-a_running_container_cannot_be_re_declared_into_a_different_confinement">
/// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
#[tokio::test(flavor = "multi_thread")]
async fn a_running_container_cannot_be_re_declared_into_a_different_confinement() {
    // One executor serves several documents in a row, so the same container
    // name can be declared twice. Repeating a declaration is fine; changing
    // it after the container exists is not, because `docker run` flags are
    // fixed at creation.
    let limits = DockerLimits {
        network: "none".to_string(),
        allowed_network: "bridge".to_string(),
        ..DockerLimits::default()
    };
    let Ok(ex) = DockerExecutor::with_limits(limits).await else {
        eprintln!("skipping: docker unavailable");
        return;
    };
    ex.declare_capabilities("reused", ContainerCapabilities::new())
        .await
        .unwrap();
    // Offline, so the image must already be cached.
    if ex.ensure_started("reused", IMAGE).await.is_err() {
        eprintln!("skipping: {IMAGE} not cached locally");
        return;
    }

    ex.declare_capabilities("reused", ContainerCapabilities::new())
        .await
        .expect("repeating the same declaration must be a no-op");

    let err = ex
        .declare_capabilities(
            "reused",
            ContainerCapabilities::new().allow_network("github.com", "443"),
        )
        .await
        .expect_err("a conflicting declaration was accepted");
    ex.shutdown().await.ok();

    let msg = err.to_string();
    assert!(
        msg.contains("already running") && msg.contains("Next steps"),
        "unhelpful refusal: {msg}"
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hick-exec/src/volume.rs](contract.md#output-a58d8477e5c2)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/volume_state.rs](../../../mechanisms/crates/hick-literate/src/volume_state.rs.md#output-d1d876a59c4b)
- [crates/hick-token/src/lib.rs](contract.md#output-77c16f1d2966)
- [crates/hickory-executor-docker/src/lib.rs](../../../mechanisms/crates/hickory-executor-docker/src/lib.rs.md#output-772796e266eb)
- [crates/hickory-executor/src/lib.rs](../../../mechanisms/crates/hickory-executor/src/lib.rs.md#output-87ce480a6eab)

### Tests and fixtures

- [crates/hick-literate/tests/capability_enforcement_tests.rs](contract.md#output-8034529313d8)
- [crates/hickory-executor-docker/tests/contract.rs](../../../mechanisms/crates/hickory-executor-docker/tests/contract.rs.md#output-99afca03241b)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
