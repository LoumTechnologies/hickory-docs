<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain examples/deadline.js

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-915891005eb7"></a>
## `examples/deadline.js`

<a id="f-915891005eb7-elapsed"></a>
### `elapsed`

<storycode:copy id="f-915891005eb7-elapsed">
function elapsed(attempts, budget) {
  var total = 0;
  for (var i = 0; i < attempts.length; i++) {
    var allowed = Math.max(0, budget - total);
    total += Math.min(attempts[i], allowed);
  }
  return total;
}
</storycode:copy>
<a id="f-915891005eb7-actual"></a>
### `actual`

<storycode:copy id="f-915891005eb7-actual">
var actual = elapsed([80, 80], 100);
</storycode:copy>
<a id="f-915891005eb7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-915891005eb7-expression_statement">
console.log("two 80ms attempts, 100ms total budget: " + actual);
</storycode:copy>
<a id="f-915891005eb7-if_statement"></a>
### `if_statement`

<storycode:copy id="f-915891005eb7-if_statement">
if (actual !== 100) throw new Error("R1 failed: expected 100, got " + actual);
</storycode:copy>
<a id="f-915891005eb7-if_statement-2"></a>
### `if_statement`

<storycode:copy id="f-915891005eb7-if_statement-2">
if (elapsed([20, 30], 100) !== 50) throw new Error("short attempts must remain 50");
</storycode:copy>
<a id="f-915891005eb7-if_statement-3"></a>
### `if_statement`

<storycode:copy id="f-915891005eb7-if_statement-3">
if (elapsed([], 100) !== 0) throw new Error("no attempts must consume zero");
</storycode:copy>
<a id="f-915891005eb7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-915891005eb7-expression_statement-2">
console.log("R1: all three checks passed");
</storycode:copy>

### Assemble `examples/deadline.js`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="examples/deadline.js"><storycode:paste select="#f-915891005eb7-elapsed"/><storycode:paste select="#f-915891005eb7-actual"/><storycode:paste select="#f-915891005eb7-expression_statement"/><storycode:paste select="#f-915891005eb7-if_statement"/><storycode:paste select="#f-915891005eb7-if_statement-2"/><storycode:paste select="#f-915891005eb7-if_statement-3"/><storycode:paste select="#f-915891005eb7-expression_statement-2"/></storycode:file>

</storycode:doc>
