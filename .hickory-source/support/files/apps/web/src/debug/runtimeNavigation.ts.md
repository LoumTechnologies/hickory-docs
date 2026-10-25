<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/runtimeNavigation.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4219cd6695c336b43d999f4b535d762f8fa54126c103491516f2541c8300bcc7`.

<book:entry path="apps/web/src/debug/runtimeNavigation.ts">
export const RUNTIME_REVEAL = "hickory:runtime-reveal";
export function revealRuntimeEvent(run: string, event: string) {
  window.dispatchEvent(new CustomEvent(RUNTIME_REVEAL, { detail: { run, event } }));
}
</book:entry>

</book:doc>
