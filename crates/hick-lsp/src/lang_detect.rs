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

use Highlight::{Editor, PlainText};

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
const EXTENSIONS: &[(&str, &str, Highlight)] = &[
    ("rs", "rust", Editor),
    ("py", "python", Editor),
    ("js", "javascript", Editor),
    ("mjs", "javascript", Editor),
    ("cjs", "javascript", Editor),
    ("ts", "typescript", Editor),
    ("jsx", "javascriptreact", Editor),
    ("tsx", "typescriptreact", Editor),
    ("html", "html", Editor),
    ("htm", "html", Editor),
    ("css", "css", Editor),
    ("json", "json", Editor),
    ("toml", "toml", Editor),
    ("yaml", "yaml", Editor),
    ("yml", "yaml", Editor),
    ("md", "markdown", Editor),
    ("go", "go", Editor),
    // C#. This table is the ONLY thing that routes a generated file to a
    // language, for the language server and the debugger both — so until this
    // row existed, `hick lsp install csharp` fetched a server that could never
    // be reached: every `.cs` virtual file got `language_id: None` and was
    // skipped before discovery was consulted.
    ("cs", "csharp", Editor),
    ("java", "java", Editor),
    ("c", "c", Editor),
    ("cpp", "cpp", Editor),
    ("cc", "cpp", Editor),
    ("cxx", "cpp", Editor),
    ("h", "cpp", Editor),
    ("hpp", "cpp", Editor),
    ("sh", "shellscript", Editor),
    ("bash", "shellscript", Editor),
    ("zsh", "shellscript", Editor),
    ("rb", "ruby", Editor),
    ("php", "php", Editor),
    ("swift", "swift", Editor),
    ("kt", "kotlin", Editor),
    ("kts", "kotlin", Editor),
    ("scala", "scala", Editor),
    ("sc", "scala", Editor),
    ("lua", "lua", Editor),
    // No CodeMirror grammar exists for either, so the editor holds them as
    // text and the ladder reports that instead of claiming otherwise.
    ("zig", "zig", PlainText),
    ("nix", "nix", PlainText),
    // SQL is DataGrip's whole subject and was highlighted by the web app for
    // months while being routed nowhere: `is_data_language` already named it,
    // which made that function's `sql` arm dead code.
    ("sql", "sql", Editor),
    // XML, and the four .NET spellings of it. A document that ingests
    // `dotnet new` gets a `.csproj` whether or not it asked for one.
    ("xml", "xml", Editor),
    ("csproj", "xml", Editor),
    ("props", "xml", Editor),
    ("targets", "xml", Editor),
    ("xaml", "xml", Editor),
    ("xsd", "xml", Editor),
    // The rest of the JetBrains pack's languages.
    ("dart", "dart", Editor),
    ("groovy", "groovy", Editor),
    ("gradle", "groovy", Editor),
    ("r", "r", Editor),
    ("fs", "fsharp", Editor),
    ("fsx", "fsharp", Editor),
    ("vb", "vb", Editor),
];

/// Look up the LSP language identifier for a file path based on its extension.
///
/// Returns `None` if the extension is not recognized.
pub fn language_id(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?;
    let lowered = ext.to_ascii_lowercase();
    EXTENSIONS
        .iter()
        .find(|(candidate, _, _)| *candidate == lowered)
        .map(|(_, id, _)| *id)
}

/// Every language id this table can produce, sorted and without duplicates.
///
/// This is Hickory's answer to "which languages does a document know how to
/// hold at all" — the Bronze rung of `hick lang`. Derived from the routing
/// table rather than listed again, so a language cannot be reported on
/// without being routable, or routable without being reported.
pub fn known_language_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = EXTENSIONS.iter().map(|(_, id, _)| *id).collect();
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
        .find(|(_, id, _)| *id == language)
        .map(|(_, _, h)| *h)
        .unwrap_or(Highlight::PlainText)
}

/// The whole table, for `just codegen` to hand to the web app.
pub fn extensions() -> &'static [(&'static str, &'static str, Highlight)] {
    EXTENSIONS
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
        for (ext, id, _) in EXTENSIONS {
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
        for (ext, id, h) in EXTENSIONS {
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
        for (ext, _, _) in EXTENSIONS {
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
