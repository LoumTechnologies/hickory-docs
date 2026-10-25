<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/ai-engineer-deadline.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `636ddbd13794d41faa0907e64e96bcf6d213a466f2dfd44423ae2b17a767099a`.

<book:entry path="examples/ai-engineer-deadline.md">
# A deadline that includes retries

## Meeting evidence

Sam: The total request deadline is 100 milliseconds, including retries.
Alex: Add a test where two attempts would each take 80 milliseconds.

## Requirement

R1: A request must consume no more than its total budget across all attempts.

## Recorded example agent proposal

The deadline bounds the entire operation.

<hick:file id="deadline" path="deadline.js">
function elapsed(attempts, budget) {
  var total = 0;
  for (var i = 0; i < attempts.length; i++) {
    var allowed = Math.max(0, budget - total);
    total += Math.min(attempts[i], allowed);
  }
  return total;
}
var actual = elapsed([80, 80], 100);
console.log("two 80ms attempts, 100ms total budget: " + actual);
if (actual !== 100) throw new Error("R1 failed: expected 100, got " + actual);
if (elapsed([20, 30], 100) !== 50) throw new Error("short attempts must remain 50");
if (elapsed([], 100) !== 0) throw new Error("no attempts must consume zero");
console.log("R1: all three checks passed");
</hick:file>

## Executable witness

Requires Node.js. The browser tour executes deadline.js with the browser debugger.
First run hick weave deadline-spec.md to materialize deadline.js, then hick test deadline-spec.md. The check runs that file; drift checking keeps it tied to this document.

<hick:container name="shell" image="node:22-alpine" />
<hick:volume name="src" input="." />
<hick:exec container="shell" mount="src:project">
node project/deadline.js
<hick:expect match="exact">two 80ms attempts, 100ms total budget: 100
R1: all three checks passed
</hick:expect>
</hick:exec>

## Scope

This deterministic model checks budget allocation, not OS scheduling or real network timing.
</book:entry>

</book:doc>
