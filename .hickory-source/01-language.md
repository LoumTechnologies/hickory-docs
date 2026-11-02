<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# 1. Build an exact project tree

We have a Rust workspace. Before we execute a command or draw an editor, we need
something both hosts can agree on: the files a document constructs. We will build
that value in memory first. This lets us test construction without granting it
filesystem or process access.

## Represent a file without losing its identity

A text string cannot distinguish a regular file from a symlink, and it cannot
tell us whether a script is executable. Add those properties to each entry. Keep
byte origins alongside the output so later chapters can return an edit to the
right Markdown range. Origins use bytes, not the editor's character positions.

<a id="f-a485394d6759-EntryKind"></a>
<book:copy id="f-a485394d6759-EntryKind">
/// Portable entry metadata. Symlink bytes are the exact UTF-8 target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryKind {
    File,
    Symlink,
}

</book:copy>

<a id="f-a485394d6759-ByteOrigin"></a>
<book:copy id="f-a485394d6759-ByteOrigin">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteOrigin {
    pub output: (usize, usize),
    pub document: String,
    pub source: (usize, usize),
}

</book:copy>

<a id="f-a485394d6759-ProjectEntry"></a>
<book:copy id="f-a485394d6759-ProjectEntry">
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
    pub bytes: Arc<[u8]>,
    pub kind: EntryKind,
    pub executable: bool,
    pub origins: Vec<ByteOrigin>,
}
</book:copy>

The [inventory requirement](guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md)
is why file kind and executable status belong here. They are part of the program
we reconstruct, rather than incidental properties of this checkout.

## Define what an unchanged output means

Next, we need to compare two constructions. Moving a fragment changes its source
coordinates but should not change the generated program. Compare bytes, kind and
executable status; leave origin coordinates out of output equality.

<a id="f-a485394d6759-impl-ProjectEntry"></a>
<book:copy id="f-a485394d6759-impl-ProjectEntry">
impl ProjectEntry {
    pub fn file(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
            kind: EntryKind::File,
            executable: false,
            origins: vec![],
        }
    }
    pub fn same_output(&self, other: &Self) -> bool {
        self.bytes == other.bytes && self.kind == other.kind && self.executable == other.executable
    }
}

</book:copy>

<a id="f-a485394d6759-ProjectTree"></a>
<book:copy id="f-a485394d6759-ProjectTree">
/// Immutable entry sharing makes before/after snapshots cheap. There is no IO.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectTree {
    entries: BTreeMap<String, Arc<ProjectEntry>>,
}
</book:copy>

Share entries through `Arc` so candidate trees can reuse unchanged values. Add
path validation, insertion, collision refusal and sorted comparison in the
[rest of the tree implementation](mechanisms/crates/hick-lang/src/project/tree.rs.md).
<book:include file="mechanisms/crates/hick-lang/src/project/tree.rs.md" />

## Make the test distinguish equal text from an equal project

Construct a binary asset, a symlink and an executable script. Then change a byte,
add an unexpected path and clear the executable bit. The assertion must name all
three differences. Invalid portable paths and case collisions must also refuse.

<a id="f-005c1e9344c7-inventory_checks_binary_modes_symlinks_and_portable_paths"></a>
<book:copy id="f-005c1e9344c7-inventory_checks_binary_modes_symlinks_and_portable_paths">
// Guarantee: docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md
#[test]
fn inventory_checks_binary_modes_symlinks_and_portable_paths() {
    let project = generate("<hick:project exclude=\".git/ node_modules/\"/><hick:entry path=\"asset.bin\" encoding=\"hex\">00 ff 01</hick:entry><hick:entry path=\"link\" kind=\"symlink\">asset.bin</hick:entry><hick:file path=\"run.sh\" executable=\"true\">echo ok\n</hick:file>").unwrap();
    let mut expected = project.new.clone();
    expected
        .insert("extra.lock", ProjectEntry::file(b"lock".to_vec()))
        .unwrap();
    let mut asset = expected.get("asset.bin").unwrap().as_ref().clone();
    asset.bytes = vec![0, 254, 1].into();
    expected.insert("asset.bin", asset).unwrap();
    let mut script = expected.get("run.sh").unwrap().as_ref().clone();
    script.executable = false;
    expected.insert("run.sh", script).unwrap();
    let differences = project.new.compare(&expected);
    assert_eq!(
        differences
            .iter()
            .map(|d| d.path.as_str())
            .collect::<Vec<_>>(),
        vec!["asset.bin", "extra.lock", "run.sh"]
    );
    assert!(!project.inventory.contains("project.md"));
    assert!(!project.inventory.contains("node_modules/x"));
    assert!(project.inventory.contains("migration.sql"));
    for path in [
        "../x", "/x", "x//y", "x\\y", "C:x", "NUL.txt", "x.", "x/../y", "é.rs",
    ] {
        assert!(validate_project_path(path).is_err());
    }
    let mut t = tree(&[("A", b"x")]);
    assert!(t.insert("a", ProjectEntry::file(b"y".to_vec())).is_err());
    assert!(
        t.insert("A/child", ProjectEntry::file(b"y".to_vec()))
            .is_err()
    );
}
</book:copy>

The fixture helpers and remaining project witnesses are maintained here:
<book:include file="mechanisms/crates/hick-lang/tests/project.rs.md" />

## Evaluate the Markdown into that tree

Now give construction operations explicit predecessor states. Collect named copy
fragments across the declared source closure, assemble files with paste selections,
and apply exact patches against their named inputs. Refuse unknown constructs
rather than returning a plausible but incomplete tree. The evaluator below adds
those operations to the value we just built.

<book:include file="mechanisms/crates/hick-lang/src/project/evaluate.rs.md" />

Run the focused tree witness from the completed checkout:

```sh
cargo test --locked -p hick-lang --test project \
  inventory_checks_binary_modes_symlinks_and_portable_paths -- --exact
```

We can now describe and compare a complete output without writing it. Next,
[retain a scaffolder's output and author a patch above it](02-scaffolds.md).
</book:doc>
