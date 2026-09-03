# A Routed Language Is Drawn, Or Says It Is Not

Given a language Hickory routes, when a file or a `hick:file` block holding
it is opened — in the document editor, the output pane, or the plain-file
pane — then it is syntax-highlighted, in exactly the set of languages
`hick lang`'s `DRAW` column reports as `yes`.

Which languages exist is stated **once**, in the server's routing table
(`crates/hick-lsp/src/lang_detect.rs`), and reaches the web app by being
emitted (`just codegen` → `apps/web/src/editor/generated/languages.ts`).
The web app decides only **how** each is drawn. A language marked
`Highlight::Editor` with no grammar bound, or a grammar bound for a language
the table does not mark, fails a test rather than reaching a person.

A language for which no grammar exists anywhere — Nix and Zig today — is
marked `Highlight::PlainText`, opens as plain text, and is reported that way.
Bronze therefore no longer describes itself as "highlighted": it says the
language is routed and runnable, and `DRAW` answers the rest.

## Why

This is the fifth instance of one bug: a hand-maintained list of languages
kept beside the thing it described. The first four are recorded in
`a-language-tier-is-measured-not-declared.md`. The fifth was found on
2026-09-03 and was the widest: `apps/web/src/editor/languages.ts` kept its own
table, and the two disagreed in **both directions at once**.

Routed, reported as at least Bronze — whose text then read "the language is
routed, **highlighted**, and runnable" — and opening as undifferentiated grey
text: `toml`, `yaml`, `yml`, `go`, `java`, `c`, `cpp`, `cc`, `cxx`, `h`,
`hpp`, `rb`, `php`, `swift`, `kt`, `scala`, `lua`, `zig`, `nix`. That is
GoLand, IntelliJ, CLion, RubyMine, PhpStorm and AppCode's languages, all of
them silently plain.

Highlighted beautifully and routed nowhere, so no language server, no
debugger, and no row in `hick lang` at all: `sql`, `xml`, `csproj`, `xaml`,
`props`, `targets`, `xsd`, `htm`, `mjs`, `cjs`, `zsh`. `language_tier`'s
`is_data_language` already named `sql`, which made that arm dead code — the
table had been written for a language the router had never heard of.

A test comparing the two lists would not have been enough, because it still
leaves two places to edit and a person to keep them in step. Emitting one from
the other removes the second place.

The same edit found a sixth, one layer down: `parserForLanguage` read a
parser only at `support.language.parser`, which is where a lezer package puts
it. `StreamLanguage.define()` returns a `Language`, which carries `.parser`
itself — so every legacy-mode language returned `null` and rendered as a
blank block rather than an error. The shell, C# and XML had been affected the
whole time.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-lsp/src/lang_detect.rs` — `EXTENSIONS` rows carry
  `Highlight`, and `highlight()` reads them; `crates/hick-lsp/src/bin/emit-languages.rs`
  emits the table; `just codegen` writes it and `just check-codegen` (wired
  into `just ci`) fails on drift; `apps/web/src/editor/languages.ts` imports
  the generated module and binds `GRAMMARS`, with `boundGrammars()` exposed
  for the drift test; `crates/hickory-cli/src/language_tier.rs` computes
  `draw` from `lang_detect::highlight` and `next_step` names it first,
  because it is the lowest rung.
- Test coverage: `apps/web/src/editor/languages.test.ts` — "draws every
  language the server says is drawable", "claims no grammar the server does
  not expect", "resolves every routed extension to its language", "actually
  attaches a grammar for every drawable language" (which is what catches the
  `StreamLanguage` parser bug — an id being *recognised* was never the
  question), and "degrades honestly for a routed language with no grammar";
  `hick_lsp::lang_detect::tests::one_language_has_one_highlight_answer`,
  `the_jetbrains_pack_is_routed`, `extensions_are_case_insensitive`,
  `rows_are_spelled_the_way_the_lookup_reads_them`.
- Caveat requiring LLM review: the guarantee is checked at the level of "a
  grammar is attached", not "the colours are right". A grammar that loads and
  mis-tokenises would pass. The legacy modes are upstream and widely used,
  which is the argument for accepting that; a screenshot test per language is
  the thing that would close it.
