# Export a project for read-only browsing

Use this when you maintain a Hickory project and want source, syntax highlighting,
hover information and navigation available without a running language server.

Commit the project source first. Install Node 22+, Python 3 and the code-model
provider for the language you want to analyze. From the project directory:

```sh
hick project export book.md --output public/source \
  --model typescript=hick-model-typescript --install-dependencies
```

Replace `book.md` with your entry document. The command discovers the declared
source closure; it does not require `.hickory-source/` or Hickory's own ownership
maps. It defaults to the committed `HEAD` revision. Use `--revision COMMIT` or
`--root PATH` to select a different committed project. Dirty files and unrelated
notes are excluded.

`--install-dependencies` installs dependencies from the frozen npm lockfiles in a
temporary analysis directory, with package scripts disabled. Omit it for offline
analysis; unresolved imports remain diagnostics. Providers and their runtimes
must already be installed. The current bulk occurrence adapter supports
TypeScript and JavaScript. Other installed code models can still supply their
native type graphs.

The output contains readable `index.json`, gzip-compressed source and model
payloads (`.json.gz`), exact source documents with hash-bound syntax
and semantic data, and code-model graph artifacts. Serve these files alongside a
compatible read-only viewer, such as the source viewer built by this repository's
`just site`. The export command produces data artifacts, not an HTML website.
The viewer needs no Node, Python, model process or LSP connection. Use
`--uncompressed` for plain JSON payloads. To link unpublished generated files to
a public repository, supply `--source-url "https://github.com/ORG/REPO/blob/{revision}/"`;
without that explicit prefix, links stay within the exported source closure.

For TypeScript, the model indexes the constructed code once and exports types,
documentation, symbol relationships, call signatures, diagnostics, semantic
classifications, hints and folds. Hickory's byte provenance maps those locations
back to the authored fragments, including cross-page copies. Reference lists are
shared by symbol instead of repeated for every occurrence.

An optional `--lsp PATH` captures pages not covered by the selected model. This
fallback is slower. Without a covering provider, source and syntax remain
available and semantic state is unavailable. Read-only export does not enable
rename, formatting, completion edits or code actions.

The bulk model retains unresolved evidence and per-language structure. It does
not claim that every LSP answer or every language is supported. See diagnostics
and the selected provider's schema when interpreting incomplete bindings.
