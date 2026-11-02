<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep source navigation readable beside code

For engineers maintaining the public source browser.

Constrain the prose width while allowing implementation blocks a wider measure. Related chapters wrap into columns, and the searchable picker remains within the viewport on small screens.

<storycode:copy id="public-source-apps-web-src-embed-source-css">
.source-browser { max-width: 100rem; margin: 0 auto; padding: 2rem clamp(1rem, 4vw, 4rem); }
.source-browser header { max-width: 64rem; }
.source-browser h2, .source-browser nav a { overflow-wrap: anywhere; }
.source-browser nav ul { columns: 2 24rem; padding-left: 1.25rem; }
.source-browser nav li { break-inside: avoid; margin-bottom: .5rem; }
.source-browser label { display: block; margin: 1rem 0; }
.source-browser input, .source-browser select { display: block; width: min(100%, 60rem); margin-top: .5rem; padding: .5rem; }
.source-browser .hickory-embed { margin-top: 1.5rem; }
</storycode:copy>

<storycode:file path="apps/web/src/embed/source.css"><storycode:paste select="#public-source-apps-web-src-embed-source-css"/></storycode:file>
</storycode:doc>
