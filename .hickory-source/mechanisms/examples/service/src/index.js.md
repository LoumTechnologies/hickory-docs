<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain examples/service/src/index.js

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e1620ae3c852"></a>
## `examples/service/src/index.js`

<a id="f-e1620ae3c852-greet"></a>
### `greet`

<storycode:copy id="f-e1620ae3c852-greet">
export function greet(who) {
  if (!who) throw new TypeError("greet needs somebody to greet");
  return "hello, " + who;
}
</storycode:copy>

### Assemble `examples/service/src/index.js`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="examples/service/src/index.js"><storycode:paste select="#f-e1620ae3c852-greet"/></storycode:file>

</storycode:doc>
