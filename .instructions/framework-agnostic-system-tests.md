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
