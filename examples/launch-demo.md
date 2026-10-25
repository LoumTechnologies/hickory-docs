<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="launch-demo.md">
# Hickory Docs in one minute

This is a small, runnable product-demo document. Open it in Hickory Docs, then
press **Run**: prose, commands, tests, generated files, lineage, and the
diagram all come from this one Markdown file.

<hick:var name="release">Launch day</hick:var>
<hick:feature name="behind-the-scenes" description="Show the optional release note" />

## 1. Write the program where it is easiest to explain

This headline is a source fragment. It remains readable here and is assembled
into a real file below.

<hick:copy id="headline" class="announcement">Hello from Hickory Docs</hick:copy>
<hick:cut id="signature">— generated from one living document</hick:cut>

<hick:file path="launch-demo.txt">
<hick:paste select=".announcement" min="1" max="1" />
<hick:paste select="#signature" />
Release: <hick:val name="release" />
</hick:file>

The Output view shows `launch-demo.txt`; its lineage points each character
back to the fragment above.

## 2. A notebook, a REPL, or a document that ships the work

The same AI suggestion means different things in each surface. This is a real
table in Hickory: edit it as a grid here, or write it out as CSV for another
tool with a `path` attribute.

<hick:table>
Surface,The AI gives you,What remains reviewable,What can become the next input
REPL,a proposed command,the scrollback,whatever a person copies out
Jupyter notebook,a changed cell and output,the notebook cell,the next cell
Hickory Docs,prose plus a patch plus proof,the source and exact change,the whole explained project
</hick:table>

Here the “AI answer” is not a paragraph describing a change. It is an exact,
editable patch to a program the document already constructs:

<hick:copy id="release-function" class="release-program">def release_message(name):
    return f"Ship it, {name}."
</hick:copy>

<hick:copy id="release-main" class="release-program">if __name__ == "__main__":
    print(release_message("team"))
</hick:copy>

<hick:file id="release-program" path="ai-era-release.py" language="python">
<hick:paste select=".release-program" separator="&#10;" min="2" max="2" />
</hick:file>

```diff
--- a/ai-era-release.py
+++ b/ai-era-release.py
@@ -1,5 +1,5 @@
 def release_message(name):
-    return f"Ship it, {name}."
+    return f"Ship it, {name} — your evidence is attached."
 
 
 if __name__ == "__main__":
```

The patch is intentionally small enough to review in one glance. The AI can
propose it, while the document preserves why it belongs, what it changes, and
what later cells must prove. A declared Hickory project promotes this exact
unified diff into an editable `&lt;hick:patch&gt;` construction step.

## 3. Literate programming for an AI-sized project

The program above is deliberately split by meaning, not by file order: one
paragraph owns the promise, another owns the entry point, and the generated
file assembles both. An AI can improve one fragment, add a patch, or trace a
generated line back to its explanation without rewriting an opaque notebook
cell or guessing which terminal command mattered.

That is literate programming as a working surface: prose supplies intent,
named fragments supply code, patches record change, and execution supplies
evidence. The artifact is equally useful to the person reviewing it, the
agent extending it, and the tool running it.

## 4. Ask where a generated sentence came from

Open `launch-demo.txt` in the Output view and select **“Hello from Hickory
Docs.”** Its provenance ribbon leads straight back to the `&lt;hick:copy
id="headline"&gt;` block near the top of this page. Select the signature and
the ribbon lands on its hidden source fragment instead. The generated file is
not merely linked to this document: each span knows the source that produced
it.

The same evidence is available outside the editor:

```sh
hick lineage examples/launch-demo.md --output launch-demo.txt
```

That command reports the output-byte ranges, their source-byte ranges, and
whether each range came from a paste, literal text, or a computed value. It
answers “where did this sentence come from?” from the artifact itself—not from
an AI’s retrospective explanation.

## 5. Run a cell and make the claim executable

The environment card makes the execution context visible. The cell below has
no network access and its expected result is checked every time it runs.

<hick:container name="demo" image="alpine:3.20">
  <hick:deny network="*" />
</hick:container>

<hick:exec id="proof" container="demo">
printf 'Everything is connected.\n'
<hick:expect match="exact">Everything is connected.
</hick:expect>
</hick:exec>

## 6. Turn structure into a picture

The graph is part of the document, not a screenshot. Drag its boxes in the
editor to arrange the story without changing the declared relationships.

<hick:diagram renderer="graph" asserts="#proof">
{
  "topology": {
    "nodes": [{"id": "document"}, {"id": "verified output"}, {"id": "generated file"}],
    "edges": [
      {"from": "document", "to": "verified output"},
      {"from": "document", "to": "generated file"}
    ]
  },
  "layout": {
    "document": {"x": 60, "y": 40, "w": 180, "h": 56},
    "verified output": {"x": 60, "y": 160, "w": 180, "h": 56},
    "generated file": {"x": 300, "y": 160, "w": 180, "h": 56}
  }
}
</hick:diagram>

<hick:when feature="behind-the-scenes">
## Optional feature: reveal the release note

Feature-gated sections stay out of the default run and appear when you enable
their named feature.
</hick:when>

## What just happened

- **Markdown is executable:** this page contains the narrative and the cell.
- **Results are verified:** the expectation prevents a stale success claim.
- **Files are generated with lineage:** `launch-demo.txt` is assembled from
  named fragments, so edits have a visible source.
- **Provenance is byte-precise:** select text in an output or run `hick
  lineage` to trace it to the source that produced it.
- **AI changes stay inspectable:** `personalize-release` is a patch, not a
  chat suggestion that disappears into history.
- **Literate programming scales outward:** prose, source fragments, generated
  files, patches, and proof remain one connected project.
- **Diagrams are live:** topology is declared while layout stays editable.
- **Features are intentional:** the optional section is controlled by a named
  toggle instead of a forked document.
</hick:doc>
