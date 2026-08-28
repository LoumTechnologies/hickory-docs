//! Maps file extensions to language identifiers for child LSP routing.

/// Extension → language id, and the only place either is written down.
///
/// A table rather than a `match` because this list is read two ways now:
/// forwards, to route a file, and backwards, to enumerate the languages
/// Hickory knows about at all (`known_language_ids`, which `hick lang`
/// reports a tier for). A second list of the same languages kept beside this
/// one is exactly the shape of bug that has already shipped twice — `cs` was
/// missing here, which made the C# language server unreachable, and later
/// `hick init`'s reported-languages list omitted C# for the same reason.
const EXTENSIONS: &[(&str, &str)] = &[
    ("rs", "rust"),
    ("py", "python"),
    ("js", "javascript"),
    ("ts", "typescript"),
    ("jsx", "javascriptreact"),
    ("tsx", "typescriptreact"),
    ("html", "html"),
    ("css", "css"),
    ("json", "json"),
    ("toml", "toml"),
    ("yaml", "yaml"),
    ("yml", "yaml"),
    ("md", "markdown"),
    ("go", "go"),
    // C#. This table is the ONLY thing that routes a generated file to a
    // language, for the language server and the debugger both — so until this
    // row existed, `hick lsp install csharp` fetched a server that could never
    // be reached: every `.cs` virtual file got `language_id: None` and was
    // skipped before discovery was consulted.
    ("cs", "csharp"),
    ("java", "java"),
    ("c", "c"),
    ("cpp", "cpp"),
    ("cc", "cpp"),
    ("cxx", "cpp"),
    ("h", "cpp"),
    ("hpp", "cpp"),
    ("sh", "shellscript"),
    ("bash", "shellscript"),
    ("rb", "ruby"),
    ("php", "php"),
    ("swift", "swift"),
    ("kt", "kotlin"),
    ("scala", "scala"),
    ("lua", "lua"),
    ("zig", "zig"),
    ("nix", "nix"),
];

/// Look up the LSP language identifier for a file path based on its extension.
///
/// Returns `None` if the extension is not recognized.
pub fn language_id(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?;
    EXTENSIONS
        .iter()
        .find(|(candidate, _)| *candidate == ext)
        .map(|(_, id)| *id)
}

/// Every language id this table can produce, sorted and without duplicates.
///
/// This is Hickory's answer to "which languages does a document know how to
/// hold at all" — the Bronze rung of `hick lang`. Derived from the routing
/// table rather than listed again, so a language cannot be reported on
/// without being routable, or routable without being reported.
pub fn known_language_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = EXTENSIONS.iter().map(|(_, id)| *id).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
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

    #[test]
    fn every_routable_language_is_enumerable() {
        // The two directions must agree: anything `language_id` can return is
        // something `known_language_ids` lists, because they are one table.
        let ids = known_language_ids();
        for (ext, id) in EXTENSIONS {
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
