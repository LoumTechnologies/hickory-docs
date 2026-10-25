<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/messages/2026-08-17-eng-standup.txt

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `91a572ba3a7c33c06263b507c72310cfdcdb57de3c2d4386fef49577ab3e41fd`.

<book:entry path="examples/messages/2026-08-17-eng-standup.txt">
Standup, 2026-08-17:

p95 checkout latency dropped from 480ms to 210ms after the connection-pool fix (Grafana dashboard "checkout-slo", 2026-08-16 snapshot).

The release-pipeline incident (2026-08-14, three independent failures) is closed; fixes verified in CI run #412.

Remaining risk this week: the include-lineage migration touches every woven document; rollback is a git revert.

Questions welcome — every line above has a source; ask me and I will point at it.
</book:entry>

</book:doc>
