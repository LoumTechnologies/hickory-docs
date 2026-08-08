# Staging And Production Parity

Staging exists to catch what a local dev environment cannot. It earns that only
by differing from production in **size**, not in **shape**.

## What may differ

- Instance and database sizes.
- Replica, worker, and node counts.
- Backup retention and log retention windows.
- Traffic volume — staging does not load test.
- Third-party accounts in sandbox/test mode.

## What may not differ

Unless the cost or vendor constraint is explicit and written down:

- Network topology.
- The deploy path itself.
- TLS mechanism and certificate automation.
- Database engine and version.
- Migration path.
- DNS shape.
- Runtime roles and permissions model.
- Configuration schema — the same variable names, per
  `config-and-environments`.

A staging environment reached a different way than production tests the wrong
thing.

## Separation

Use separate domains for production and staging, and keep orthogonal:

- DNS records
- OAuth callbacks and client ids
- Cookie domains
- Webhook endpoints and signing secrets
- TLS issuance
- API keys and sender identities

## Staging data

Staging sends real email, so:

- **Never import real customer data into staging by default.**
- Gate outbound email, SMS, and webhooks so staging cannot contact a real
  customer even by accident.
- Prefer real integrations in restricted/test mode over stubbing them out — a
  sandbox exercises the real code path.
- Evaluate **production** data for its awkward shapes (NULLs, missing values,
  mistyped entries, unusual lengths) and maintain a seed script that reproduces
  those shapes in staging. Seeding pristine data hides exactly the bugs staging
  is for.

## Reducing staging cost

When staging cost is too high, reduce it in this order:

1. Smaller instance and database sizes.
2. Fewer replicas or workers.
3. Shorter retention and backup windows.
4. Lower traffic or synthetic load.
5. Sandbox/test-mode third-party accounts.
6. Architectural differences — only when the savings are material **and** the
   reduced coverage is stated explicitly.

Tearing staging down between uses (a Teardown Staging workflow) is a legitimate
cost lever; bring it back before the next serious test.

**Never "save money" by moving staging onto a developer machine.** That is not
staging, it is the local dev environment wearing a misleading name.
