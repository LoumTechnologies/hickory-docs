<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/DiffView.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `610d39177ade9c20ace8aa9d039ed44a2122591899de4ab6e54adc1b87d0c1d6`.

<book:entry path="apps/web/src/components/DiffView.css">
.diff-breakpoints { display: inline-flex; width: 3rem; vertical-align: baseline; }
.diff-breakpoints > span { display: inline-block; width: 1.5rem; }
.diff-breakpoint { border: 0; padding: 0; width: 1.5rem; background: transparent; color: var(--fg-muted); font: inherit; cursor: pointer; }
.diff-breakpoint:hover, .diff-breakpoint[aria-pressed="true"] { color: var(--fail); }
.diff-breakpoint:focus-visible { outline: 1px solid var(--accent-2); }
</book:entry>

</book:doc>
