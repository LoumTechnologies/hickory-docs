---
id: TASK-55
title: 'Secret scanning and substitution gate: prevent secrets reaching the LLM'
status: To Do
assignee: []
created_date: '2026-05-20 13:09'
labels:
  - security
  - sdk
  - llm-client
dependencies: []
references:
  - docs/specs/user-input.md
  - hick-agent/crates/hick-agent-sdk/src/lib.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a secret-scanning middleware layer that sits between the agent and the LLM client. Its job is to ensure secrets never reach a cloud LLM API, whether typed by the user, read from a file, or returned by a tool call.\n\n## Detection\n\nSecrets are detected using two complementary techniques:\n\n**Pattern matching**: a configurable set of regexes covering well-known secret formats. Shipped defaults:\n- Generic high-entropy strings ≥ 20 chars (Shannon entropy > 4.5 bits/char)\n- `sk-[A-Za-z0-9]{32,}` — OpenAI / Anthropic API keys\n- `ghp_[A-Za-z0-9]{36}` — GitHub personal access tokens\n- `AKIA[0-9A-Z]{16}` — AWS access key IDs\n- `xoxb-[0-9]+-[0-9]+-[A-Za-z0-9]+` — Slack bot tokens\n- `-----BEGIN (RSA|EC|OPENSSH) PRIVATE KEY-----`\n- `[A-Za-z0-9+/]{40,}={0,2}` with high entropy (catches base64-encoded secrets)\n\nUsers can add custom patterns in agent TOML under `security.secret_patterns`.\n\n**Scope**: scan is applied to:\n1. User message text before adding to the context window\n2. File contents returned by tool calls (read_file, bash stdout, etc.) before adding to context\n3. Web fetch response bodies before adding to context\n4. The full messages array immediately before every LLM API call (belt-and-suspenders)\n\n## Substitution\n\nWhen a secret is detected:\n\n1. Generate a deterministic placeholder: `{{HICK_SECRET:xxxx}}` where `xxxx` is the first 8 hex chars of the SHA-256 of the original value. This makes the same secret produce the same placeholder within a session, so the LLM can reference it consistently.\n2. Replace all occurrences of the real value in the message with the placeholder.\n3. Store the mapping `placeholder → real_value` in an in-memory, session-scoped `SecretRegistry`. This registry is never serialised to disk, never written to the session log, and never sent over the relay.\n4. Notify the user once per unique secret detected: `[secret detected and substituted: {{HICK_SECRET:xxxx}}]`.\n\n## Reverse substitution\n\nWhen the LLM response or a tool call argument contains a placeholder:\n\n1. Before executing a tool call: substitute placeholders back to real values in the argument list. The real values are used for execution; the tool result added to context uses the placeholder form.\n2. Before displaying to the user: substitute back so the user sees meaningful output.\n3. In the session log: keep placeholder form. Real secrets MUST NOT appear in the XML session log.\n\n## Secret fields from `<hick:ask type=\"secret\">`\n\nWhen the user fills in a `type=\"secret\"` form field (see `docs/specs/user-input.md`), the value is written directly to the `SecretRegistry` with a named placeholder (`{{HICK_SECRET:field_name}}`). It is NEVER added to the LLM context as a literal value.\n\n## FilteredLlmClient integration\n\nThe existing `FilteredLlmClient` in `hick-agent-sdk` is the right place for the pre-call scan. Add a `SecretFilter` layer that runs before the network filter. The `SecretRegistry` is owned by the session and passed to both the `SecretFilter` and to the tool-result ingestion path.\n\n## Non-goals\n\n- This feature does not scan the agent's own source code or workspace for secrets (that is a separate CI concern).\n- It does not prevent secrets from appearing in terminal output outside the agent (user's shell, other processes).\n- It does not encrypt the in-memory registry; if the process is compromised, secrets in memory are exposed like any other in-memory data.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Pasting a known-format secret (e.g. ghp_... token) into a user message replaces it with a placeholder before the LLM API call
- [ ] #2 Reading a file containing a secret via a tool call substitutes the secret before the content is added to LLM context
- [ ] #3 The full messages array is scanned immediately before every LLM API call as a final gate
- [ ] #4 The same secret always produces the same placeholder within a session
- [ ] #5 Placeholder is substituted back to real value in tool call arguments before execution
- [ ] #6 Real secrets never appear in the XML session log (placeholder form is kept)
- [ ] #7 User is notified once per unique secret detected (not on every occurrence)
- [ ] #8 Custom secret patterns are loadable from agent TOML (security.secret_patterns)
- [ ] #9 SecretRegistry is in-memory only; it is not written to disk, the relay, or the session log
- [ ] #10 hick:ask type=secret fields write directly to SecretRegistry and are never added to LLM context as plaintext
<!-- AC:END -->
