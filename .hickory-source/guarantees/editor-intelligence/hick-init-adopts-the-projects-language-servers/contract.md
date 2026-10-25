<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# `hick init` Adopts The Project's Own Language Servers

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a repository whose editor configuration already names a language server
for a language — `"python.languageServer": "Pylsp"` in `.vscode/settings.json`,
a `language_servers` list in `.zed/settings.json`, a `[language-server.…]`
entry bound to a `[[language]]` in `.helix/languages.toml` — when `hick init`
runs in that repository, then it records that server's command in
`.hick-lsp.json`, and `hick-lsp` spawns **that** server for `hick:file` blocks
of that language instead of its own built-in default.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/hick-init-adopts-the-projects-language-servers.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
