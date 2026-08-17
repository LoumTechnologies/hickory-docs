# Every Hover Hint Is Drawn In The App Theme

Given any control, row, or gutter marker in the app that explains itself on
hover, when the pointer rests on it (or keyboard focus reaches it), then the
explanation is drawn by the app in the palette the user chose — the same
tokens as every other raised surface — and never by the operating system's
`title=` tooltip.

The reason is that `title=` is drawn by the browser and the OS: a white card
in the system font, at the system's own delay, with the system's own wrapping.
Against the default dark editor palette that reads as a piece of another
program showing through, and it changes appearance per platform, which is not
something a downloadable desktop app can hide behind "that's just the
browser". The three themes in `styles.css` are a promise about the whole
window; a hover hint is inside the window.

Concretely: no JSX in `apps/web/src` sets a `title` attribute for explanatory
text, and no editor decoration sets `.title` on a DOM node. They carry
`data-tip` instead, and a single `TooltipLayer`, mounted once per entry point
(the app and the marketing site), draws it.

A tooltip is a picture, not an accessible name. An icon-only control still
carries its own `aria-label`; removing `title=` must never take a control's
name away from a screen reader, because `title` was silently serving as the
accessible name of anything whose whole content was a glyph.

The card is placed above its anchor where there is room, flips below where
there is not, and is clamped inside the viewport in both axes — a hint that
opens off-screen is the same as no hint. It never takes pointer events, and it
disappears on scroll, resize, Escape, and any press, so it can never be left
pointing at a row that has moved.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - Placement is pure and tested: `apps/web/src/lib/tooltip.ts::placeTip`
    (above by preference, flip below, clamp both axes) and `tipTargetOf`
    (delegation from whatever child the pointer is over, empty text ignored),
    covered by `apps/web/src/lib/tooltip.test.ts`.
  - The layer: `apps/web/src/components/TooltipLayer.tsx` — one delegated
    `pointerover`/`focusin` listener, a 450ms rest delay, and hide on
    `pointerdown`, `focusout`, Escape, scroll (captured, so a scroll inside
    any pane counts), resize, and window blur. Rendered through a portal on
    `document.body` with `aria-hidden`, so the control's own label is what is
    announced. `TooltipLayer.test.tsx` covers show-through-a-child, each hide
    path, an anchor removed during the delay, and listener teardown.
  - Mounted in both entry points: `apps/web/src/App.tsx` (desktop app) and
    `apps/web/src/views/LandingView.tsx` (static site, whose demos have
    tooltips of their own).
  - Styling: `.tip` in `apps/web/src/styles.css` uses `--bg-raised`,
    `--border`, and `--fg`, matching `.cm-tooltip` — the CodeMirror hover
    surface, which was already themed — so the two tooltip surfaces in the
    app are one surface.
  - Call sites converted: `title=` → `data-tip=` across the shell
    (`ShellView`, `FolderTreePane`), workspace (`workspaceTabs`, `CellPanel`,
    `OutputTree`, `SearchPanel`, `EnvCard`, `ChatDock`), lineage
    (`StageColumn`, `HoleRow`, `LineageColumns`), settings, the landing demo
    (`DemoSplit`), and the debug strip; and `.title =` → `.dataset.tip =` in
    the editor decorations (`debug/cmDebug.ts` breakpoint dots, paused arrow,
    frame markers, eval toggle; `editor/folding.ts` fold markers).
  - Enforced against future code, not just present code: the
    `no native tooltips` case in `tooltip.test.ts` walks every non-test source
    under `apps/web/src` and fails on a `title=` attribute or a `.title =`
    assignment (`document.title`, the window title, is exempt — a different
    thing entirely). `npx tsc -b --noEmit` clean; all 676 web tests pass.
  - Seen running: the mock dev server driven with Playwright, hovering the
    divider's port button, screenshotted under `dark` and `light` — the card
    is the pane surface in both, above the anchor, and the OS tooltip is gone.
- Caveats — what LLM review could NOT establish:
  - `warm-dark` was not screenshotted; it shares the token names, so it is
    asserted by the palette rather than seen.
  - Only the desktop build was driven. The marketing site mounts the same
    layer (`LandingView`), but its demo tooltips were not hovered live.
  - Native `title` remains correct in one place it is not used for hints:
    nothing in the app currently relies on it, but a future `<abbr>` or an
    `<iframe title>` is a different attribute with a different job and is not
    covered by this guarantee.
