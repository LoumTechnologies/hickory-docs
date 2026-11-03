<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/framework-agnostic-system-tests.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This literal entry preserves the committed repository guidance, including the skill refresh in `efbe277f`. Edit this owner when the guidance changes; regeneration must retain the exact payload.


Current payload SHA-256: `f61f59313e9ebf415e54bb45aa9c6d02e5df21375c724ba3e3f7fc68c0281b93`.

<book:entry path=".instructions/framework-agnostic-system-tests.md">
---
skills:
  - implement-dev-environment
---

Always maintain a comprehensive system test suite that is agnostic to the tech stack used to implement the app. The purpose of this system test suite is to be able to rewrite the entire app in a different tech stack without changing a single line of system test code. System tests must interact with the app the same way a real client would: real HTTP requests, real browser automation. System tests must never depend on the web framework's own in-process test harness (e.g., Django's `LiveServerTest` is an anti-pattern for these system tests).

Every requirement must be codified in a system test if at all possible; if not possible, you must notify me. All changes to requirements should include a change to system tests; if they do not, that is a shortcoming of the system test suite which must be remedied; notify me about this as well.

Maintain an abstraction layer between the system test suite and the app code that abstracts tech stack-specific details away from the system test suite. Rewriting the app may require rewriting this abstraction layer.

If the app has multiple frontends or APIs with the same functionality, this abstraction layer should provide an interface to the systems tests with a separate implementation for each, so the same exact system test can run against each frontend or API by changing out which abstraction layer implementation it uses.

These system tests must be implemented in an appropriate programming language that is not part of the tech stack. This helps avoid accidental dependencies between the app implementation and the system tests.

System tests should be grouped together such that there are as few groups as possible and it is not possible for there to be any order in which the system tests within a group can be executed that could cause flakiness. Groups should be run in sequence in CI and pre-commit hooks.
</book:entry>

</book:doc>
