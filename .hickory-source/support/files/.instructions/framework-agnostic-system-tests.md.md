<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/framework-agnostic-system-tests.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `b7fef8eed87f9853f7216cc8ce1a57419017ea976d026b3f3a29dcb3ce2b06b1`.

<book:entry path=".instructions/framework-agnostic-system-tests.md">
---
skills:
  - implement-dev-environment
---

# Framework-Agnostic System Tests

- **System and integration tests drive the app the same way a real client
  would** — real HTTP requests, real browser automation — and must never
  depend on the web framework's own in-process test harness (e.g. Django's
  `LiveServerTestCase`, a framework-specific test client that bypasses the
  network).
- The goal: the app could be reimplemented in a different language or
  framework without changing a single line of these tests. If a test
  imports anything from the app framework, it isn't a system test anymore.
- **Unit tests are exempt** — they're expected to be rewritten on a
  framework or language change, and are allowed to use framework-specific
  test utilities.
</book:entry>

</book:doc>
