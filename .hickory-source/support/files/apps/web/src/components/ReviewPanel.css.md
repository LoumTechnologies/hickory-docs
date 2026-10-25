<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ReviewPanel.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1de324529373aaceead0ae0fadf46edc6ad77860091d0ba5c0ba897b00706170`.

<book:entry path="apps/web/src/components/ReviewPanel.css">
.review-tools { display:flex; gap:1rem; padding:.3rem 1rem; flex-wrap:wrap; }
.status-bar { position:relative; z-index:20; }
.status-bar__right > .more-actions > .more-actions__items { top:auto; bottom:100%; max-height:min(32rem, calc(100vh - 3rem)); overflow:auto; }
.status-bar__right .more-actions__items .review-tools { padding:0; gap:.3rem; }
.status-bar__right .more-actions__items .workflow-tour-launcher { margin:0; }
.review-panel { position:fixed; right:1rem; top:4rem; z-index:150; width:min(30rem, calc(100vw - 2rem)); max-height:80vh; overflow:auto; background:var(--bg-raised); color:var(--fg); border:1px solid var(--border); border-radius:8px; padding:1rem; box-shadow:var(--shadow); }
.review-panel header { display:flex; justify-content:space-between; }
.review-panel nav { display:flex; gap:.4rem; flex-direction:column; margin:1rem 0; }
.review-panel textarea { width:100%; min-height:6rem; box-sizing:border-box; }
.review-panel blockquote, .review-panel pre { white-space:pre-wrap; overflow-wrap:anywhere; margin:.7rem 0; padding:.6rem; background:var(--bg-sunken); }
.review-panel code { overflow-wrap:anywhere; }
.review-panel article { border-top:1px solid var(--border); padding:.6rem 0; }
.review-panel button { margin:.2rem; }
</book:entry>

</book:doc>
