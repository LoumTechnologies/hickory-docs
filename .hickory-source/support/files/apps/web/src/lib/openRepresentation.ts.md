<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/openRepresentation.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e81f030873e5e867eebb9b455ea37156f657394d8c3540c1b96f9ad51e40327d`.

<book:entry path="apps/web/src/lib/openRepresentation.ts">
import { representations, type Backing } from "../api/representations";
export const OPEN_REPRESENTATION = "hickory.open-representation";
export function showRepresentation(id: string, base?: string, target?: string) {
  window.dispatchEvent(new CustomEvent(OPEN_REPRESENTATION, { detail: { id, base, target } }));
}
export async function openLiterate(backing: Backing, base?: string, target?: string) {
  const view = await representations.create(backing);
  showRepresentation(view.id, base, target);
}
</book:entry>

</book:doc>
