// Protects docs/guarantees/languages/a-routed-language-is-drawn.md
// and docs/guarantees/languages/a-language-tier-is-measured-not-declared.md
//
// The server's routing table is the only statement of which languages exist.
// This file's job is to prove the editor agrees with it — the check that was
// missing on 2026-09-03, when the two disagreed about nineteen extensions.

import { describe, it, expect } from "vitest";
import {
  boundGrammars,
  languageExtensions,
  languageFromPath,
  normalizeLanguage,
} from "./languages";
import {
  EXTENSION_TO_LANGUAGE,
  HIGHLIGHTED_LANGUAGES,
  ROUTED_LANGUAGE_IDS,
} from "./generated/languages";

describe("the editor and the server know the same languages", () => {
  it("draws every language the server says is drawable", () => {
    const bound = new Set(boundGrammars());
    const missing = HIGHLIGHTED_LANGUAGES.filter((id) => !bound.has(id));
    expect(
      missing,
      `lang_detect marks these Highlight::Editor but languages.ts binds no grammar: ` +
        `${missing.join(", ")}. Either bind one or mark the row Highlight::PlainText.`,
    ).toEqual([]);
  });

  it("claims no grammar the server does not expect", () => {
    const expected = new Set<string>(HIGHLIGHTED_LANGUAGES);
    const extra = boundGrammars().filter((id) => !expected.has(id));
    expect(
      extra,
      `languages.ts binds grammars for ${extra.join(", ")}, which lang_detect does ` +
        `not mark Highlight::Editor. Update the routing table so \`hick lang\` says so.`,
    ).toEqual([]);
  });

  it("resolves every routed extension to its language", () => {
    for (const [ext, id] of Object.entries(EXTENSION_TO_LANGUAGE)) {
      expect(languageFromPath(`some/file.${ext}`), `.${ext}`).toBe(id);
    }
  });

  it("actually attaches a grammar for every drawable language", () => {
    // Not just "an id was recognised": the extension list must really grow,
    // which is what proves the import resolved and the mode loaded.
    for (const id of HIGHLIGHTED_LANGUAGES) {
      expect(languageExtensions(id).length, `${id} produced no language extension`).toBeGreaterThan(1);
    }
  });

  it("degrades honestly for a routed language with no grammar", () => {
    const plain = ROUTED_LANGUAGE_IDS.filter(
      (id) => !(HIGHLIGHTED_LANGUAGES as readonly string[]).includes(id),
    );
    expect(plain).toContain("nix");
    expect(plain).toContain("zig");
    for (const id of plain) {
      expect(normalizeLanguage(id), `${id} should still be a known id`).toBe(id);
      expect(languageExtensions(id).length, `${id} should not attach a grammar`).toBe(1);
    }
  });
});

describe("the languages a JetBrains user arrives with", () => {
  it("highlights the ones that used to open as grey text", () => {
    for (const path of [
      "main.go",
      "Main.java",
      "Main.kt",
      "Main.scala",
      "main.c",
      "main.cpp",
      "app.rb",
      "App.swift",
      "init.lua",
      "Cargo.toml",
      "ci.yaml",
      "index.php",
      "main.dart",
      "analysis.r",
      "Program.fs",
      "Module.vb",
      "build.gradle",
    ]) {
      expect(languageExtensions(path).length, path).toBeGreaterThan(1);
    }
  });

  it("routes the ones that used to highlight but exist nowhere else", () => {
    expect(languageFromPath("schema.sql")).toBe("sql");
    expect(languageFromPath("App.csproj")).toBe("xml");
    expect(languageFromPath("MainWindow.xaml")).toBe("xml");
  });

  it("keeps the spellings people actually type", () => {
    expect(normalizeLanguage("jsx")).toBe("javascriptreact");
    expect(normalizeLanguage("tsx")).toBe("typescriptreact");
    expect(normalizeLanguage("bash")).toBe("shellscript");
    expect(normalizeLanguage("shell")).toBe("shellscript");
    expect(normalizeLanguage("c#")).toBe("csharp");
    expect(normalizeLanguage("c++")).toBe("cpp");
    expect(normalizeLanguage("postgres")).toBe("sql");
    expect(normalizeLanguage("nonsense")).toBeNull();
  });
});
