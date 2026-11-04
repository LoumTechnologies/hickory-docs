# Basic web application contract

For authors and evaluators deciding what this version supports.

Given a valid deployment configuration and writable SQLite storage:

- A visitor can register, sign in and sign out through HTML forms. Passwords are
  salted PBKDF2-SHA256 hashes; session tokens are random and only their hashes
  persist. Successful account entry rotates the session. Logout revokes it.
- Sessions expire on the server and survive a process restart while unexpired.
- An authenticated user can create, read, update, complete and delete their own
  tasks through forms and JSON. Another user sees no task contents and receives
  404 for foreign IDs. A stale update/delete receives 409 without changing data.
- Missing/wrong CSRF tokens cannot mutate accounts or tasks. Titles are validated
  before persistence. Invalid JSON and media types receive 400/415; invalid domain
  input receives 422. No successful mutation precedes these refusals.
- HTML interpolation encodes user data. Redirects use validated local paths. SQL
  template values are bound parameters; transaction failures roll back.
- The application explicitly refuses disallowed origins. Allowed origins receive
  CORS headers on application successes and errors; supported preflights return
  204. Credentialed CORS uses an exact origin. This is not authentication.
- CSS assets come from an explicit catalog with ETags. HEAD suppresses response
  bodies. Unknown routes and unsupported methods return 404 and 405 respectively.
- The host bounds bodies, buffers responses, logs unexpected exceptions without
  returning their stack traces, and stops its owned listener on shutdown.

Across ordinary assembly boundaries, supported APIs require Html for an HTML
page, LocalUrl for a redirect, Principal for a protected read, WritePermit for a
reference-repository write, Title for a task title, Guid for a task route link,
and a CORS-prepared result for the transport. Consumers cannot directly construct
Html, WritePermit or Prepared. These are construction constraints, not a claim
that arbitrary C# programs cannot contain web vulnerabilities.

---

Verification status: partially verified. The maintained results report records
macOS runtime, compiler and HTTP witnesses and source hashes. Browser execution,
other operating systems and production assurance remain unverified. The focused witnesses
are tests/test_app.py (live HTTP, persistence, ownership, concurrency and refusal
state), tests/browser.cjs (real-browser forms), Checks/Program.cs (transaction and
capability boundaries), and evaluation/compiler_matrix.py (selected compiler
constraints). Test comments refer to this contract.

Untestable by HTTP alone: source-level compiler constraints, password work factor,
and whether application authors confine raw database/transport capabilities.
Compiler witnesses and source review cover those paths to the stated extent.
C# null suppression/reflection and application bypass remain possible. Capabilities
are not single-use and do not automatically expire if retained by an app author.
Transport-generated errors may predate the app’s CORS policy. The native driver
and deployment behavior require platform witnesses; local macOS results do not
establish Linux/Windows support. Server CORS decisions are exercised over HTTP. Actual browser enforcement is
not established by those header assertions; browser execution status is recorded
separately in the maintained evidence report. No production security
review or general AI-efficiency claim is made.
