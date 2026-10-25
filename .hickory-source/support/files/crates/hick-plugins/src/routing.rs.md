<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/routing.rs

For: maintainers changing this file and reviewers following the related requirements.

Each file has one constructor. These links establish navigation, not verified behavioral coverage.

- [Launch the artifact built from the selected compiled project](../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md)
- [A Typed Path Opens That File](../../../../../guarantees/editor-intelligence/a-typed-path-opens-that-file/contract.md)
- [A Routed Language Is Drawn, Or Says It Is Not](../../../../../guarantees/languages/a-routed-language-is-drawn/contract.md)

<book:entry path="crates/hick-plugins/src/routing.rs">
//! Maps file extensions to language identifiers for child LSP routing.

/// Whether the editor can draw this language, or only hold it.
///
/// This is a fact about the product, not about a laptop, and it belongs
/// beside the routing table for the same reason everything else does: the
/// tier ladder promises at Bronze that a routed language is "highlighted",
/// and a promise nothing measures is a promise that drifts. It drifted:
/// `apps/web/src/editor/languages.ts` kept its own list of languages, and on
/// 2026-09-03 the two disagreed in both directions — Go, Java, Kotlin,
/// Scala, C, C++, Ruby, Swift, Lua, TOML and YAML were routed and reported
/// as Bronze while opening as undifferentiated grey text, and SQL and XML
/// highlighted beautifully while being routed nowhere at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    /// A grammar exists and the editor binds it.
    Editor,
    /// No grammar exists for this language. The editor shows plain text and
    /// `hick lang` says so rather than claiming a Bronze it cannot keep.
    PlainText,
}

impl Highlight {
    pub fn is_editor(self) -> bool {
        matches!(self, Highlight::Editor)
    }
}

/// Extension → language id → whether the editor can draw it, and the only
/// place any of the three is written down.
///
/// A table rather than a `match` because this list is read three ways now:
/// forwards, to route a file; backwards, to enumerate the languages Hickory
/// knows about at all (`known_language_ids`, which `hick lang` reports a tier
/// for); and sideways, by `just codegen`, which emits it for the web app so
/// the editor cannot know a different set of languages than the server does.
///
/// A second list of the same languages kept beside this one is exactly the
/// shape of bug that has now shipped five times — `cs` was missing here,
/// which made the C# language server unreachable; `hick init`'s
/// reported-languages list omitted C# for the same reason; both
/// `known_languages()` functions omitted the React ids; and the web app's
/// own table disagreed with this one about nineteen file extensions.
static EXTENSIONS: std::sync::LazyLock<Vec<(&str, &str, Highlight)>> =
    std::sync::LazyLock::new(|| {
        crate::registry()
            .plugins()
            .flat_map(|p| p.interfaces)
            .flat_map(|i| match i {
                crate::Interface::Syntax { extensions, .. } => *extensions,
                _ => &[],
            })
            .copied()
            .collect()
    });

/// Look up the LSP language identifier for a file path based on its extension.
///
/// Returns `None` if the extension is not recognized.
pub fn language_id(path: &str) -> Option<&'static str> {
    crate::registry().language_id(path)
}

