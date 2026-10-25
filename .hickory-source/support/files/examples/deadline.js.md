<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/deadline.js

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `41035db30c9b4ae20206a6453441ff2531c80711ff8fa5c5ea9178e727269860`.

<book:entry path="examples/deadline.js">
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
</book:entry>

</book:doc>
