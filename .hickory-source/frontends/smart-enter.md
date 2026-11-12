<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Choose indentation before inserting a native newline

For maintainers changing Enter behavior consistently across the simple editors.

Enter is a local source edit. It must not wait for a language server or formatter,
reconstruct an editor, rewrite unrelated lines or claim a save. Smart is the default;
Copy retains the current prefix, and None inserts only the existing line terminator.
The platform still owns composition, selection, permissions and undo.

## Describe syntax families once

The same bundled registry feeds all three evaluators. Extensions and fence names
select indentation policies, not a second claim about language-server support.
Strings and comments are removed from structural decisions before applying a rule.
Balanced delimiters handle expressions and braces; keyword rules handle Python,
Ruby, Lua, shell, Elixir, Julia, SQL and Visual Basic; tags handle HTML/XML.
Unknown contexts retain their prefix. These are editing heuristics, not full
parsers: raw strings, heredocs, preprocessor branches and language extensions may
need a formatter or manual correction. No automatic pairing or save-time cleanup
is implied. A scan is bounded to 65,536 preceding UTF-16 units and 4,096 units per
line. Beyond those bounds, carry indentation rather than blocking a keystroke or
guessing lexical state from an arbitrary truncated prefix.

A rule has explicit quote/comment delimiters and separate opening, closing,
branch and post-statement patterns. A Python `return` dedents only after its
expression has finished; a closing bracket returns to its recorded opening indent.
Delimiter strings escape the less-than character in JSON so Hickory cannot
interpret a comment opener as document syntax. Decoder checks reject empty or
incomplete delimiters before scanning. The two outputs below replay the same authored bytes: browser/test input and the
resource shared by Apple and Android.

<ui:copy id="simple-enter-language-rules">
{
  "maxContext": 65536,
  "maxLine": 4096,
  "profiles": [
    {
      "id": "python",
      "names": [
        "py",
        "pyw",
        "pyi",
        "python"
      ],
      "comments": [
        "#"
      ],
      "blocks": [],
      "multiline": [
        "\"\"\"",
        "'''"
      ],
      "quotes": "\"'",
      "open": ":\\s*$",
      "close": "",
      "middle": "^(?:elif|else|except|finally|case)\\b",
      "after": "^(?:return|raise|break|continue|pass)\\b",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": true,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "braces",
      "names": [
        "c",
        "h",
        "cpp",
        "cc",
        "cxx",
        "hpp",
        "hh",
        "hxx",
        "cs",
        "csharp",
        "java",
        "js",
        "javascript",
        "jsx",
        "javascriptreact",
        "mjs",
        "cjs",
        "ts",
        "typescript",
        "tsx",
        "typescriptreact",
        "mts",
        "cts",
        "go",
        "swift",
        "kt",
        "kotlin",
        "kts",
        "dart",
        "groovy",
        "r",
        "zig",
        "nix",
        "tf",
        "terraform",
        "hcl",
        "scss",
        "less",
        "css",
        "m",
        "mm",
        "objc",
        "objective-c",
        "sol",
        "solidity",
        "proto",
        "pl",
        "pm",
        "perl",
        "php",
        "phtml"
      ],
      "comments": [
        "//",
        "#"
      ],
      "blocks": [
        [
          "/*",
          "*/"
        ]
      ],
      "multiline": [
        "\"\"\"",
        "`"
      ],
      "quotes": "\"'",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "^(?:else\\s+)?(?:if|for|while|switch|catch|with)\\b.*\\)\\s*$|^else\\s*$|=>\\s*$",
      "regexLiterals": true
    },
    {
      "id": "rust",
      "names": [
        "rs",
        "rust"
      ],
      "comments": [
        "//"
      ],
      "blocks": [
        [
          "/*",
          "*/"
        ]
      ],
      "multiline": [
        "\"\"\"",
        "`"
      ],
      "quotes": "\"'",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "scala",
      "names": [
        "scala",
        "sc"
      ],
      "comments": [
        "//"
      ],
      "blocks": [
        [
          "/*",
          "*/"
        ]
      ],
      "multiline": [
        "\"\"\"",
        "`"
      ],
      "quotes": "\"'",
      "open": "(?::|=|=>|\\bthen|\\bdo|\\bmatch)\\s*$",
      "close": "^end\\b",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": true,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "json",
      "names": [
        "json",
        "jsonc",
        "json5"
      ],
      "comments": [
        "//"
      ],
      "blocks": [
        [
          "/*",
          "*/"
        ]
      ],
      "multiline": [],
      "quotes": "\"",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "yaml",
      "names": [
        "yaml",
        "yml"
      ],
      "comments": [
        "#"
      ],
      "blocks": [],
      "multiline": [],
      "quotes": "\"'",
      "open": ":\\s*$|^-\\s*$",
      "close": "",
      "middle": "",
      "after": "",
      "literal": ":\\s*[|>][-+0-9]*\\s*$",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": true,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "toml",
      "names": [
        "toml"
      ],
      "comments": [
        "#"
      ],
      "blocks": [],
      "multiline": [
        "\"\"\"",
        "'''"
      ],
      "quotes": "\"'",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "ruby",
      "names": [
        "rb",
        "ruby",
        "rake",
        "gemspec",
        "rakefile",
        "gemfile"
      ],
      "comments": [
        "#"
      ],
      "blocks": [
        [
          "=begin",
          "=end"
        ]
      ],
      "multiline": [],
      "quotes": "\"'",
      "open": "^(?:class|module|def|if|unless|case|while|until|for|begin)\\b(?!.*\\bend\\s*$)|\\bdo(?:\\s*\\|[^|]*\\|)?\\s*$",
      "close": "^end\\b",
      "middle": "^(?:else|elsif|when|rescue|ensure)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": true
    },
    {
      "id": "lua",
      "names": [
        "lua"
      ],
      "comments": [
        "--"
      ],
      "blocks": [
        [
          "--[[",
          "]]"
        ]
      ],
      "multiline": [
        "[["
      ],
      "quotes": "\"'",
      "open": "^(?:local\\s+)?function\\b|\\b(?:then|do|repeat)\\s*$",
      "close": "^(?:end|until)\\b",
      "middle": "^(?:else|elseif)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "shell",
      "names": [
        "sh",
        "bash",
        "zsh",
        "ksh",
        "shellscript"
      ],
      "comments": [
        "#"
      ],
      "blocks": [],
      "multiline": [
        "`"
      ],
      "quotes": "\"'",
      "open": "\\b(?:then|do)\\s*$|^case\\b.*\\bin\\s*$",
      "close": "^(?:fi|done|esac)\\b",
      "middle": "^(?:else|elif)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "powershell",
      "names": [
        "ps1",
        "psm1",
        "psd1",
        "powershell"
      ],
      "comments": [
        "#"
      ],
      "blocks": [
        [
          "\u003c#",
          "#>"
        ]
      ],
      "multiline": [],
      "quotes": "\"'",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "elixir",
      "names": [
        "ex",
        "exs",
        "elixir"
      ],
      "comments": [
        "#"
      ],
      "blocks": [],
      "multiline": [
        "\"\"\"",
        "`"
      ],
      "quotes": "\"'",
      "open": "\\bdo\\s*$|->\\s*$",
      "close": "^end\\b",
      "middle": "^(?:else|rescue|after|catch)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "julia",
      "names": [
        "jl",
        "julia"
      ],
      "comments": [
        "#"
      ],
      "blocks": [
        [
          "#=",
          "=#"
        ]
      ],
      "multiline": [
        "\"\"\"",
        "`"
      ],
      "quotes": "\"'",
      "open": "^(?:function|macro|struct|mutable\\s+struct|module|baremodule|if|for|while|try|begin|let|quote)\\b",
      "close": "^end\\b",
      "middle": "^(?:else|elseif|catch|finally)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "sql",
      "names": [
        "sql"
      ],
      "comments": [
        "--"
      ],
      "blocks": [
        [
          "/*",
          "*/"
        ]
      ],
      "multiline": [],
      "quotes": "\"'",
      "open": "\\b(?:BEGIN|THEN|LOOP|CASE)\\s*$|^CASE\\b",
      "close": "^END\\b",
      "middle": "^(?:ELSE|ELSIF|WHEN)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": true,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "vb",
      "names": [
        "vb",
        "vbs",
        "visualbasic"
      ],
      "comments": [
        "'"
      ],
      "blocks": [],
      "multiline": [],
      "quotes": "\"",
      "open": "^(?:(?:Public|Private|Protected|Shared|Async|Friend|Partial)\\s+)*(?:Sub|Function|Class|Module|Namespace|Structure|Try|Select\\s+Case|For|While|Do)\\b|^If\\b.*\\bThen\\s*$",
      "close": "^(?:End\\b|Next\\b|Loop\\b|Wend\\b)",
      "middle": "^(?:Else|ElseIf|Catch|Finally|Case)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": true,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "haskell",
      "names": [
        "hs",
        "lhs",
        "haskell"
      ],
      "comments": [
        "--"
      ],
      "blocks": [
        [
          "{-",
          "-}"
        ]
      ],
      "multiline": [],
      "quotes": "\"",
      "open": "(?:=|->|\\bwhere|\\bdo|\\blet|\\bof|\\bthen|\\belse)\\s*$",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": true,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "fsharp",
      "names": [
        "fs",
        "fsx",
        "fsharp"
      ],
      "comments": [
        "//"
      ],
      "blocks": [
        [
          "(*",
          "*)"
        ]
      ],
      "multiline": [
        "\"\"\"",
        "`"
      ],
      "quotes": "\"",
      "open": "(?:=|->|\\bthen|\\belse|\\bdo|\\bwith)\\s*$",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": true,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "html",
      "names": [
        "html",
        "htm"
      ],
      "comments": [],
      "blocks": [
        [
          "\u003c!--",
          "-->"
        ]
      ],
      "multiline": [],
      "quotes": "\"'",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": true,
      "voidTags": [
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta",
        "param",
        "source",
        "track",
        "wbr"
      ],
      "foldCase": true,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "xml",
      "names": [
        "xml",
        "xsl",
        "xslt",
        "xsd",
        "svg",
        "vue",
        "svelte"
      ],
      "comments": [],
      "blocks": [
        [
          "\u003c!--",
          "-->"
        ]
      ],
      "multiline": [],
      "quotes": "\"'",
      "open": "",
      "close": "",
      "middle": "",
      "after": "",
      "literal": "",
      "tags": true,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    },
    {
      "id": "matlab",
      "names": [
        "matlab"
      ],
      "comments": [
        "%"
      ],
      "blocks": [
        [
          "%{",
          "%}"
        ]
      ],
      "multiline": [],
      "quotes": "\"",
      "open": "^(?:function|if|for|parfor|while|switch|try|classdef|properties|methods)\\b",
      "close": "^end\\b",
      "middle": "^(?:else|elseif|case|otherwise|catch)\\b",
      "after": "",
      "literal": "",
      "tags": false,
      "voidTags": [],
      "foldCase": false,
      "offside": false,
      "next": "",
      "regexLiterals": false
    }
  ]
}
</ui:copy>

