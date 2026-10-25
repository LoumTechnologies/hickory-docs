<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/embed.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6cfb3992e6f9901a2f6a02fa6993172863a7ca6304113926fee11a8ad215b93e`.

<book:entry path="apps/web/src/embed/embed.css">
.hickory-embed {
  /* The embed owns its palette so it remains legible in an arbitrary host,
     but it deliberately uses the desktop editor's cool, restrained language. */
  --embed-bg: #1e2227;
  --embed-raised: #23282e;
  --embed-fg: #dde3ea;
  --embed-muted: #9aa5b1;
  --embed-border: #363d45;
  --embed-accent: #5eb0ef;
  --embed-teal: #46c4bc;
  --embed-string: #7fce8e;
  --embed-literal: #d9ab68;
  color: var(--embed-fg);
  background: var(--embed-bg);
  border: 1px solid var(--embed-border);
  border-radius: 6px;
  overflow: hidden;
  font: 14px/1.6 system-ui, sans-serif;
}
/* Short documents should remain compact. A host that needs a large editing
   canvas can size its own pane without forcing whitespace on every embed. */
.hickory-embed .cm-scroller { font-family: ui-monospace, monospace; }
.hickory-embed .cm-content { padding: 12px; caret-color: var(--embed-fg); }
.hickory-embed .cm-cursor { border-left-color: var(--embed-fg); }
.hickory-embed .cm-focused .cm-selectionBackground { background: #2b4a6f; }
.hickory-embed .cm-hick-tag { opacity: .65; }
.hickory-embed .cm-heading { font-weight: 700; color: var(--embed-fg); }
.hickory-embed .cm-gutters { color: var(--embed-muted); background: var(--embed-raised); border-color: var(--embed-border); }
.hickory-embed .hickory-assets { display: flex; gap: 12px; padding: 12px; }
.hickory-embed .hickory-assets:empty { display: none; }
.hickory-embed .hickory-assets img { max-width: 160px; max-height: 100px; }
.hickory-embed .cm-hick-chrome { color: var(--embed-muted); font-size: .8em; }
.hickory-embed .cm-md-heading { font-weight: 700; color: var(--embed-fg); }
.hickory-embed .cm-file-line, .hickory-embed .cm-cell-line { background: #191d21; border-left: 2px solid var(--embed-teal); font-family: ui-monospace, monospace; font-size: 13px; }
.hickory-embed .cm-file-chip, .hickory-embed .cm-block-banner { font: 12px system-ui, sans-serif; color: var(--embed-teal); padding: 0 6px; background: var(--embed-raised); border-radius: 3px; margin-left: 5px; }
.hickory-embed .tok-keyword, .hickory-embed .tok-typeName { color: var(--embed-accent); }
.hickory-embed .tok-string { color: var(--embed-string); }
.hickory-embed .tok-number { color: var(--embed-literal); }
.hickory-embed .tok-comment { color: var(--embed-muted); }
</book:entry>

</book:doc>
