//! Maps file extensions to language identifiers for child LSP routing.

/// Look up the LSP language identifier for a file path based on its extension.
///
/// Returns `None` if the extension is not recognized.
pub fn language_id(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?;
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "js" => Some("javascript"),
        "ts" => Some("typescript"),
        "jsx" => Some("javascriptreact"),
        "tsx" => Some("typescriptreact"),
        "html" => Some("html"),
        "css" => Some("css"),
        "json" => Some("json"),
        "toml" => Some("toml"),
        "yaml" | "yml" => Some("yaml"),
        "md" => Some("markdown"),
        "go" => Some("go"),
        // C#. This table is the ONLY thing that routes a generated file to a
        // language, for the language server and the debugger both — so until
        // this line existed, `hick lsp install csharp` fetched a server that
        // could never be reached: every `.cs` virtual file got
        // `language_id: None` and was skipped before discovery was consulted.
        "cs" => Some("csharp"),
        "java" => Some("java"),
        "c" => Some("c"),
        "cpp" | "cc" | "cxx" => Some("cpp"),
        "h" | "hpp" => Some("cpp"),
        "sh" | "bash" => Some("shellscript"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "kt" => Some("kotlin"),
        "scala" => Some("scala"),
        "lua" => Some("lua"),
        "zig" => Some("zig"),
        "nix" => Some("nix"),
        _ => None,
    }
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