<ui:file path="apps/simple-contract/enter-rules.json"><ui:paste select="#simple-enter-language-rules"/></ui:file>

<ui:file path="apps/apple-simple/Sources/Resources/enter-rules.json"><ui:paste select="#simple-enter-language-rules"/></ui:file>

## Load the browser policy independently of editor capabilities

Decode the shared rule set and compile patterns lazily. Missing native resources
fall back to prefix copying. The returned edit uses UTF-16 coordinates, matching
the platform editor and the common fixtures.

<ui:copy id="simple-enter-ts-policy">
import registry from "../../../simple-contract/enter-rules.json";
export type EnterMode = "smart" | "copy" | "none";
export type EnterPlan = { from: number; to: number; text: string; caret: number };
type Profile = { id: string; names: string[]; comments: string[]; blocks: string[][]; multiline: string[]; quotes: string; open: string; close: string; middle: string; next: string; regexLiterals: boolean; after: string; literal: string; tags: boolean; voidTags: string[]; foldCase: boolean; offside: boolean };
const compiled = new Map<string, RegExp>();
function matches(profile: Profile, pattern: string, text: string): boolean {
  if (!pattern) return false;
  const key = `${profile.foldCase}:${pattern}`;
  let regex = compiled.get(key);
  if (!regex) { regex = new RegExp(pattern, profile.foldCase ? "i" : ""); compiled.set(key, regex); }
  return regex.test(text);
}
function profileFor(path: string): Profile | undefined {
  const name = path.split(/[\\/]/).pop()!.toLowerCase();
  const ext = name.includes(".") ? name.slice(name.lastIndexOf(".") + 1) : name;
  return (registry.profiles as Profile[]).find(profile => profile.names.includes(ext) && profile.blocks.every(pair => pair.length === 2 && pair.every(Boolean)) && profile.comments.every(Boolean) && profile.multiline.every(Boolean));
}
function outdent(prefix: string, unit: string): string {
  if (prefix.endsWith("\t")) return prefix.slice(0, -1);
  let count = 0; while (count < unit.length && prefix.endsWith(" ")) { prefix = prefix.slice(0, -1); count++; }
  return prefix;
}
export function enterMode(): EnterMode {
  try { const mode = localStorage.getItem("editor.enterIndent"); return mode === "copy" || mode === "none" ? mode : "smart"; }
  catch { return "smart"; }
}
</ui:copy>

## Keep browser lexical state across the bounded prefix

Hide strings and comments before counting brackets. Remember each opening
indentation and unfinished logical statement, so an inner closing delimiter does
not drift the following line right and `return ( … )` dedents only at completion.
Literal YAML bodies and unfinished strings retain their exact prefix.

