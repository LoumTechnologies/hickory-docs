<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/gutterActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `daddc09a9462055ee722a0987898f0f29c6fd991f1cee0b5b6cdafa930780610`.

<book:entry path="apps/web/src/editor/gutterActions.ts">
import { ViewPlugin } from "@codemirror/view";

/** CodeMirror hides decorative gutters from accessibility; ours contain actions. */
export const accessibleGutterActions = ViewPlugin.define(view => {
  const expose = () => view.dom.querySelector(".cm-gutters")?.removeAttribute("aria-hidden");
  view.requestMeasure({ read: () => null, write: expose });
  return { update: expose };
});
</book:entry>

</book:doc>
