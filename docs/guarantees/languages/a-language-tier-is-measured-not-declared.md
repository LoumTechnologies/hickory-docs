# A Language Tier Is Measured, Not Declared

Given a language Hickory can route, when `hick lang` reports its tier, then
every column is computed from the catalogue or discovery path that would
actually serve the request — never from a list written down beside it. A
language is reported as having a language server exactly when discovery knows
a candidate for it or a catalogue can install one; as debuggable exactly when
the same is true of an adapter; and as Gold only when a code model server
exists for it, which is true of no language today.

A data or markup language is **exempt on the axes that do not apply to it**
rather than reported as lacking them: its debugger, index and model columns
read `n/a`, Silver is the top of its ladder, and it is never rounded up to
Gold because three columns cannot apply.

Bronze does not claim highlighting. Whether the editor can draw a language is
its own measured column, `DRAW`, and is guaranteed separately by
`a-routed-language-is-drawn.md`.

## Why

Five bugs of one shape have shipped here, each a hand-maintained list beside
the thing it described: `lang_detect` with no `cs` row (C# support existed and
was unreachable), `hick init`'s report omitting C#, and both
`known_languages()` functions omitting the React language ids while their
`candidates()` served them — so `.tsx` files were reported as having neither
a language server nor a debugger on a machine that had both. The fifth was the
web app's own language table, which disagreed with the routing table about
nineteen extensions in both directions — see `a-routed-language-is-drawn.md`,
which is also where the fix that removes the second list is recorded.

None was caught by a test, because a list that is the only statement of its
own contents cannot be checked against anything. The rule is therefore about
where a fact comes from, not about keeping two facts in step.

---

Last LLM verification:
- Date: 2026-08-28
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/language_tier.rs` — `survey` iterates
  `hick_lsp::lang_detect::known_language_ids()` (derived from the `EXTENSIONS`
  routing table, not a second list); `support` reads
  `hick_lsp::discovery::discover`/`known_languages`,
  `hick_dap::discovery::discover`/`known_languages`,
  `index_install::discover`, and the three `tool_install::Catalogue`s.
  `Have::NotApplicable` carries the data-language exemption and `counts()`
  treats it as satisfied while `tier` caps such languages at Silver.
- Test coverage: `language_tier::tests::every_installable_language_is_surveyed`
  (derives from all three catalogues),
  `every_ecosystem_indexed_language_is_surveyed`,
  `a_tier_needs_every_rung_below_it`;
  `hick_lsp::lang_detect::tests::every_routable_language_is_enumerable`;
  `hick_lsp::discovery::tests::known_languages_and_candidates_agree` and
  `hick_dap::discovery::tests::known_languages_and_candidates_agree` (both
  directions, which is what closes the class).
- Caveat requiring LLM review: `indexed_by_the_ecosystem` is still a literal
  list, kept in step with `index_install::how_to_get` by a test that names the
  same languages. It is the one remaining hand-maintained row in this module
  and should be derived if `how_to_get` ever grows a machine-readable form.
