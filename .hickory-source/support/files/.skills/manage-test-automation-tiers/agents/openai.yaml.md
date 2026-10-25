<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .skills/manage-test-automation-tiers/agents/openai.yaml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `240764c4d662f4de58a26aaa8874a24c3053304080b9ed16b87719f2a51db2b9`.

<book:entry path=".skills/manage-test-automation-tiers/agents/openai.yaml">
interface:
  display_name: "Manage Test Automation Tiers"
  short_description: "Grade tests eyeball → watched → automated, demote on code-change blast radius"
  default_prompt: "Use $manage-test-automation-tiers to set up a trust ladder (eyeball → watched → automated) for this project's tests/flows, tracking state in a single test-tiers.yaml and using code-review-graph's blast-radius analysis against each test's real code-coverage report to demote a test back to human eyeballs whenever a code change touches lines it actually covers, promoting it back after enough confirmed clean runs."
</book:entry>

</book:doc>