<ui:copy id="simple-enter-ts-scanner">
// The scanner retains only lexical state and opening indentation, never an AST.
class EnterScanner {
  block = "";
  quoted = false;
  brackets: { token: string; indent: string; line: number }[] = [];
  keywords: string[] = [];
  logical = "";
  logicalIndent = "";
  literal: string | null = null;
  singleIndent: string | null = null;
  clean(row: string, profile: Profile): string {
    let code = "", index = 0; this.quoted = false;
    while (index < row.length) {
      if (this.block) {
        if (row.startsWith(this.block, index) && (index === 0 || row[index - 1] !== "\\")) { index += this.block.length; this.block = ""; }
        else index++;
        code += " "; continue;
      }
      const block = profile.blocks.find(pair => row.startsWith(pair[0], index));
      const multi = profile.multiline.find(mark => row.startsWith(mark, index));
      if (block || multi) { this.block = block ? block[1] : multi!; index += block ? block[0].length : multi!.length; code += " "; continue; }
      if (profile.comments.some(mark => row.startsWith(mark, index))) break;
      if (profile.regexLiterals && row[index] === "/" && /(?:^|\b(?:return|throw|yield|case)|[=(:,!&|?;{}\[>])\s*$/.test(code)) {
        index++; let characterClass = false, closed = false;
        while (index < row.length) {
          const next = row[index++];
          if (next === "\\") { index++; continue; }
          if (next === "[") characterClass = true; if (next === "]") characterClass = false;
          if (next === "/" && !characterClass) { closed = true; break; }
        }
        if (!closed) { this.quoted = true; this.block = "/"; }
        code += "~"; continue;
      }
      const char = row[index++];
      if (profile.quotes.includes(char)) {
        if (char === "'" && profile.id === "rust" && !row.slice(index).includes("'")) { code += char; continue; }
        let closed = false;
        while (index < row.length) { const next = row[index++]; if (next === "\\") { index++; continue; } if (next === char) { closed = true; break; } }
        if (!closed) { this.quoted = true; this.block = char; }
        code += "~"; continue;
      }
      code += char;
    }
    return code.trim();
  }
  line(row: string, profile: Profile, line: number, unit: string): { indent: string; split: string; literal: boolean } {
    const prefix = row.match(/^[ \t]*/)![0];
    if (this.literal !== null && (!row.trim() || prefix.length > this.literal.length)) return { indent: prefix, split: "", literal: true };
    this.literal = null;
    const wasBlock = !!this.block, code = this.clean(row, profile), startDepth = this.brackets.length;
    if (wasBlock || this.block || this.quoted || !code) return { indent: prefix, split: "", literal: true };
    if (!startDepth) { this.logical = code; this.logicalIndent = prefix; }
    let base = prefix;
    if (matches(profile, profile.close, code)) base = this.keywords.pop() ?? outdent(prefix, unit);
    const middle = matches(profile, profile.middle, code);
    if (middle && !profile.offside && this.keywords.length) base = this.keywords[this.keywords.length - 1];
    if (profile.tags) {
      for (const match of code.matchAll(/<(\/?)([A-Za-z][\w:.-]*)\b[^>]*>/g)) {
        const tag = profile.foldCase ? match[2].toLowerCase() : match[2];
        if (match[1]) { if (this.brackets.at(-1)?.token === `</${tag}>`) base = this.brackets.pop()!.indent; }
        else if (!match[0].endsWith("/>") && !profile.voidTags.includes(tag)) this.brackets.push({ token: `</${tag}>`, indent: base, line });
      }
    } else for (const char of code) {
      if ("{[(".includes(char)) this.brackets.push({ token: char, indent: base, line });
      const opening = "}])".indexOf(char);
      if (opening >= 0 && this.brackets.at(-1)?.token === "{[("[opening]) base = this.brackets.pop()!.indent;
    }
    const top = this.brackets.at(-1);
    if (top) return { indent: top.line === line || base.length <= top.indent.length ? top.indent + unit : base, split: top.token.startsWith("</") ? top.token : "}])"["{[(".indexOf(top.token)], literal: false };
    if (matches(profile, profile.next, code)) {
      this.singleIndent ??= base; return { indent: base + unit, split: "", literal: false };
    }
    if (matches(profile, profile.open, code) || middle) {
      if (!middle) this.keywords.push(base);
      return { indent: base + unit, split: "", literal: false };
    }
    if (matches(profile, profile.literal, code)) { this.literal = base; return { indent: base + unit, split: "", literal: false }; }
    if (matches(profile, profile.after, this.logical)) base = outdent(this.logicalIndent, unit);
    if (this.singleIndent !== null) { base = this.singleIndent; this.singleIndent = null; }
    return { indent: base, split: "", literal: false };
  }
}
</ui:copy>

## Propose one browser edit and its caret

Resolve named Markdown fences and Hickory source bodies without treating prose as
code. Preserve CRLF and selected source ranges. Between an opening and its existing
closer, insert an indented body line and a separate aligned closer; consume only
the closer’s preceding whitespace. The caller applies the proposal through its
existing protected-source and native undo path.

<ui:copy id="simple-enter-ts-proposal">
// Return one ordinary source replacement and a caret within it. No save or format request.
export function smartEnter(source: string, from: number, to: number, path: string, unit: string, mode: EnterMode): EnterPlan {
  const start = from === 0 ? 0 : source.lastIndexOf("\n", from - 1) + 1;
  const next = source.indexOf("\n", from);
  const eol = (next >= 0 ? next > 0 && source[next - 1] === "\r" : start >= 2 && source[start - 2] === "\r") ? "\r\n" : "\n";
  const prefix = source.slice(start, from).match(/^[ \t]*/)![0];
  const fallback = (indent: string): EnterPlan => ({ from, to, text: eol + indent, caret: from + eol.length + indent.length });
  if (mode === "none") return fallback("");
  if (mode === "copy" || from > registry.maxContext || from - start > registry.maxLine) return fallback(prefix);
  const rows = source.slice(0, from).split("\n");
  let profile = profileFor(path), scanner = new EnterScanner(), fence = "", payload = "";
  const markdown = /\.(md|markdown)$/i.test(path);
  if (!markdown && !profile) return fallback(prefix);
  let result = { indent: prefix, split: "", literal: true };
  for (let line = 0; line < rows.length; line++) {
    const row = rows[line].replace(/\r$/, ""), trimmed = row.trim();
    if (row.length > registry.maxLine) return fallback(prefix);
    if (markdown) {
      const marker = trimmed.match(/^(`{3,}|~{3,})([^ ]*)/);
      if (fence) {
        if (trimmed.startsWith(fence) && /^(`+|~+)\s*$/.test(trimmed)) { fence = ""; profile = undefined; scanner = new EnterScanner(); continue; }
      } else if (!payload && marker) {
        fence = marker[1]; profile = profileFor(marker[2]); scanner = new EnterScanner(); continue;
      } else if (!fence) {
        const open = trimmed.match(/^<[\w.-]+:(file|copy|cut|exec)\b[^>]*>/);
        if (open) {
          payload = open[1]; const hint = open[0].match(/\b(?:path|language|lang)\s*=\s*["']([^"']+)["']/);
          profile = hint ? profileFor(hint[1]) : undefined; scanner = new EnterScanner(); continue;
        }
        if (payload && trimmed.startsWith("</") && trimmed.includes(`:${payload}>`)) { payload = ""; profile = undefined; scanner = new EnterScanner(); continue; }
      }
    }
    if (profile) result = scanner.line(row, profile, line, unit);
  }
  if (!profile || result.literal) return fallback(prefix);
  const suffix = source.slice(to, source.indexOf("\n", to) < 0 ? source.length : source.indexOf("\n", to)).replace(/\r$/, "");
  const whitespace = suffix.match(/^[ \t]*/)![0].length;
  const top = scanner.brackets.at(-1);
  if (result.split && (profile.foldCase ? suffix.trimStart().toLowerCase() : suffix.trimStart()).startsWith(result.split) && top) {
    const openingHere = top.line === rows.length - 1;
    const text = openingHere ? eol + result.indent + eol + top.indent : eol + top.indent;
    return { from, to: to + whitespace, text, caret: from + eol.length + (openingHere ? result.indent.length : top.indent.length) };
  }
  return fallback(result.indent);
}
</ui:copy>

<ui:file path="apps/web/src/simple/SmartEnter.ts"><ui:paste select="#simple-enter-ts-policy"/><ui:paste select="#simple-enter-ts-scanner"/><ui:paste select="#simple-enter-ts-proposal"/></ui:file>

## Load the Apple policy independently of editor capabilities

Decode the shared rule set and compile patterns lazily. Missing native resources
fall back to prefix copying. The returned edit uses UTF-16 coordinates, matching
the platform editor and the common fixtures.

<ui:copy id="simple-enter-swift-policy">
import Foundation

struct SimpleEnterPlan { var range: NSRange; var text: String; var caret: Int }
private struct EnterDefinition: Decodable {
    var id: String; var names: [String]; var comments: [String]; var blocks: [[String]]
    var multiline: [String]; var quotes: String; var open: String; var close: String
    var middle: String; var next: String; var regexLiterals: Bool; var after: String; var literal: String; var tags: Bool
    var voidTags: [String]; var foldCase: Bool; var offside: Bool
}
private final class EnterProfile {
    let rule: EnterDefinition
    private var patterns: [String: NSRegularExpression] = [:]
    init(_ rule: EnterDefinition) { self.rule = rule }
    func matches(_ pattern: String, _ text: String) -> Bool {
        guard !pattern.isEmpty else { return false }
        if patterns[pattern] == nil { patterns[pattern] = try? NSRegularExpression(pattern: pattern, options: rule.foldCase ? .caseInsensitive : []) }
        return patterns[pattern]?.firstMatch(in: text, range: NSRange(location: 0, length: text.utf16.count)) != nil
    }
}
private final class EnterResourceMarker: NSObject {}
final class SimpleEnterRules {
    let maxContext: Int; let maxLine: Int
    private let profiles: [EnterProfile]
    init(data: Data) throws {
        struct Registry: Decodable { var maxContext: Int; var maxLine: Int; var profiles: [EnterDefinition] }
        let registry = try JSONDecoder().decode(Registry.self, from: data)
        guard registry.profiles.allSatisfy({ rule in rule.blocks.allSatisfy { $0.count == 2 && $0.allSatisfy { !$0.isEmpty } } && rule.comments.allSatisfy { !$0.isEmpty } && rule.multiline.allSatisfy { !$0.isEmpty } }) else {
            throw NSError(domain: "SmartEnterRules", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid indentation delimiters; rebuild the bundled Enter rules."])
        }
        maxContext = registry.maxContext; maxLine = registry.maxLine; profiles = registry.profiles.map(EnterProfile.init)
    }
    fileprivate func profile(_ path: String) -> EnterProfile? {
        let name = (path.replacingOccurrences(of: "\\", with: "/") as NSString).lastPathComponent.lowercased()
        let ext = (name as NSString).pathExtension
        return profiles.first { $0.rule.names.contains(ext.isEmpty ? name : ext) }
    }
    static let shared: SimpleEnterRules? = {
        // Both SwiftPM and Xcode bundle layouts, with a readable-text fallback if absent.
        let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, Bundle(for: EnterResourceMarker.self).bundleURL.deletingLastPathComponent(), URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
        let bundles = [Bundle.main] + roots.compactMap { Bundle(url: $0.appendingPathComponent("HickorySimple_HickorySimple.bundle")) }
        for bundle in bundles {
            if let url = bundle.url(forResource: "enter-rules", withExtension: "json", subdirectory: "HickoryAssets/Resources") ?? bundle.url(forResource: "enter-rules", withExtension: "json", subdirectory: "Resources") ?? bundle.url(forResource: "enter-rules", withExtension: "json"),
               let data = try? Data(contentsOf: url), let rules = try? SimpleEnterRules(data: data) { return rules }
        }
        return nil
    }()
}
private func enterOutdent(_ prefix: String, _ unit: String) -> String {
    var result = prefix
    if result.hasSuffix("\t") { result.removeLast(); return result }
    var count = 0
    while count < unit.count && result.hasSuffix(" ") { result.removeLast(); count += 1 }
    return result
}
private let enterExpressionCache: NSCache<NSString, NSRegularExpression> = { let cache = NSCache<NSString, NSRegularExpression>(); cache.countLimit = 32; return cache }()
private func enterExpression(_ pattern: String) -> NSRegularExpression? {
    if let value = enterExpressionCache.object(forKey: pattern as NSString) { return value }
    guard let value = try? NSRegularExpression(pattern: pattern) else { return nil }
    enterExpressionCache.setObject(value, forKey: pattern as NSString); return value
}
private func enterCapture(_ pattern: String, _ text: String) -> [String]? {
    guard let regex = enterExpression(pattern), let match = regex.firstMatch(in: text, range: NSRange(location: 0, length: text.utf16.count)) else { return nil }
    let source = text as NSString
    return (0..<match.numberOfRanges).map { match.range(at: $0).location == NSNotFound ? "" : source.substring(with: match.range(at: $0)) }
}
</ui:copy>

## Keep Apple lexical state across the bounded prefix

Hide strings and comments before counting brackets. Remember each opening
indentation and unfinished logical statement, so an inner closing delimiter does
not drift the following line right and `return ( … )` dedents only at completion.
Literal YAML bodies and unfinished strings retain their exact prefix.

<ui:copy id="simple-enter-swift-scanner">
private struct EnterOpening { var token: String; var indent: String; var line: Int }
private struct EnterResult { var indent: String; var split = ""; var literal = false }
private final class EnterScanner {
    var block = "", quoted = false, brackets: [EnterOpening] = [], keywords: [String] = []
    var logical = "", logicalIndent = "", literal: String?, singleIndent: String?
    func clean(_ row: String, _ profile: EnterProfile) -> String {
        let text = row as NSString, rule = profile.rule
        var code = "", index = 0; quoted = false
        func has(_ mark: String) -> Bool { let size = mark.utf16.count; return index + size <= text.length && text.substring(with: NSRange(location: index, length: size)) == mark }
        while index < text.length {
            if !block.isEmpty {
                if has(block) && (index == 0 || text.character(at: index - 1) != 92) { index += block.utf16.count; block = "" } else { index += 1 }
                code += " "; continue
            }
            let pair = rule.blocks.first { has($0[0]) }, multi = rule.multiline.first { has($0) }
            if let pair { block = pair[1]; index += pair[0].utf16.count; code += " "; continue }
            if let multi { block = multi; index += multi.utf16.count; code += " "; continue }
            if rule.comments.contains(where: has) { break }
            if rule.regexLiterals, text.character(at: index) == 47, enterCapture(#"(?:^|\b(?:return|throw|yield|case)|[=(:,!&|?;{}\[>])\s*$"#, code) != nil {
                index += 1; var characterClass = false, closed = false
                while index < text.length {
                    let next = text.character(at: index); index += 1
                    if next == 92 { index = min(text.length, index + 1); continue }
                    if next == 91 { characterClass = true }; if next == 93 { characterClass = false }
                    if next == 47 && !characterClass { closed = true; break }
                }
                if !closed { quoted = true; block = "/" }; code += "~"; continue
            }
            let char = text.substring(with: NSRange(location: index, length: 1)); index += 1
            if rule.quotes.contains(char) {
                if char == "'", rule.id == "rust", !text.substring(from: index).contains("'") { code += char; continue }
                var closed = false
                while index < text.length {
                    let next = text.substring(with: NSRange(location: index, length: 1)); index += 1
                    if next == "\\" { index = min(text.length, index + 1); continue }
                    if next == char { closed = true; break }
                }
                if !closed { quoted = true; block = char }; code += "~"; continue
            }
            code += char
        }
        return code.trimmingCharacters(in: .whitespaces)
    }
    func line(_ row: String, _ profile: EnterProfile, _ number: Int, _ unit: String) -> EnterResult {
        let prefix = String(row.prefix { $0 == " " || $0 == "\t" }), rule = profile.rule
        if let literal, row.trimmingCharacters(in: .whitespaces).isEmpty || prefix.count > literal.count { return EnterResult(indent: prefix, literal: true) }
        literal = nil
        let wasBlock = !block.isEmpty, code = clean(row, profile), startDepth = brackets.count
        if wasBlock || !block.isEmpty || quoted || code.isEmpty { return EnterResult(indent: prefix, literal: true) }
        if startDepth == 0 { logical = code; logicalIndent = prefix }
        var base = prefix
        if profile.matches(rule.close, code) { base = keywords.popLast() ?? enterOutdent(prefix, unit) }
        let middle = profile.matches(rule.middle, code)
        if middle, !rule.offside, let parent = keywords.last { base = parent }
        if rule.tags {
            let regex = enterExpression(#"<(/?)([A-Za-z][\w:.-]*)\b[^>]*>"#)!, text = code as NSString
            for match in regex.matches(in: code, range: NSRange(location: 0, length: text.length)) {
                let raw = text.substring(with: match.range), name = text.substring(with: match.range(at: 2))
                let tag = rule.foldCase ? name.lowercased() : name
                if match.range(at: 1).length > 0 { if brackets.last?.token == "</" + tag + ">" { base = brackets.removeLast().indent } }
                else if !raw.hasSuffix("/>") && !rule.voidTags.contains(tag) { brackets.append(EnterOpening(token: "</" + tag + ">", indent: base, line: number)) }
            }
        } else {
            for char in code {
                if "{[(".contains(char) { brackets.append(EnterOpening(token: String(char), indent: base, line: number)) }
                if let index = Array("}])").firstIndex(of: char), brackets.last?.token == String(Array("{[(")[index]) { base = brackets.removeLast().indent }
            }
        }
        if let top = brackets.last {
            let split = top.token.hasPrefix("</") ? top.token : String(Array("}])")[Array("{[(").firstIndex(of: Character(top.token))!])
            return EnterResult(indent: top.line == number || base.count <= top.indent.count ? top.indent + unit : base, split: split)
        }
        if profile.matches(rule.next, code) {
            if singleIndent == nil { singleIndent = base }; return EnterResult(indent: base + unit)
        }
        if profile.matches(rule.open, code) || middle {
            if !middle { keywords.append(base) }; return EnterResult(indent: base + unit)
        }
        if profile.matches(rule.literal, code) { literal = base; return EnterResult(indent: base + unit) }
        if profile.matches(rule.after, logical) { base = enterOutdent(logicalIndent, unit) }
        if let singleIndent { base = singleIndent; self.singleIndent = nil }
        return EnterResult(indent: base)
    }
}
</ui:copy>

## Propose one Apple edit and its caret

Resolve named Markdown fences and Hickory source bodies without treating prose as
code. Preserve CRLF and selected source ranges. Between an opening and its existing
closer, insert an indented body line and a separate aligned closer; consume only
the closer’s preceding whitespace. The caller applies the proposal through its
existing protected-source and native undo path.

<ui:copy id="simple-enter-swift-proposal">
func smartEnter(_ source: String, range: NSRange, path: String, unit: String, mode: String, rules: SimpleEnterRules? = .shared) -> SimpleEnterPlan {
    let text = source as NSString, currentLine = (source as NSString).lineRange(for: NSRange(location: range.location, length: 0))
    let row = text.substring(with: NSRange(location: currentLine.location, length: range.location - currentLine.location)), prefix = String(row.prefix { $0 == " " || $0 == "\t" })
    let eol = simpleNewlineText(source, at: range.location).hasPrefix("\r\n") ? "\r\n" : "\n"
    func fallback(_ indent: String) -> SimpleEnterPlan { SimpleEnterPlan(range: range, text: eol + indent, caret: range.location + eol.utf16.count + indent.utf16.count) }
    if mode == "none" { return fallback("") }
    guard mode != "copy", let rules, range.location <= rules.maxContext, row.utf16.count <= rules.maxLine else { return fallback(prefix) }
    let rows = text.substring(to: range.location).components(separatedBy: "\n"), markdown = ["md", "markdown"].contains((path as NSString).pathExtension.lowercased())
    var profile = rules.profile(path), scanner = EnterScanner(), fence = "", payload = "", result = EnterResult(indent: prefix, literal: true)
    if !markdown && profile == nil { return fallback(prefix) }
    for (line, raw) in rows.enumerated() {
        let row = raw.hasSuffix("\r") ? String(raw.dropLast()) : raw, trimmed = row.trimmingCharacters(in: .whitespaces)
        if row.utf16.count > rules.maxLine { return fallback(prefix) }
        if markdown {
            let marker = enterCapture(#"^(`{3,}|~{3,})([^ ]*)"#, trimmed)
            if !fence.isEmpty {
                if trimmed.hasPrefix(fence), enterCapture(#"^(`+|~+)\s*$"#, trimmed) != nil { fence = ""; profile = nil; scanner = EnterScanner(); continue }
            } else if payload.isEmpty, let marker { fence = marker[1]; profile = rules.profile(marker[2]); scanner = EnterScanner(); continue }
            else {
                if let open = enterCapture(#"^<[\w.-]+:(file|copy|cut|exec)\b[^>]*>"#, trimmed) {
                    payload = open[1]; let hint = enterCapture(#"\b(?:path|language|lang)\s*=\s*["']([^"']+)["']"#, open[0])
                    profile = hint.flatMap { rules.profile($0[1]) }; scanner = EnterScanner(); continue
                }
                if !payload.isEmpty, trimmed.hasPrefix("</"), trimmed.contains(":" + payload + ">") { payload = ""; profile = nil; scanner = EnterScanner(); continue }
            }
        }
        if let profile { result = scanner.line(row, profile, line, unit) }
    }
    guard profile != nil, !result.literal else { return fallback(prefix) }
    let suffixRange = text.lineRange(for: NSRange(location: NSMaxRange(range), length: 0))
    let suffix = text.substring(with: NSRange(location: NSMaxRange(range), length: NSMaxRange(suffixRange) - NSMaxRange(range))).trimmingCharacters(in: .newlines)
    let whitespace = suffix.prefix { $0 == " " || $0 == "\t" }.utf16.count
    if !result.split.isEmpty, (profile?.rule.foldCase == true ? String(suffix.dropFirst(whitespace)).lowercased() : String(suffix.dropFirst(whitespace))).hasPrefix(result.split), let top = scanner.brackets.last {
        let openingHere = top.line == rows.count - 1
        let inserted = openingHere ? eol + result.indent + eol + top.indent : eol + top.indent
        return SimpleEnterPlan(range: NSRange(location: range.location, length: range.length + whitespace), text: inserted, caret: range.location + eol.utf16.count + (openingHere ? result.indent.utf16.count : top.indent.utf16.count))
    }
    return fallback(result.indent)
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SmartEnter.swift"><ui:paste select="#simple-enter-swift-policy"/><ui:paste select="#simple-enter-swift-scanner"/><ui:paste select="#simple-enter-swift-proposal"/></ui:file>

## Load the Android policy independently of editor capabilities

Decode the shared rule set and compile patterns lazily. Missing native resources
fall back to prefix copying. The returned edit uses UTF-16 coordinates, matching
the platform editor and the common fixtures.

<ui:copy id="simple-enter-kt-policy">
package com.hickorydocs.simple

import org.json.JSONObject

data class SimpleEnterPlan(val from: Int, val to: Int, val text: String, val caret: Int)
internal class EnterProfile(val json: JSONObject) {
    val id = json.getString("id")
    fun strings(key: String) = json.getJSONArray(key).let { values -> (0 until values.length()).map { values.getString(it) } }
    val names = strings("names"); val comments = strings("comments"); val multiline = strings("multiline")
    val blocks = json.getJSONArray("blocks").let { values -> (0 until values.length()).map { values.getJSONArray(it).let { pair -> require(pair.length() == 2 && pair.getString(0).isNotEmpty() && pair.getString(1).isNotEmpty()); pair.getString(0) to pair.getString(1) } } }
    val quotes = json.getString("quotes"); val tags = json.getBoolean("tags"); val voidTags = strings("voidTags")
    val regexLiterals = json.getBoolean("regexLiterals")
    val foldCase = json.getBoolean("foldCase"); val offside = json.getBoolean("offside")
    private val patterns = mutableMapOf<String, Regex>()
    fun matches(key: String, text: String): Boolean {
        val pattern = json.getString(key)
        if (pattern.isEmpty()) return false
        return patterns.getOrPut(key) { Regex(pattern, if (foldCase) setOf(RegexOption.IGNORE_CASE) else emptySet()) }.containsMatchIn(text)
    }
}
class SimpleEnterRules(data: String) {
    private val registry = JSONObject(data)
    val maxContext = registry.getInt("maxContext"); val maxLine = registry.getInt("maxLine")
    private val profiles = registry.getJSONArray("profiles").let { values -> (0 until values.length()).map { EnterProfile(values.getJSONObject(it)) } }
    internal fun profile(path: String): EnterProfile? {
        val name = path.replace('\\', '/').substringAfterLast('/').lowercase()
        val ext = name.substringAfterLast('.')
        return profiles.firstOrNull { ext in it.names }
    }
}
private fun enterOutdent(prefix: String, unit: String): String {
    if (prefix.endsWith('\t')) return prefix.dropLast(1)
    var result = prefix; var count = 0
    while (count < unit.length && result.endsWith(' ')) { result = result.dropLast(1); count++ }
    return result
}
</ui:copy>

## Keep Android lexical state across the bounded prefix

Hide strings and comments before counting brackets. Remember each opening
indentation and unfinished logical statement, so an inner closing delimiter does
not drift the following line right and `return ( … )` dedents only at completion.
Literal YAML bodies and unfinished strings retain their exact prefix.

<ui:copy id="simple-enter-kt-scanner">
private val enterExpressionCache = java.util.concurrent.ConcurrentHashMap<String, Regex>()
private fun enterExpression(pattern: String): Regex = enterExpressionCache.computeIfAbsent(pattern) { Regex(it) }
private data class EnterOpening(val token: String, val indent: String, val line: Int)
private data class EnterResult(val indent: String, val split: String = "", val literal: Boolean = false)
private class EnterScanner {
    var block = ""; var quoted = false
    val brackets = mutableListOf<EnterOpening>(); private val keywords = mutableListOf<String>()
    private var logical = ""; private var logicalIndent = ""; private var literal: String? = null; private var singleIndent: String? = null
    fun clean(row: String, profile: EnterProfile): String {
        val code = StringBuilder(); var index = 0; quoted = false
        while (index < row.length) {
            if (block.isNotEmpty()) {
                if (row.startsWith(block, index) && (index == 0 || row[index - 1] != '\\')) { index += block.length; block = "" } else index++
                code.append(' '); continue
            }
            val pair = profile.blocks.firstOrNull { row.startsWith(it.first, index) }
            val multi = profile.multiline.firstOrNull { row.startsWith(it, index) }
            if (pair != null || multi != null) { block = pair?.second ?: multi!!; index += pair?.first?.length ?: multi!!.length; code.append(' '); continue }
            if (profile.comments.any { row.startsWith(it, index) }) break
            if (profile.regexLiterals && row[index] == '/' && enterExpression("(?:^|\\b(?:return|throw|yield|case)|[=(:,!&|?;{}\\[>])\\s*$").containsMatchIn(code)) {
                index++; var characterClass = false; var closed = false
                while (index < row.length) {
                    val next = row[index++]
                    if (next == '\\') { index = minOf(row.length, index + 1); continue }
                    if (next == '[') characterClass = true
                    if (next == ']') characterClass = false
                    if (next == '/' && !characterClass) { closed = true; break }
                }
                if (!closed) { quoted = true; block = "/" }
                code.append('~'); continue
            }
            val char = row[index++]
            if (char in profile.quotes) {
                if (char == '\'' && profile.id == "rust" && !row.substring(index).contains('\'')) { code.append(char); continue }
                var closed = false
                while (index < row.length) {
                    val next = row[index++]
                    if (next == '\\') { index = minOf(row.length, index + 1); continue }
                    if (next == char) { closed = true; break }
                }
                if (!closed) { quoted = true; block = char.toString() }
                code.append('~'); continue
            }
            code.append(char)
        }
        return code.toString().trim()
    }
    fun line(row: String, profile: EnterProfile, number: Int, unit: String): EnterResult {
        val prefix = row.takeWhile { it == ' ' || it == '\t' }
        literal?.let { if (row.isBlank() || prefix.length > it.length) return EnterResult(prefix, literal = true) }
        literal = null
        val wasBlock = block.isNotEmpty(); val code = clean(row, profile); val startDepth = brackets.size
        if (wasBlock || block.isNotEmpty() || quoted || code.isEmpty()) return EnterResult(prefix, literal = true)
        if (startDepth == 0) { logical = code; logicalIndent = prefix }
        var base = prefix
        if (profile.matches("close", code)) base = keywords.removeLastOrNull() ?: enterOutdent(prefix, unit)
        val middle = profile.matches("middle", code)
        if (middle && !profile.offside && keywords.isNotEmpty()) base = keywords.last()
        if (profile.tags) {
            for (match in enterExpression("<(\\/?)([A-Za-z][\\w:.-]*)\\b[^>]*>").findAll(code)) {
                val tag = if (profile.foldCase) match.groupValues[2].lowercase() else match.groupValues[2]
                if (match.groupValues[1].isNotEmpty()) { if (brackets.lastOrNull()?.token == "</$tag>") base = brackets.removeLast().indent }
                else if (!match.value.endsWith("/>") && tag !in profile.voidTags) brackets.add(EnterOpening("</$tag>", base, number))
            }
        } else for (char in code) {
            if (char in "{[(") brackets.add(EnterOpening(char.toString(), base, number))
            val opening = "}])".indexOf(char)
            if (opening >= 0 && brackets.lastOrNull()?.token == "{[("[opening].toString()) base = brackets.removeLast().indent
        }
        brackets.lastOrNull()?.let { top ->
            val split = if (top.token.startsWith("</")) top.token else "}])"["{[(".indexOf(top.token)].toString()
            return EnterResult(if (top.line == number || base.length <= top.indent.length) top.indent + unit else base, split)
        }
        if (profile.matches("next", code)) {
            if (singleIndent == null) singleIndent = base
            return EnterResult(base + unit)
        }
        if (profile.matches("open", code) || middle) {
            if (!middle) keywords.add(base)
            return EnterResult(base + unit)
        }
        if (profile.matches("literal", code)) { literal = base; return EnterResult(base + unit) }
        if (profile.matches("after", logical)) base = enterOutdent(logicalIndent, unit)
        singleIndent?.let { base = it; singleIndent = null }
        return EnterResult(base)
    }
}
</ui:copy>

## Propose one Android edit and its caret

Resolve named Markdown fences and Hickory source bodies without treating prose as
code. Preserve CRLF and selected source ranges. Between an opening and its existing
closer, insert an indented body line and a separate aligned closer; consume only
the closer’s preceding whitespace. The caller applies the proposal through its
existing protected-source and native undo path.

<ui:copy id="simple-enter-kt-proposal">
fun smartEnter(source: String, from: Int, to: Int, path: String, unit: String, mode: String, rules: SimpleEnterRules?): SimpleEnterPlan {
    val start = if (from == 0) 0 else source.lastIndexOf('\n', from - 1) + 1
    val prefix = source.substring(start, from).takeWhile { it == ' ' || it == '\t' }
    val eol = if (simpleNewlineText(source, from).startsWith("\r\n")) "\r\n" else "\n"
    fun fallback(indent: String) = SimpleEnterPlan(from, to, eol + indent, from + eol.length + indent.length)
    if (mode == "none") return fallback("")
    if (mode == "copy" || rules == null || from > rules.maxContext || from - start > rules.maxLine) return fallback(prefix)
    val rows = source.substring(0, from).split('\n'); val markdown = path.substringAfterLast('.', "").lowercase() in listOf("md", "markdown")
    var profile = rules.profile(path); var scanner = EnterScanner(); var fence = ""; var payload = ""
    var result = EnterResult(prefix, literal = true)
    if (!markdown && profile == null) return fallback(prefix)
    for ((line, raw) in rows.withIndex()) {
        val row = raw.removeSuffix("\r"); val trimmed = row.trim()
        if (row.length > rules.maxLine) return fallback(prefix)
        if (markdown) {
            val marker = enterExpression("^(`{3,}|~{3,})([^ ]*)").find(trimmed)
            if (fence.isNotEmpty()) {
                if (trimmed.startsWith(fence) && enterExpression("^(`+|~+)\\s*$").containsMatchIn(trimmed)) { fence = ""; profile = null; scanner = EnterScanner(); continue }
            } else if (payload.isEmpty() && marker != null) { fence = marker.groupValues[1]; profile = rules.profile(marker.groupValues[2]); scanner = EnterScanner(); continue }
            else {
                val open = enterExpression("^<[\\w.-]+:(file|copy|cut|exec)\\b[^>]*>").find(trimmed)
                if (open != null) {
                    payload = open.groupValues[1]; val hint = enterExpression("\\b(?:path|language|lang)\\s*=\\s*[\"']([^\"']+)[\"']").find(open.value)
                    profile = hint?.let { rules.profile(it.groupValues[1]) }; scanner = EnterScanner(); continue
                }
                if (payload.isNotEmpty() && trimmed.startsWith("</") && trimmed.contains(":$payload>")) { payload = ""; profile = null; scanner = EnterScanner(); continue }
            }
        }
        profile?.let { result = scanner.line(row, it, line, unit) }
    }
    if (profile == null || result.literal) return fallback(prefix)
    val end = source.indexOf('\n', to).let { if (it < 0) source.length else it }
    val suffix = source.substring(to, end).removeSuffix("\r"); val whitespace = suffix.takeWhile { it == ' ' || it == '\t' }.length
    val top = scanner.brackets.lastOrNull()
    if (result.split.isNotEmpty() && (if (profile.foldCase) suffix.trimStart().lowercase() else suffix.trimStart()).startsWith(result.split) && top != null) {
        val openingHere = top.line == rows.size - 1
        val inserted = if (openingHere) eol + result.indent + eol + top.indent else eol + top.indent
        return SimpleEnterPlan(from, to + whitespace, inserted, from + eol.length + if (openingHere) result.indent.length else top.indent.length)
    }
    return fallback(result.indent)
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SmartEnter.kt"><ui:paste select="#simple-enter-kt-policy"/><ui:paste select="#simple-enter-kt-scanner"/><ui:paste select="#simple-enter-kt-proposal"/></ui:file>

## Exercise the same source/caret outcomes on every host

Each fixture marks the caret with `¦`. Tests remove that marker, apply the
proposed replacement, and compare both complete source and resulting selection.
This covers language families, dedents, closer splitting, comments, strings,
Unicode, line endings, modes and mixed documents. Separate focused checks cover
scan bounds and actual native/DOM Enter and undo; fixture equality alone is not
physical keyboard or IME acceptance.

<ui:copy id="simple-enter-fixtures">
{
  "cases": [
    {
      "name": "python suite",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f():¦",
      "after": "def f():\n    ¦"
    },
    {
      "name": "python async suite",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "async def f():¦",
      "after": "async def f():\n    ¦"
    },
    {
      "name": "python multiline suite",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f(\n    value\n):¦",
      "after": "def f(\n    value\n):\n    ¦"
    },
    {
      "name": "python return",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f():\n    return 1¦",
      "after": "def f():\n    return 1\n¦"
    },
    {
      "name": "python nested return",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f():\n    if ok:\n        return 1¦",
      "after": "def f():\n    if ok:\n        return 1\n    ¦"
    },
    {
      "name": "python pass",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "if ok:\n    pass¦",
      "after": "if ok:\n    pass\n¦"
    },
    {
      "name": "python break",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "if ok:\n    break¦",
      "after": "if ok:\n    break\n¦"
    },
    {
      "name": "python continue",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "if ok:\n    continue¦",
      "after": "if ok:\n    continue\n¦"
    },
    {
      "name": "python raise ValueError()",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "if ok:\n    raise ValueError()¦",
      "after": "if ok:\n    raise ValueError()\n¦"
    },
    {
      "name": "python pending return expression",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f():\n    return call(¦",
      "after": "def f():\n    return call(\n        ¦"
    },
    {
      "name": "python completed return expression",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f():\n    return call(\n        1\n    )¦",
      "after": "def f():\n    return call(\n        1\n    )\n¦"
    },
    {
      "name": "python colon in string",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "text = \":\"¦",
      "after": "text = \":\"\n¦"
    },
    {
      "name": "python comment colon",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "    # if fake:¦",
      "after": "    # if fake:\n    ¦"
    },
    {
      "name": "python trailing comment",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "if ok: # comment¦",
      "after": "if ok: # comment\n    ¦"
    },
    {
      "name": "python triple quoted content",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "text = \"\"\"\nif fake:¦",
      "after": "text = \"\"\"\nif fake:\n¦"
    },
    {
      "name": "python unknown unfinished string",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "text = \"fake:¦",
      "after": "text = \"fake:\n¦"
    },
    {
      "name": "python two spaces",
      "path": "sample.py",
      "mode": "smart",
      "unit": "  ",
      "before": "if ok:\n  if more:¦",
      "after": "if ok:\n  if more:\n    ¦"
    },
    {
      "name": "tabs",
      "path": "sample.py",
      "mode": "smart",
      "unit": "\t",
      "before": "def f():\n\tif ok:¦",
      "after": "def f():\n\tif ok:\n\t\t¦"
    },
    {
      "name": "crlf",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def f():\r\n    return 1¦",
      "after": "def f():\r\n    return 1\r\n¦"
    },
    {
      "name": "js braces",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "function f() {¦",
      "after": "function f() {\n    ¦"
    },
    {
      "name": "ts braces",
      "path": "sample.ts",
      "mode": "smart",
      "unit": "    ",
      "before": "function f(): void {¦",
      "after": "function f(): void {\n    ¦"
    },
    {
      "name": "java braces",
      "path": "sample.java",
      "mode": "smart",
      "unit": "    ",
      "before": "void run() {¦",
      "after": "void run() {\n    ¦"
    },
    {
      "name": "cs braces",
      "path": "sample.cs",
      "mode": "smart",
      "unit": "    ",
      "before": "void Run() {¦",
      "after": "void Run() {\n    ¦"
    },
    {
      "name": "cpp braces",
      "path": "sample.cpp",
      "mode": "smart",
      "unit": "    ",
      "before": "int main() {¦",
      "after": "int main() {\n    ¦"
    },
    {
      "name": "c braces",
      "path": "sample.c",
      "mode": "smart",
      "unit": "    ",
      "before": "int main() {¦",
      "after": "int main() {\n    ¦"
    },
    {
      "name": "rs braces",
      "path": "sample.rs",
      "mode": "smart",
      "unit": "    ",
      "before": "fn f<'a>() {¦",
      "after": "fn f<'a>() {\n    ¦"
    },
    {
      "name": "go braces",
      "path": "sample.go",
      "mode": "smart",
      "unit": "    ",
      "before": "func f() {¦",
      "after": "func f() {\n    ¦"
    },
    {
      "name": "kt braces",
      "path": "sample.kt",
      "mode": "smart",
      "unit": "    ",
      "before": "fun f() {¦",
      "after": "fun f() {\n    ¦"
    },
    {
      "name": "swift braces",
      "path": "sample.swift",
      "mode": "smart",
      "unit": "    ",
      "before": "func f() {¦",
      "after": "func f() {\n    ¦"
    },
    {
      "name": "dart braces",
      "path": "sample.dart",
      "mode": "smart",
      "unit": "    ",
      "before": "void main() {¦",
      "after": "void main() {\n    ¦"
    },
    {
      "name": "php braces",
      "path": "sample.php",
      "mode": "smart",
      "unit": "    ",
      "before": "function run() {¦",
      "after": "function run() {\n    ¦"
    },
    {
      "name": "r braces",
      "path": "sample.r",
      "mode": "smart",
      "unit": "    ",
      "before": "f <- function() {¦",
      "after": "f <- function() {\n    ¦"
    },
    {
      "name": "zig braces",
      "path": "sample.zig",
      "mode": "smart",
      "unit": "    ",
      "before": "pub fn main() void {¦",
      "after": "pub fn main() void {\n    ¦"
    },
    {
      "name": "scala braces",
      "path": "sample.scala",
      "mode": "smart",
      "unit": "    ",
      "before": "def main() {¦",
      "after": "def main() {\n    ¦"
    },
    {
      "name": "groovy braces",
      "path": "sample.groovy",
      "mode": "smart",
      "unit": "    ",
      "before": "def run() {¦",
      "after": "def run() {\n    ¦"
    },
    {
      "name": "ps1 braces",
      "path": "sample.ps1",
      "mode": "smart",
      "unit": "    ",
      "before": "function Run {¦",
      "after": "function Run {\n    ¦"
    },
    {
      "name": "existing brace split",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "function f() {¦}",
      "after": "function f() {\n    ¦\n}"
    },
    {
      "name": "existing spaced brace split",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "function f() {¦   }",
      "after": "function f() {\n    ¦\n}"
    },
    {
      "name": "javascript return stays in brace",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "function f() {\n    return 1;¦",
      "after": "function f() {\n    return 1;\n    ¦"
    },
    {
      "name": "closing brace",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "function f() {\n    work();\n    }¦",
      "after": "function f() {\n    work();\n    }\n¦"
    },
    {
      "name": "braces inside string",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "const value = \"{\";¦",
      "after": "const value = \"{\";\n¦"
    },
    {
      "name": "comment brace",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "// {¦",
      "after": "// {\n¦"
    },
    {
      "name": "template literal brace",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "const text = `\n{¦",
      "after": "const text = `\n{\n¦"
    },
    {
      "name": "nested expression closing",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    ",
      "before": "call(\n    other(\n        value\n    )¦",
      "after": "call(\n    other(\n        value\n    )\n    ¦"
    },
    {
      "name": "json pair",
      "path": "sample.json",
      "mode": "smart",
      "unit": "    ",
      "before": "{\"values\": [¦]}",
      "after": "{\"values\": [\n    ¦\n]}"
    },
    {
      "name": "rb block",
      "path": "sample.rb",
      "mode": "smart",
      "unit": "    ",
      "before": "def f¦",
      "after": "def f\n    ¦"
    },
    {
      "name": "rb closes",
      "path": "sample.rb",
      "mode": "smart",
      "unit": "    ",
      "before": "def f\n    value\n    end¦",
      "after": "def f\n    value\n    end\n¦"
    },
    {
      "name": "lua block",
      "path": "sample.lua",
      "mode": "smart",
      "unit": "    ",
      "before": "function f()¦",
      "after": "function f()\n    ¦"
    },
    {
      "name": "lua closes",
      "path": "sample.lua",
      "mode": "smart",
      "unit": "    ",
      "before": "function f()\n    value\n    end¦",
      "after": "function f()\n    value\n    end\n¦"
    },
    {
      "name": "sh block",
      "path": "sample.sh",
      "mode": "smart",
      "unit": "    ",
      "before": "if true; then¦",
      "after": "if true; then\n    ¦"
    },
    {
      "name": "sh closes",
      "path": "sample.sh",
      "mode": "smart",
      "unit": "    ",
      "before": "if true; then\n    value\n    fi¦",
      "after": "if true; then\n    value\n    fi\n¦"
    },
    {
      "name": "ex block",
      "path": "sample.ex",
      "mode": "smart",
      "unit": "    ",
      "before": "def run do¦",
      "after": "def run do\n    ¦"
    },
    {
      "name": "ex closes",
      "path": "sample.ex",
      "mode": "smart",
      "unit": "    ",
      "before": "def run do\n    value\n    end¦",
      "after": "def run do\n    value\n    end\n¦"
    },
    {
      "name": "jl block",
      "path": "sample.jl",
      "mode": "smart",
      "unit": "    ",
      "before": "function run()¦",
      "after": "function run()\n    ¦"
    },
    {
      "name": "jl closes",
      "path": "sample.jl",
      "mode": "smart",
      "unit": "    ",
      "before": "function run()\n    value\n    end¦",
      "after": "function run()\n    value\n    end\n¦"
    },
    {
      "name": "sql block",
      "path": "sample.sql",
      "mode": "smart",
      "unit": "    ",
      "before": "BEGIN¦",
      "after": "BEGIN\n    ¦"
    },
    {
      "name": "sql closes",
      "path": "sample.sql",
      "mode": "smart",
      "unit": "    ",
      "before": "BEGIN\n    value\n    END¦",
      "after": "BEGIN\n    value\n    END\n¦"
    },
    {
      "name": "vb block",
      "path": "sample.vb",
      "mode": "smart",
      "unit": "    ",
      "before": "Public Sub Run()¦",
      "after": "Public Sub Run()\n    ¦"
    },
    {
      "name": "vb closes",
      "path": "sample.vb",
      "mode": "smart",
      "unit": "    ",
      "before": "Public Sub Run()\n    value\n    End Sub¦",
      "after": "Public Sub Run()\n    value\n    End Sub\n¦"
    },
    {
      "name": "ruby else",
      "path": "sample.rb",
      "mode": "smart",
      "unit": "    ",
      "before": "if ok\n    work\nelse¦",
      "after": "if ok\n    work\nelse\n    ¦"
    },
    {
      "name": "haskell where",
      "path": "sample.hs",
      "mode": "smart",
      "unit": "    ",
      "before": "f x =¦",
      "after": "f x =\n    ¦"
    },
    {
      "name": "fsharp equals",
      "path": "sample.fs",
      "mode": "smart",
      "unit": "    ",
      "before": "let run () =¦",
      "after": "let run () =\n    ¦"
    },
    {
      "name": "yaml key",
      "path": "sample.yaml",
      "mode": "smart",
      "unit": "    ",
      "before": "key:¦",
      "after": "key:\n    ¦"
    },
    {
      "name": "yaml quoted scalar",
      "path": "sample.yaml",
      "mode": "smart",
      "unit": "    ",
      "before": "key: \"value\"¦",
      "after": "key: \"value\"\n¦"
    },
    {
      "name": "yaml literal opening",
      "path": "sample.yaml",
      "mode": "smart",
      "unit": "    ",
      "before": "key: |¦",
      "after": "key: |\n    ¦"
    },
    {
      "name": "yaml literal content",
      "path": "sample.yaml",
      "mode": "smart",
      "unit": "    ",
      "before": "key: |\n    text:¦",
      "after": "key: |\n    text:\n    ¦"
    },
    {
      "name": "html pair",
      "path": "sample.html",
      "mode": "smart",
      "unit": "    ",
      "before": "<div>¦</div>",
      "after": "<div>\n    ¦\n</div>"
    },
    {
      "name": "html void",
      "path": "sample.html",
      "mode": "smart",
      "unit": "    ",
      "before": "<br>¦",
      "after": "<br>\n¦"
    },
    {
      "name": "xml self closing",
      "path": "sample.xml",
      "mode": "smart",
      "unit": "    ",
      "before": "<item/>¦",
      "after": "<item/>\n¦"
    },
    {
      "name": "xml closing",
      "path": "sample.xml",
      "mode": "smart",
      "unit": "    ",
      "before": "<item>\n    value\n    </item>¦",
      "after": "<item>\n    value\n    </item>\n¦"
    },
    {
      "name": "fenced python",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "Prose:\n```python\ndef f():¦",
      "after": "Prose:\n```python\ndef f():\n    ¦"
    },
    {
      "name": "fenced ruby",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "~~~ruby\ndef f¦",
      "after": "~~~ruby\ndef f\n    ¦"
    },
    {
      "name": "hickory python",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "<h:file path=\"main.py\">\ndef f():¦",
      "after": "<h:file path=\"main.py\">\ndef f():\n    ¦"
    },
    {
      "name": "hickory typed copy",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "<h:copy id=\"body\" language=\"python\">\nif ok:¦",
      "after": "<h:copy id=\"body\" language=\"python\">\nif ok:\n    ¦"
    },
    {
      "name": "outside code",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "Prose:\n    words:¦",
      "after": "Prose:\n    words:\n    ¦"
    },
    {
      "name": "unknown fence",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "```unknown\nif fake:¦",
      "after": "```unknown\nif fake:\n¦"
    },
    {
      "name": "closed fence",
      "path": "sample.md",
      "mode": "smart",
      "unit": "    ",
      "before": "```python\nif ok:\n```\nNote:¦",
      "after": "```python\nif ok:\n```\nNote:\n¦"
    },
    {
      "name": "unknown path",
      "path": "sample.unknown",
      "mode": "smart",
      "unit": "    ",
      "before": "    anything {¦",
      "after": "    anything {\n    ¦"
    },
    {
      "name": "match mode",
      "path": "sample.py",
      "mode": "copy",
      "unit": "    ",
      "before": "    def f():¦",
      "after": "    def f():\n    ¦"
    },
    {
      "name": "none mode",
      "path": "sample.py",
      "mode": "none",
      "unit": "    ",
      "before": "    def f():¦",
      "after": "    def f():\n¦"
    },
    {
      "name": "unicode caret",
      "path": "sample.py",
      "mode": "smart",
      "unit": "    ",
      "before": "def café():¦",
      "after": "def café():\n    ¦"
    },
    {
      "name": "unbraced control",
      "before": "if (ready)¦",
      "after": "if (ready)\n    ¦",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    "
    },
    {
      "name": "unbraced body done",
      "before": "if (ready)\n    work();¦",
      "after": "if (ready)\n    work();\n¦",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    "
    },
    {
      "name": "nested unbraced body done",
      "before": "if (ready)\n    if (more)\n        work();¦",
      "after": "if (ready)\n    if (more)\n        work();\n¦",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    "
    },
    {
      "name": "regex literal brace",
      "before": "const re = /{/;¦",
      "after": "const re = /{/;\n¦",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    "
    },
    {
      "name": "regex class slash",
      "before": "const re = /[/{}]/;¦",
      "after": "const re = /[/{}]/;\n¦",
      "path": "sample.js",
      "mode": "smart",
      "unit": "    "
    },
    {
      "name": "html case insensitive",
      "before": "<DIV>¦</DIV>",
      "after": "<DIV>\n    ¦\n</DIV>",
      "path": "sample.html",
      "mode": "smart",
      "unit": "    "
    }
  ]
}
</ui:copy>
<ui:file path="apps/simple-contract/enter-cases.json"><ui:paste select="#simple-enter-fixtures"/></ui:file>

</ui:doc>
