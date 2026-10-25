<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/FeatureSettings.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `389b8b79ff1d0b3443223fe9410528b8e32c268a237206c70ff7e23f894d2d7e`.

<book:entry path="apps/web/src/components/FeatureSettings.css">
.feature-settings { position: relative; display: inline-flex; }
.feature-settings__gear { border: 0; background: transparent; color: var(--fg-muted); font-size: 1rem; cursor: pointer; padding: .2rem .35rem; }
.feature-settings__gear:hover, .feature-settings__gear[aria-expanded="true"] { color: var(--fg); background: var(--bg-hover); }
.feature-settings__panel { position: fixed; z-index: 900; width: min(22rem, 80vw); max-height: 50vh; overflow: auto; background: var(--bg); color: var(--fg); border: 1px solid var(--border); border-radius: 6px; padding: .75rem; box-shadow: 0 6px 20px #0004; display: flex; flex-direction: column; gap: .65rem; font-size: .8rem; font-weight: normal; }
.feature-settings__panel label { display: flex; align-items: center; gap: .4rem; }
.feature-settings__panel .chat-model { margin-left: 0; }
.feature-settings__panel .acp-controls { padding: 0; border: 0; }
.acp-controls:empty { display: none; }
</book:entry>

</book:doc>
