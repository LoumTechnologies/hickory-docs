<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/FeatureSettings.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2d90f09a0bbb"></a>
## `apps/web/src/components/FeatureSettings.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/FeatureSettings.css">
.feature-settings { position: relative; display: inline-flex; }
.feature-settings__gear { border: 0; background: transparent; color: var(--fg-muted); font-size: 1rem; cursor: pointer; padding: .2rem .35rem; }
.feature-settings__gear:hover, .feature-settings__gear[aria-expanded="true"] { color: var(--fg); background: var(--bg-hover); }
.feature-settings__panel { position: fixed; z-index: 900; width: min(22rem, 80vw); max-height: 50vh; overflow: auto; background: var(--bg); color: var(--fg); border: 1px solid var(--border); border-radius: 6px; padding: .75rem; box-shadow: 0 6px 20px #0004; display: flex; flex-direction: column; gap: .65rem; font-size: .8rem; font-weight: normal; }
.feature-settings__panel label { display: flex; align-items: center; gap: .4rem; }
.feature-settings__panel .chat-model { margin-left: 0; }
.feature-settings__panel .acp-controls { padding: 0; border: 0; }
.acp-controls:empty { display: none; }
</storycode:entry>

</storycode:doc>
