<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/density.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The editor keeps its space](../../../../../contract.md)

Captured SHA-256: `35224dc500c9cedf70fa7116772627d4aece23a1ea32d6931a1fbcf22c2bdb1d`.

<book:entry path="apps/web/src/editor/density.css">
/* Density changes spacing, never source bytes or the chosen font size. */
:root {
  --editor-code-leading: 1.4;
  --editor-document-leading: 1.45;
  --editor-pad-top: .5rem;
  --editor-pad-x: .75rem;
  --editor-pad-bottom: .75rem;
  --editor-number-pad: .5em;
  --workspace-padding: .25rem;
}
:root[data-editor-density=comfortable] {
  --editor-code-leading: 1.5;
  --editor-document-leading: 1.65;
  --editor-pad-top: 2rem;
  --editor-pad-x: 2.5rem;
  --editor-pad-bottom: 3rem;
  --editor-number-pad: 1em;
  --workspace-padding: 1rem;
}
:root[data-touch-targets=true] .spatial-shell .spatial-files .filesystem-editor .cm-line,
:root[data-touch-targets=true] .spatial-shell .spatial-files .filesystem-editor .cm-gutterElement { min-height: 36px; line-height: 36px; }

:root[data-right-line-numbers=false] .cm-right-rail { display: none; }

:root .doc-page.wide-mode .doc-main { padding: var(--workspace-padding, .25rem); }

.cm-card-rail[data-empty=true] { width: 0; border-left: 0; }
</book:entry>

</book:doc>
