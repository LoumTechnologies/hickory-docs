
# Owning what a scaffolder wrote

`dotnet new webapi` writes forty files nobody typed, and the interesting work
is changing four lines across three of them. That leaves two bad options and
one good one.

**Paste the scaffold in** and the document becomes a silent snapshot: the next
SDK writes a different `Program.cs`, and the document keeps generating last
year's while claiming to be what the generator produces.

**Leave it out** and the scaffold is a step in a README, so the document no
longer describes the program it claims to build.

**Ingest it.** The generator's output becomes ordinary `hick:file` bytes inside
the document, marked with the run that produced them. Your four lines are then
ordinary edits, `hick lineage` reports the rest as *ingested* rather than as
text you wrote, and a clone rebuilds the whole tree without running the
generator at all.

This document does that, with a generator small enough to read.



## The volume the generator writes into

`output="service"` means the volume's contents land under `service/`, and they
keep landing there after the ingest — an ingest must not rearrange the tree it
was asked to preserve.



## The generator

A real one would be `npm create`, `cargo new`, or `dotnet new`. This one writes
the same *shape*: a few source files, plus a directory of dependencies that no
document should ever own.

The command lives in a `hick:copy` child rather than as loose text in the cell.
That is not decoration: once forty file bodies sit beside it, nothing would
mark which bytes get run.

$ mkdir -p out/src out/node_modules/left-pad
  printf '{\n  "name": "greeter",\n  "version": "0.1.0",\n  "main": "src/index.js"\n}\n' > out/package.json
  printf 'export function greet(who) {\n  return "hello, " + who;\n}\n' > out/src/index.js
  printf '# greeter\n\nGenerated. Do not edit by hand.\n' > out/README.md
  printf 'module.exports = function () {};\n' > out/node_modules/left-pad/index.js
  ls out
README.md
node_modules
package.json
src


*Ingested from `#scaffold` on 2026-08-24 — run `5826f56b23bc`, 3 file(s), 1 skipped. These bytes came from that run, not from this document's author.*

### `service/README.md`

```markdown
# greeter

Generated. Do not edit by hand.
```

### `service/package.json`

```json
{
  "name": "greeter",
  "version": "0.1.0",
  "main": "src/index.js"
}
```

### `service/src/index.js`

```javascript
export function greet(who) {
  if (!who) throw new TypeError("greet needs somebody to greet");
  return "hello, " + who;
}
```


## What ingest did with it

The block below was written by:



Three things about it are worth reading rather than skimming.

**`node_modules/` is not in it.** The repository's own `.gitignore` is the
filter — no new configuration, and it is what you already meant. A scaffolder
writes build output beside source, and a document that owns the source must
not carry the derived bytes. The count is recorded as `skipped=`.

**`sha256=` is the recorded base.** Running `hick ingest --from` again is a
three-way merge, not a second block: the bytes as they were ingested are the
base, the fresh run is theirs, and this document — including anything you
changed — is ours. The base is recovered from the commit that introduced that
fingerprint, because a hash verifies bytes rather than reconstructing them.

**The nesting is `exec > ingested > file`.** Never `exec > file`, because
`file > exec` already means the opposite — *run this, paste the output here* —
and the same two tags meaning inverse things by order would be unreadable.


## And now your four lines

Everything above is somebody else's. The edit below is not — it is an ordinary
change to ordinary document bytes, and it needs no new grammar, no anchors, and
no patch file.

The `TypeError` line in `service/src/index.js` above is not the generator's.
Open that file in any editor, change it further, and the change lands back here
byte-exactly — through the same reverse edit any generated file gets, because
these really are document bytes and not a patch.

**One thing to be accurate about**, since it is the sort of claim that would
otherwise be believed: `hick lineage service/src/index.js` reports the *whole*
file as `ingested`, including that line. Everything inside the block carries
the run's fingerprint, so what the report distinguishes is **this arrived from
that command** versus **somebody here typed this** — at the granularity of the
block, not of the line.

Telling your bytes from the generator's *within* the block would need the base
to compare against, and the base is not in this document: it is the version of
this file at the commit that introduced `sha256=`. Git holds it, which is the
same division of labour a re-ingest already relies on — but the weave does not
consult git, so the ribbon cannot either. Naming that limit is better than a
report that quietly guesses.

$ cat out/package.json
{
  "name": "greeter",
  "version": "0.1.0",
  "main": "src/index.js"
}


