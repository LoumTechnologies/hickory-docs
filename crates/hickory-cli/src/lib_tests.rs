use super::*;

#[cfg(test)]
mod contained_output_path_tests {
    use super::contained_output_path;
    use std::path::Path;

    #[test]
    fn ordinary_relative_paths_resolve_under_base() {
        let base = Path::new("/tmp/doc");
        assert_eq!(
            contained_output_path(base, "src/app.py").unwrap(),
            base.join("src/app.py")
        );
        // `..` that stays inside the base is allowed.
        assert_eq!(
            contained_output_path(base, "a/../b.txt").unwrap(),
            base.join("a/../b.txt")
        );
        assert_eq!(
            contained_output_path(base, "./c.txt").unwrap(),
            base.join("./c.txt")
        );
    }

    #[test]
    fn absolute_paths_are_refused() {
        // Path::join would DISCARD the base for these — the exact behaviour
        // the guard exists to remove.
        let base = Path::new("/tmp/doc");
        let err = contained_output_path(base, "/etc/cron.d/x").unwrap_err();
        assert!(err.to_string().contains("refusing output path"), "{err}");
    }

    #[test]
    fn parent_escapes_are_refused() {
        let base = Path::new("/tmp/doc");
        for p in ["../evil.sh", "a/../../evil.sh", ".."] {
            let err = contained_output_path(base, p).unwrap_err();
            assert!(
                err.to_string().contains("refusing output path"),
                "{p}: {err}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn drive_prefixes_are_refused() {
        let base = Path::new("C:\\work\\doc");
        assert!(contained_output_path(base, "C:\\evil.txt").is_err());
    }
}

// Protects docs/guarantees/verification/a-drift-report-names-what-differs.md
// Protects docs/guarantees/execution/an-output-path-declared-twice-warns-before-it-runs.md
#[cfg(test)]
mod output_collision_tests {
    use super::output_collision_warnings;

    fn doc(source: &str) -> hick_lang::HickDocument {
        hick_lang::parse(source).expect("the document parses")
    }

    #[test]
    fn a_second_hick_file_at_an_already_ingested_path_warns() {
        // The exact shape that bit the ingest-based tutorial: an author
        // tries to give an ingested file an earlier "stub" version via a
        // second, top-level `hick:file` at the same path.
        let warnings = output_collision_warnings(&doc(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:exec container="c" mount="project:out">
dotnet new console -o out
<hick:ingested from="#x" sha256="abc" at="2026-08-24" files="1" skipped="0">
<hick:file path="app/Program.cs">real content</hick:file>
</hick:ingested>
</hick:exec>
<hick:file path="app/Program.cs">stub</hick:file>
</hick:doc>
"##));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("app/Program.cs"), "{}", warnings[0]);
        assert!(warnings[0].contains("already ingested"), "{}", warnings[0]);
    }

    #[test]
    fn hand_authored_files_beside_an_ingested_one_are_silent() {
        // The false positive an earlier, prefix-based version of this check
        // produced on the real ingest-based tutorial: hand-authored files
        // living in the SAME output directory as an ingested scaffold, at
        // DIFFERENT paths, are the ordinary shape and must never warn.
        let warnings = output_collision_warnings(&doc(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:exec container="c" mount="project:out">
dotnet new console -o out
<hick:ingested from="#x" sha256="abc" at="2026-08-24" files="1" skipped="0">
<hick:file path="app/Program.cs">real content</hick:file>
</hick:ingested>
</hick:exec>
<hick:file path="app/Todo.cs">class Todo {}</hick:file>
<hick:file path="app/TodoStore.cs">class TodoStore {}</hick:file>
</hick:doc>
"##));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_document_with_no_ingest_at_all_is_silent() {
        // A bare scaffolder cell that has not been ingested yet cannot be
        // checked this way — its future output filenames are not known
        // until it runs, which is exactly the run this check happens before.
        let warnings = output_collision_warnings(&doc(r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:file path="app/Program.cs">stub</hick:file>
<hick:exec container="c" mount="project:out">
dotnet new console -o out
</hick:exec>
</hick:doc>
"#));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn an_ingested_files_own_child_path_does_not_self_collide() {
        // The ingested `hick:file` is the one and only declaration of its
        // path — nothing else in the document repeats it — so there is
        // nothing to warn about by itself.
        let warnings = output_collision_warnings(&doc(r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:volume name="project" output="app" />
<hick:exec container="c" mount="project:out">
dotnet new console -o out
<hick:ingested from="#x" sha256="abc" at="2026-08-24" files="1" skipped="0">
<hick:file path="app/Program.cs">real content</hick:file>
</hick:ingested>
</hick:exec>
</hick:doc>
"##));
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}

#[cfg(test)]
mod project_dir_tests {
    use super::project_dir_of;
    use std::path::Path;

    /// `parent()` of a bare filename is `Some("")`, not `None`.
    ///
    /// The trap that made every `hick run <bare-filename>` on the machine
    /// share one scratch directory: `unwrap_or(".")` never fires for
    /// `Some("")`, so the empty path went straight through to a hash that is
    /// the same constant for everybody.
    #[test]
    fn a_document_named_without_a_directory_lives_in_the_current_one() {
        assert_eq!(project_dir_of(Path::new("d.hick")), Path::new("."));
        assert_eq!(
            project_dir_of(Path::new("notes/d.hick")),
            Path::new("notes")
        );
        assert_eq!(project_dir_of(Path::new("/tmp/d.hick")), Path::new("/tmp"));
        // Never the empty path, whatever it is handed.
        for name in ["d.hick", "notes/d.hick", "/tmp/d.hick", ""] {
            assert!(
                !project_dir_of(Path::new(name)).as_os_str().is_empty(),
                "{name} produced an empty project directory"
            );
        }
    }
}
