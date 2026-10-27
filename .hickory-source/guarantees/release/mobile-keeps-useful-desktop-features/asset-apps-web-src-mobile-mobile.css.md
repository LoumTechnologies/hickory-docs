<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Fit the shared editor around a phone keyboard

Fit the shared editor around a phone keyboard. This literal construction preserves build configuration exactly.

<mobile:entry path="apps/web/src/mobile/mobile.css">
:root { color-scheme: light dark; font-family: system-ui, sans-serif; --accent: #3172b8; }
* { box-sizing: border-box; }
body { margin: 0; }
.mobile-app { max-width: 62rem; margin: auto; padding: max(1rem, env(safe-area-inset-top)) max(1rem, env(safe-area-inset-right)) max(1rem, env(safe-area-inset-bottom)) max(1rem, env(safe-area-inset-left)); }
h1 { margin: 0; font-size: 1.6rem; }
h2 { font-size: 1.15rem; }
p { line-height: 1.5; }
button, input, textarea, select { font: inherit; min-height: 44px; border: 1px solid #8888; border-radius: .4rem; padding: .55rem; }
button { cursor: pointer; margin: .2rem; }
button:disabled { cursor: default; opacity: .55; }
label { display: grid; gap: .3rem; margin: .8rem 0; }
input, textarea, select { width: 100%; }
textarea { min-height: 6rem; }
section, details { margin-top: 1rem; }
summary { min-height: 44px; padding: .6rem 0; cursor: pointer; }
.document-item { display: block; width: 100%; text-align: left; }
small { display: block; margin-top: .3rem; opacity: .7; }
[role="alert"] { border-left: 3px solid #bf701e; padding: .6rem; overflow-wrap: anywhere; }
.mobile-document { min-height: 100dvh; display: flex; flex-direction: column; }
.mobile-document .demo-editor { flex: 1; min-height: 36dvh; border: 1px solid #8888; border-radius: .4rem; }
.mobile-document .cm-editor { min-height: 36dvh; font-size: 16px; }
.mobile-document .cm-content { padding: .75rem; }
.mobile-document .cm-scroller { overflow: auto; }
.mobile-document pre { white-space: pre-wrap; overflow-wrap: anywhere; }
.mobile-document footer { padding-top: .75rem; }
.mobile-document footer p { font-size: .85rem; }
</mobile:entry>
</mobile:doc>
