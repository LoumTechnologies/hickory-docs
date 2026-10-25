<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/service/src/index.js

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d7d72f53080bc74e95f257f41f072d21b43f93cc7f882deae1c6cd775c5e5b85`.

<book:entry path="examples/service/src/index.js">
export function greet(who) {
  if (!who) throw new TypeError("greet needs somebody to greet");
  return "hello, " + who;
}
</book:entry>

</book:doc>
