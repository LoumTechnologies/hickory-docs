---
skills:
  - implement-dev-environment
---

# Third-Party Integration Mocking

- **The dev environment must work for any combination of third-party
  accounts a developer has** — e.g. SendGrid but not Stripe, or neither.
  Never require the full set, and never assume "has none" or "has all" are
  the only cases.
- **A missing credential degrades gracefully** (log/no-op the call) rather
  than crashing the app or blocking unrelated features — including when the
  machine has no internet connection at all.
- **Mocking a third-party API locally is allowed, but is a deliberate,
  explicit decision, not a default.** Before adding a mock, say why the
  real graceful-degradation behavior (log/no-op) isn't enough for that
  integration, and weigh the ongoing cost of keeping the mock's behavior in
  sync with the real API.
- **When a mock is chosen for a storage-style API (e.g. S3), prefer a
  local-folder-backed mock** — writes land as real files under a local
  directory so what got "uploaded" is directly inspectable, instead of
  living only inside an opaque in-memory or containerized fake.
