<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/scaffolded-service.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This maintained example keeps the scaffold immutable and explains the authored patch above it. Historical requirement captures remain separate.


Maintained output SHA-256: `596679a249be401e05db1fb3cce6695fcc0b81d1c20b359a7b9cefee595fdf56`.

<book:entry path="examples/scaffolded-service.md">
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="scaffolded-service.md">
# Start with the scaffold; explain the changes

A scaffolder can write forty files while the interesting work changes four lines.
Keep its exact output in this document, folded by default in Hickory, then explain
an authored patch above that immutable base. A clone reconstructs the result
without rerunning the scaffolder or borrowing a transcript cache.

## Capture the starting point once

This small shell generator stands in for `cargo init`, `npm create` or `dotnet new`.
When authoring a fresh document, write its volume and command first, run
`hick ingest --from '#scaffold' scaffolded-service.md`, then add the project
construction and patches shown below. Ingestion is one-shot; rerunning a
scaffolder to replace an existing base is deferred.

<hick:container name="gen" image="alpine:3.20" />
<hick:volume name="project" output="service" />
<hick:exec id="scaffold-base" container="gen" mount="project:out">
<hick:copy id="scaffold">
mkdir -p out/src out/node_modules/left-pad
printf '{\n  "name": "greeter",\n  "version": "0.1.0",\n  "main": "src/index.js"\n}\n' > out/package.json
printf 'export function greet(who) {\n  return "hello, " + who;\n}\n' > out/src/index.js
printf '# greeter\n\nGenerated. Do not edit by hand.\n' > out/README.md
printf 'module.exports = function () {};\n' > out/node_modules/left-pad/index.js
ls out
</hick:copy>
<hick:ingested from="#scaffold" sha256="5826f56b23bcfcb25b1af1071b3ecfbbd92c24f36bae26a1473bbefc6fc05243" at="2026-08-24" files="3" skipped="1">
<hick:file path="service/README.md">
# greeter

Generated. Do not edit by hand.
</hick:file>
<hick:file path="service/package.json">
{
  "name": "greeter",
  "version": "0.1.0",
  "main": "src/index.js"
}
</hick:file>
<hick:file path="service/src/index.js">
export function greet(who) {
  return "hello, " + who;
}
</hick:file>
</hick:ingested>
</hick:exec>

The capture contains the three source files, with `node_modules/` omitted by the
repository's ignore policy. Its `sha256` covers sorted paths and their exact
bytes. Reconstruction refuses a changed path or byte. Hickory lets you unfold
and read these files, but edits belong in the patch below. The content hash is
an integrity check, not a signature authenticating a hostile edit to both data
and hash.

## Reject a greeting without a recipient

The scaffold accepts an empty recipient. Add a failure explicitly, preserving
the original capture. Declaring the predecessor makes the patch's input exact;
changed context fails rather than applying at a guessed offset.

<hick:project exclude="README.md agent-audit-evaluation/ ai-engineer-deadline.md ai-engineer-meeting.vtt ai-engineer-private-deadline.md ai-transform.md analysis.py architecture-that-draws-itself.md bootstrap-ci.md bootstrap-histogram.svg deadline.js demo/ diagram-f16290a0.svg grand-tour.md launch-demo.md messages/ planned-messages.md python-profile/ receipts/ regression-explorer.html security-authorization/ text-tools-tour.md" />
<hick:patch id="require-recipient" after="scaffold-base">
--- a/service/src/index.js
+++ b/service/src/index.js
@@ -1,3 +1,4 @@
 export function greet(who) {
+  if (!who) throw new TypeError("greet needs somebody to greet");
   return "hello, " + who;
 }
</hick:patch>

This is the maintained implementation change. It remains editable independently
of the base. Untouched output keeps its ingested provenance; the inserted check
points to the authored patch.

The project inventory explicitly excludes the neighboring demonstrations in this
checkout; those exclusions are harmless when this file is copied on its own.

## Reconstruct the result

Copy this one Markdown file into an empty directory, then run:

```sh
hick project materialize scaffolded-service.md
hick project verify scaffolded-service.md
```

These commands replay the captured base and patch; they do not execute the
scaffolder. File, directory and whole-capture folds change only presentation.
</hick:doc>
</book:entry>

</book:doc>
