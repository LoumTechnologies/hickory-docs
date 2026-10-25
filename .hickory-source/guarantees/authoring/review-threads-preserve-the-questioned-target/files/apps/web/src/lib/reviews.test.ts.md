<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/reviews.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Review threads preserve the questioned target](../../../../../contract.md)

Captured SHA-256: `eb081e9b3228b4ee6d44a8b73b0b75eb41d99c1801ba3e36886476b5d4eeccdc`.

<book:entry path="apps/web/src/lib/reviews.test.ts">
import { afterEach, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { registerReviewEditor, reviewRelationship, REVIEW_TARGET, targetFor, type ReviewTarget } from "./reviews";
afterEach(() => { vi.unstubAllGlobals(); window.getSelection()?.removeAllRanges(); });
function hashStub() { vi.stubGlobal("crypto",{subtle:{digest:async()=>new Uint8Array(32).buffer}}); }
// Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
it("converts ribbon editor offsets to the questioned lines", async () => {
  hashStub();
  const view=new EditorView({state:EditorState.create({doc:"heading\nactual claim\nlast line"})});
  const off=registerReviewEditor(view,"spec.md");
  let captured:ReviewTarget|null=null;
  const receive=(event:Event)=>{captured=(event as CustomEvent<ReviewTarget>).detail;};
  window.addEventListener(REVIEW_TARGET,receive);
  await reviewRelationship(view,[8,20],{family:"declared"});
  expect(captured).toMatchObject({quote:"actual claim",lines:[2,2],kind:"relationship"});
  window.removeEventListener(REVIEW_TARGET,receive);off();view.destroy();
});
it("anchors a filtered conversation to its full recorded source", async () => {
  hashStub();
  const view=new EditorView({state:EditorState.create({doc:"Agent claim"})});
  view.dispatch({selection:{anchor:0,head:11}});
  const off=registerReviewEditor(view,"session.md",()=>"earlier turn\n\nAgent claim\nlater turn");
  expect(await targetFor(view)).toMatchObject({path:"session.md",quote:"Agent claim",lines:[3,3]});
  off();view.destroy();
});
it("refuses ambiguous quotations in a filtered conversation", async () => {
  hashStub();
  const view=new EditorView({state:EditorState.create({doc:"claim"})});view.dispatch({selection:{anchor:0,head:5}});
  const off=registerReviewEditor(view,"session.md",()=>"claim\nclaim");
  await expect(targetFor(view)).rejects.toThrow("more than once");off();view.destroy();
});
</book:entry>

</book:doc>
