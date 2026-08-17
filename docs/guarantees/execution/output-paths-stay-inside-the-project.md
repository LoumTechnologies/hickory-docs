# An Output Path Never Escapes The Document's Directory

Given any document, when a `<hick:file path=…>` declares a path that is
absolute or whose `..` components climb out of the document's output
directory, then no write, read, or drift comparison uses that path: the run
fails naming the offending path, why it was refused, and what a valid path
looks like.

A document is a file people send each other and agents write — the same
threat model that makes the sandbox the default executor. `Path::join`
silently **discards** its base when the right-hand side is absolute, so
without this check `hick weave` — a command that executes nothing — would
write `~/.ssh/authorized_keys` or `../../.git/hooks/pre-commit` on behalf of
a hostile document. Path traversal through an output declaration is a shorter
route to code execution than anything the sandbox confines.

`..` that stays inside the base (`a/../b.txt`) is allowed; the check is
lexical depth, not a string match on dots.

## Boundary

The guard covers document-declared output paths (`write_outputs`,
`write_missing_outputs`, drift comparison, and the `hick up` loop). It does
not police what an executed cell writes — that is the sandbox's job — nor the
serve API's document-creation path, which has its own containment check.

---

**Verification notes (2026-08-16).** Implemented as
`contained_output_path` in `crates/hickory-cli/src/lib.rs`, called from
`write_outputs`, `write_files_if_absent`, `check_failures` (same file), and
`weave_and_write` in `crates/hickory-cli/src/up/mod.rs`. Test coverage:
`contained_output_path_tests` in `crates/hickory-cli/src/lib.rs` (absolute
paths, escaping `..`, in-bounds `..`, Windows drive prefixes). Caveat: the
interpolated `path=` value is checked after `{{var}}` substitution, so a
hostile `--param` is covered too; binary outputs staged by `hick-literate`'s
volume seeding are not routed through this function and rely on the
sandbox.
