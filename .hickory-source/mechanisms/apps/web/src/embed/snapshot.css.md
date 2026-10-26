<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/snapshot.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f765a5f3c16c"></a>
## `apps/web/src/embed/snapshot.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/embed/snapshot.css">
.snapshot-header, .published-snapshot { max-width:72rem; margin:2rem auto; padding:0 1.5rem; line-height:1.55; }
.published-snapshot code { overflow-wrap:anywhere; }
.published-snapshot > section { margin:1.5rem 0; }
.published-snapshot__heading { display:flex; align-items:baseline; justify-content:space-between; gap:1rem; }
.published-snapshot__heading h1 { margin:0; }
.published-snapshot__status { margin:0; font-weight:700; }
.published-snapshot__status--failed { color:#d06d6d; }
.published-snapshot__status--passed { color:#7fce8e; }
.snapshot-provenance { margin:1rem 0 1.5rem; border:1px solid var(--border); border-radius:.35rem; background:var(--bg-raised); }
.snapshot-provenance > summary { cursor:pointer; padding:.5rem .65rem; color:var(--muted); }
.snapshot-provenance__contents { padding:0 .85rem .85rem; }
.snapshot-load-error { max-width:72rem; margin:1rem auto; padding:.85rem 1rem; border:1px solid #9f5a5a; border-left-width:3px; border-radius:.35rem; background:#352427; }
.snapshot-load-error h2, .snapshot-load-error p { margin:.15rem 0; }
</storycode:entry>

</storycode:doc>