/// Every language id this table can produce, sorted and without duplicates.
///
/// This is Hickory's answer to "which languages does a document know how to
/// hold at all" — the Bronze rung of `hick lang`. Derived from the routing
/// table rather than listed again, so a language cannot be reported on
/// without being routable, or routable without being reported.
pub fn known_language_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = EXTENSIONS
        .iter()
        .chain(file_names())
        .map(|(_, id, _)| *id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Whether the editor can draw this language, or only hold it as text.
///
/// Unknown languages are `PlainText`: nothing can draw what nothing routes.
pub fn highlight(language: &str) -> Highlight {
    EXTENSIONS
        .iter()
        .chain(file_names())
        .find(|(_, id, _)| *id == language)
        .map(|(_, _, h)| *h)
        .unwrap_or(Highlight::PlainText)
}

/// The whole table, for `just codegen` to hand to the web app.
pub fn extensions() -> &'static [(&'static str, &'static str, Highlight)] {
    &EXTENSIONS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_extensions() {
        assert_eq!(language_id("main.rs"), Some("rust"));
        assert_eq!(language_id("app.py"), Some("python"));
        assert_eq!(language_id("index.js"), Some("javascript"));
        assert_eq!(language_id("index.ts"), Some("typescript"));
        assert_eq!(language_id("App.jsx"), Some("javascriptreact"));
        assert_eq!(language_id("App.tsx"), Some("typescriptreact"));
        assert_eq!(language_id("page.html"), Some("html"));
        assert_eq!(language_id("style.css"), Some("css"));
        assert_eq!(language_id("data.json"), Some("json"));
        assert_eq!(language_id("config.toml"), Some("toml"));
        assert_eq!(language_id("ci.yaml"), Some("yaml"));
        assert_eq!(language_id("ci.yml"), Some("yaml"));
        assert_eq!(language_id("README.md"), Some("markdown"));
        assert_eq!(language_id("main.go"), Some("go"));
        assert_eq!(language_id("Program.cs"), Some("csharp"));
        assert_eq!(language_id("Main.java"), Some("java"));
        assert_eq!(language_id("main.c"), Some("c"));
        assert_eq!(language_id("main.cpp"), Some("cpp"));
        assert_eq!(language_id("main.cc"), Some("cpp"));
        assert_eq!(language_id("main.cxx"), Some("cpp"));
        assert_eq!(language_id("header.h"), Some("cpp"));
        assert_eq!(language_id("header.hpp"), Some("cpp"));
        assert_eq!(language_id("script.sh"), Some("shellscript"));
        assert_eq!(language_id("script.bash"), Some("shellscript"));
        assert_eq!(language_id("app.rb"), Some("ruby"));
        assert_eq!(language_id("index.php"), Some("php"));
        assert_eq!(language_id("main.swift"), Some("swift"));
        assert_eq!(language_id("Main.kt"), Some("kotlin"));
        assert_eq!(language_id("Main.scala"), Some("scala"));
        assert_eq!(language_id("init.lua"), Some("lua"));
        assert_eq!(language_id("main.zig"), Some("zig"));
        assert_eq!(language_id("flake.nix"), Some("nix"));
    }

    /// The rows added when the web app's private language list was folded
    /// into this one. Each of these was highlighted by the editor and routed
    /// by nothing, or the reverse.
    #[test]
    fn the_jetbrains_pack_is_routed() {
        assert_eq!(language_id("schema.sql"), Some("sql"));
        assert_eq!(language_id("pom.xml"), Some("xml"));
        assert_eq!(language_id("App.csproj"), Some("xml"));
        assert_eq!(language_id("Directory.Build.props"), Some("xml"));
        assert_eq!(language_id("MainWindow.xaml"), Some("xml"));
        assert_eq!(language_id("main.dart"), Some("dart"));
        assert_eq!(language_id("build.gradle"), Some("groovy"));
        assert_eq!(language_id("analysis.r"), Some("r"));
        assert_eq!(language_id("Program.fs"), Some("fsharp"));
        assert_eq!(language_id("Module.vb"), Some("vb"));
        assert_eq!(language_id("build.kts"), Some("kotlin"));
        assert_eq!(language_id("index.mjs"), Some("javascript"));
        assert_eq!(language_id("page.htm"), Some("html"));
    }

    /// An extension is matched case-insensitively, because a `.SQL` dump and
    /// a `.CS` file from an older Windows toolchain are the same languages.
    #[test]
    fn extensions_are_case_insensitive() {
        assert_eq!(language_id("DUMP.SQL"), Some("sql"));
        assert_eq!(language_id("Program.CS"), Some("csharp"));
    }

    #[test]
    fn every_routable_language_is_enumerable() {
        // The two directions must agree: anything `language_id` can return is
        // something `known_language_ids` lists, because they are one table.
        let ids = known_language_ids();
        for (ext, id, _) in EXTENSIONS.iter() {
            assert!(
                ids.contains(id),
                "`.{ext}` routes to `{id}`, which is not enumerated"
            );
        }
        assert!(
            ids.contains(&"csharp"),
            "the row that has gone missing twice"
        );
        // Sorted and deduplicated: `cpp` has five extensions and one entry.
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids, sorted);
    }

    /// Two extensions of one language cannot disagree about whether the
    /// editor can draw it — `.kt` and `.kts` are both Kotlin or neither is.
    #[test]
    fn one_language_has_one_highlight_answer() {
        for (ext, id, h) in EXTENSIONS.iter() {
            assert_eq!(
                highlight(id),
                *h,
                "`.{ext}` and another extension disagree about how `{id}` is drawn"
            );
        }
        assert_eq!(highlight("go"), Highlight::Editor);
        assert_eq!(highlight("nix"), Highlight::PlainText);
        assert_eq!(highlight("no-such-language"), Highlight::PlainText);
    }

    /// Every extension is spelled the way it appears on disk: lowercase, no
    /// leading dot. The lookup lowercases the path's extension, so an
    /// uppercase row here could never match anything.
    #[test]
    fn rows_are_spelled_the_way_the_lookup_reads_them() {
        for (ext, _, _) in EXTENSIONS.iter() {
            assert_eq!(*ext, ext.to_ascii_lowercase(), "`{ext}` cannot match");
            assert!(!ext.starts_with('.'), "`{ext}` has a leading dot");
        }
    }

    #[test]
    fn unknown_extension() {
        assert_eq!(language_id("file.xyz"), None);
        assert_eq!(language_id("file.docx"), None);
    }

    #[test]
    fn path_with_directories() {
        assert_eq!(language_id("src/lib/main.rs"), Some("rust"));
        assert_eq!(language_id("/home/user/project/app.py"), Some("python"));
        assert_eq!(language_id("a/b/c/d.ts"), Some("typescript"));
    }

    #[test]
    fn dotfiles_and_edge_cases() {
        // A bare extension-less filename — rsplit('.') returns the whole string
        assert_eq!(language_id("Makefile"), None);
        // Multiple dots
        assert_eq!(language_id("archive.tar.gz"), None);
        assert_eq!(language_id("my.module.ts"), Some("typescript"));
    }
}

/// Exact filename routes, from plugin declarations.
pub fn file_names() -> Vec<&'static (&'static str, &'static str, Highlight)> {
    crate::registry()
        .plugins()
        .flat_map(|p| p.interfaces)
        .flat_map(|i| match i {
            crate::Interface::Syntax { file_names, .. } => *file_names,
            _ => &[],
        })
        .collect()
}
</book:entry>

</book:doc>
