import { ViewPlugin } from "@codemirror/view";

/** CodeMirror hides decorative gutters from accessibility; ours contain actions. */
export const accessibleGutterActions = ViewPlugin.define(view => {
  const expose = () => view.dom.querySelector(".cm-gutters")?.removeAttribute("aria-hidden");
  view.requestMeasure({ read: () => null, write: expose });
  return { update: expose };
});
